mod manifest;
mod shader;

use std::path::{Path, PathBuf};
use std::sync::mpsc::channel as reply_channel;
use std::sync::{Arc, RwLock};
use std::time::{Duration, Instant};

use bytemuck::{Pod, Zeroable};
use smithay_client_toolkit::{
    compositor::{CompositorHandler, CompositorState, FrameCallbackData},
    delegate_registry,
    output::{OutputHandler, OutputState},
    registry::{ProvidesRegistryState, RegistryState},
    registry_handlers,
    reexports::{
        calloop::{
            EventLoop, LoopSignal,
            channel::{self, Channel, Event as ChannelEvent},
            signals::{Signal, Signals},
        },
        calloop_wayland_source::WaylandSource,
        client::{
            Connection, Proxy, QueueHandle,
            globals::registry_queue_init,
            protocol::{wl_output, wl_surface},
        },
    },
    shell::{
        wlr_layer::{
            Anchor, KeyboardInteractivity, Layer, LayerShell, LayerShellHandler, LayerSurface,
            LayerSurfaceConfigure,
        },
        WaylandSurface,
    },
};
use zbus::{fdo, interface, blocking::Connection as DbusConnection};

use manifest::{MAX_PARAMS, ManifestError};
use shader::DEFAULT_FRAGMENT;
const IDLE_TICK: Duration = Duration::from_millis(250);

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
struct Globals {
    i_time: f32,
    i_time_delta: f32,
    i_frame: u32,
    pad: f32,
    i_resolution: [f32; 4],
    i_params: [[f32; 4]; MAX_PARAMS],
}

enum Command {
    SetWallpaper {
        path: PathBuf,
        reply: std::sync::mpsc::Sender<Result<(), String>>,
    },
    Pause,
    Resume,
    Quit,
}

struct OutputSurface {
    output: wl_output::WlOutput,
    layer: LayerSurface,
    surface: wgpu::Surface<'static>,
    width: u32,
    height: u32,
    configured: bool,
}

struct AppState {
    registry_state: RegistryState,
    output_state: OutputState,
    compositor: CompositorState,
    layer_shell: LayerShell,
    qh: QueueHandle<AppState>,
    loop_signal: LoopSignal,

    instance: wgpu::Instance,
    device: wgpu::Device,
    queue: wgpu::Queue,
    format: wgpu::TextureFormat,
    bind_group_layout: wgpu::BindGroupLayout,
    uniform_buffer: wgpu::Buffer,
    bind_group: wgpu::BindGroup,
    vertex: wgpu::ShaderModule,
    pipeline: wgpu::RenderPipeline,

    surfaces: Vec<OutputSurface>,
    outputs_shared: Arc<RwLock<Vec<String>>>,
    params: [[f32; 4]; MAX_PARAMS],
    start: Instant,
    last_frame: Instant,
    frame: u32,
    paused: bool,
    running: bool,
}

fn output_name(output_state: &OutputState, output: &wl_output::WlOutput) -> String {
    output_state
        .info(output)
        .and_then(|info| info.name.clone())
        .unwrap_or_else(|| format!("output-{}", output.id()))
}

fn output_size(output_state: &OutputState, output: &wl_output::WlOutput) -> (u32, u32) {
    let Some(info) = output_state.info(output) else { return (0, 0) };
    if let Some((w, h)) = info.logical_size {
        return (w.max(0) as u32, h.max(0) as u32);
    }
    info.modes
        .iter()
        .find(|mode| mode.current)
        .or_else(|| info.modes.first())
        .map(|mode| (mode.dimensions.0.max(0) as u32, mode.dimensions.1.max(0) as u32))
        .unwrap_or((0, 0))
}

impl AppState {
    fn add_output(&mut self, conn: &Connection, output: wl_output::WlOutput) {
        if self.surfaces.iter().any(|s| s.output == output) {
            return;
        }
        let (width, height) = output_size(&self.output_state, &output);
        let wl_surface = self.compositor.create_surface(&self.qh);
        let layer = self.layer_shell.create_layer_surface(
            &self.qh,
            wl_surface,
            Layer::Background,
            Some("opaper"),
            Some(&output),
        );
        layer.set_anchor(Anchor::TOP | Anchor::LEFT | Anchor::RIGHT | Anchor::BOTTOM);
        layer.set_keyboard_interactivity(KeyboardInteractivity::None);
        layer.set_size(width, height);
        layer.commit();

        let raw_display = raw_window_handle::RawDisplayHandle::Wayland(
            raw_window_handle::WaylandDisplayHandle::new(
                std::ptr::NonNull::new(conn.backend().display_ptr() as *mut _).unwrap(),
            ),
        );
        let raw_window = raw_window_handle::RawWindowHandle::Wayland(
            raw_window_handle::WaylandWindowHandle::new(
                std::ptr::NonNull::new(layer.wl_surface().id().as_ptr() as *mut _).unwrap(),
            ),
        );
        let surface = unsafe {
            self.instance
                .create_surface_unsafe(wgpu::SurfaceTargetUnsafe::RawHandle {
                    raw_display_handle: Some(raw_display),
                    raw_window_handle: raw_window,
                })
                .expect("create wgpu surface")
        };

        self.surfaces.push(OutputSurface {
            output: output.clone(),
            layer,
            surface,
            width,
            height,
            configured: false,
        });
        self.sync_outputs();
    }

    fn sync_outputs(&self) {
        let names: Vec<String> = self
            .surfaces
            .iter()
            .map(|s| output_name(&self.output_state, &s.output))
            .collect();
        *self.outputs_shared.write().unwrap() = names;
    }

    fn draw(&mut self, wl_surface: &wl_surface::WlSurface) {
        let Some(index) = self
            .surfaces
            .iter()
            .position(|s| s.layer.wl_surface() == wl_surface)
        else { return };
        let item = &self.surfaces[index];
        if !item.configured || item.width == 0 || item.height == 0 {
            return;
        }

        let now = Instant::now();
        let delta = now.duration_since(self.last_frame);
        self.last_frame = now;
        let elapsed = now.duration_since(self.start);
        let globals = Globals {
            i_time: elapsed.as_secs_f32(),
            i_time_delta: delta.as_secs_f32(),
            i_frame: self.frame,
            pad: 0.0,
            i_resolution: [item.width as f32, item.height as f32, 1.0, 0.0],
            i_params: self.params,
        };
        self.queue.write_buffer(&self.uniform_buffer, 0, bytemuck::bytes_of(&globals));
        self.frame = self.frame.wrapping_add(1);

        let texture = match item.surface.get_current_texture() {
            wgpu::CurrentSurfaceTexture::Success(texture)
            | wgpu::CurrentSurfaceTexture::Suboptimal(texture) => texture,
            _ => {
                item.layer.wl_surface().frame(&self.qh, FrameCallbackData(item.layer.wl_surface().clone()));
                return;
            }
        };
        let view = texture.texture.create_view(&wgpu::TextureViewDescriptor::default());
        let mut encoder = self.device.create_command_encoder(&Default::default());
        {
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("opaper"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    depth_slice: None,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
                multiview_mask: None,
            });
            if !self.paused {
                pass.set_pipeline(&self.pipeline);
                pass.set_bind_group(0, &self.bind_group, &[]);
                pass.draw(0..3, 0..1);
            }
        }
        self.queue.submit(Some(encoder.finish()));
        self.queue.present(texture);
        item.layer.wl_surface().frame(&self.qh, FrameCallbackData(item.layer.wl_surface().clone()));
    }

    fn set_wallpaper(&mut self, path: &Path) -> Result<(), String> {
        let (loaded, shader_path) = manifest::load(path).map_err(|e: ManifestError| e.to_string())?;
        let source = std::fs::read_to_string(&shader_path)
            .map_err(|e| format!("cannot read shader: {e}"))?;
        shader::validate_source(&source)?;
        let fragment = self.device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some(&loaded.name),
            source: wgpu::ShaderSource::Glsl {
                shader: source.into(),
                stage: wgpu::naga::ShaderStage::Fragment,
                defines: &[],
            },
        });
        let pipeline = build_pipeline(
            &self.device,
            &self.vertex,
            &fragment,
            &self.bind_group_layout,
            self.format,
        );
        self.pipeline = pipeline;
        self.params = loaded.uniforms().map_err(|e| format!("invalid params: {e}"))?;
        Ok(())
    }
}

fn build_pipeline(
    device: &wgpu::Device,
    vertex: &wgpu::ShaderModule,
    fragment: &wgpu::ShaderModule,
    layout: &wgpu::BindGroupLayout,
    format: wgpu::TextureFormat,
) -> wgpu::RenderPipeline {
    let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: None,
        bind_group_layouts: &[Some(layout)],
        immediate_size: 0,
    });
    device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some("opaper fullscreen shader"),
        layout: Some(&pipeline_layout),
        vertex: wgpu::VertexState {
            module: vertex,
            entry_point: Some("vs_main"),
            compilation_options: Default::default(),
            buffers: &[],
        },
        fragment: Some(wgpu::FragmentState {
            module: fragment,
            entry_point: Some("main"),
            compilation_options: Default::default(),
            targets: &[Some(wgpu::ColorTargetState {
                format,
                blend: None,
                write_mask: wgpu::ColorWrites::ALL,
            })],
        }),
        primitive: wgpu::PrimitiveState::default(),
        depth_stencil: None,
        multisample: wgpu::MultisampleState::default(),
        multiview_mask: None,
        cache: None,
    })
}

struct EngineIface {
    commands: calloop::channel::Sender<Command>,
    outputs: Arc<RwLock<Vec<String>>>,
}

#[interface(name = "org.opaper.Engine")]
impl EngineIface {
    fn set_wallpaper(&self, path: &str) -> fdo::Result<()> {
        let (tx, rx) = reply_channel();
        self.commands
            .send(Command::SetWallpaper { path: PathBuf::from(path), reply: tx })
            .map_err(|_| fdo::Error::Failed("engine is stopping".to_owned()))?;
        rx.recv_timeout(Duration::from_secs(10))
            .map_err(|_| fdo::Error::Timeout("wallpaper load timed out".to_owned()))?
            .map_err(fdo::Error::Failed)
    }

    fn list_outputs(&self) -> Vec<String> {
        self.outputs.read().unwrap().clone()
    }

    fn pause(&self) {
        let _ = self.commands.send(Command::Pause);
    }

    fn resume(&self) {
        let _ = self.commands.send(Command::Resume);
    }

    fn quit(&self) {
        let _ = self.commands.send(Command::Quit);
    }
}

fn handle_command(state: &mut AppState, command: Command) {
    match command {
        Command::SetWallpaper { path, reply } => {
            let result = state.set_wallpaper(&path);
            let _ = reply.send(result);
        }
        Command::Pause => state.paused = true,
        Command::Resume => state.paused = false,
        Command::Quit => {
            state.running = false;
            state.loop_signal.stop();
        }
    }
}

impl CompositorHandler for AppState {
    fn scale_factor_changed(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        _surface: &wl_surface::WlSurface,
        _factor: i32,
    ) {
    }

    fn transform_changed(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        _surface: &wl_surface::WlSurface,
        _transform: wl_output::Transform,
    ) {
    }

    fn frame(&mut self, _conn: &Connection, _qh: &QueueHandle<Self>, surface: &wl_surface::WlSurface, _time: u32) {
        self.draw(surface);
    }

    fn surface_enter(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        _surface: &wl_surface::WlSurface,
        _output: &wl_output::WlOutput,
    ) {
    }

    fn surface_leave(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        _surface: &wl_surface::WlSurface,
        _output: &wl_output::WlOutput,
    ) {
    }
}

impl OutputHandler for AppState {
    fn output_state(&mut self) -> &mut OutputState {
        &mut self.output_state
    }

    fn new_output(&mut self, conn: &Connection, _qh: &QueueHandle<Self>, output: wl_output::WlOutput) {
        self.add_output(conn, output);
    }

    fn update_output(&mut self, _conn: &Connection, _qh: &QueueHandle<Self>, _output: wl_output::WlOutput) {
        self.sync_outputs();
    }

    fn output_destroyed(&mut self, _conn: &Connection, _qh: &QueueHandle<Self>, output: wl_output::WlOutput) {
        self.surfaces.retain(|s| s.output != output);
        self.sync_outputs();
    }
}

impl LayerShellHandler for AppState {
    fn closed(&mut self, _conn: &Connection, _qh: &QueueHandle<Self>, _layer: &LayerSurface) {
        self.running = false;
        self.loop_signal.stop();
    }

    fn configure(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        layer: &LayerSurface,
        configure: LayerSurfaceConfigure,
        _serial: u32,
    ) {
        let Some(item) = self.surfaces.iter_mut().find(|s| &s.layer == layer) else { return };
        let (old_w, old_h) = (item.width, item.height);
        item.width = if configure.new_size.0 == 0 { old_w } else { configure.new_size.0 };
        item.height = if configure.new_size.1 == 0 { old_h } else { configure.new_size.1 };
        if !item.configured || (old_w, old_h) != (item.width, item.height) {
            let surface = item.layer.wl_surface().clone();
            let width = item.width;
            let height = item.height;
            self.configure_surface_at(surface, width, height);
        }
        if !self.paused {
            let surface = layer.wl_surface();
            surface.frame(&self.qh, FrameCallbackData(surface.clone()));
        }
    }
}

impl AppState {
    fn configure_surface_at(&mut self, wl_surface: wl_surface::WlSurface, width: u32, height: u32) {
        let Some(index) = self
            .surfaces
            .iter()
            .position(|s| s.layer.wl_surface() == &wl_surface)
        else { return };
        if width != 0 && height != 0 {
            let config = wgpu::SurfaceConfiguration {
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
                format: self.format,
                view_formats: vec![],
                color_space: wgpu::SurfaceColorSpace::Auto,
                alpha_mode: wgpu::CompositeAlphaMode::Opaque,
                width,
                height,
                present_mode: wgpu::PresentMode::Mailbox,
                desired_maximum_frame_latency: 2,
            };
            self.surfaces[index].surface.configure(&self.device, &config);
            self.surfaces[index].configured = true;
        }
    }
}

impl ProvidesRegistryState for AppState {
    fn registry(&mut self) -> &mut RegistryState {
        &mut self.registry_state
    }
    registry_handlers![OutputState];
}

delegate_registry!(AppState);
smithay_client_toolkit::delegate_dispatch2!(AppState);

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let wallpaper = std::env::args().nth(1).map(PathBuf::from);

    let conn = Connection::connect_to_env()?;
    let (globals, event_queue) = registry_queue_init(&conn)?;
    let qh = event_queue.handle();
    let compositor = CompositorState::bind(&globals, &qh)?;
    let layer_shell = LayerShell::bind(&globals, &qh)?;
    let output_state = OutputState::new(&globals, &qh);

    let instance = wgpu::Instance::new(wgpu::InstanceDescriptor::new_without_display_handle());
    let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions::default()))?;
    let (device, queue) = pollster::block_on(adapter.request_device(&Default::default()))?;
    let format = wgpu::TextureFormat::Bgra8Unorm;
    let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: None,
        entries: &[wgpu::BindGroupLayoutEntry {
            binding: 0,
            visibility: wgpu::ShaderStages::FRAGMENT,
            ty: wgpu::BindingType::Buffer {
                ty: wgpu::BufferBindingType::Uniform,
                has_dynamic_offset: false,
                min_binding_size: None,
            },
            count: None,
        }],
    });
    let uniform_buffer = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("opaper uniforms"),
        size: std::mem::size_of::<Globals>() as u64,
        usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });
    let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: None,
        layout: &bind_group_layout,
        entries: &[wgpu::BindGroupEntry {
            binding: 0,
            resource: uniform_buffer.as_entire_binding(),
        }],
    });
    let vertex = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("fullscreen triangle"),
        source: wgpu::ShaderSource::Wgsl(shader::VERTEX_WGSL.into()),
    });
    let fragment = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("builtin shader"),
        source: wgpu::ShaderSource::Glsl {
            shader: DEFAULT_FRAGMENT.into(),
            stage: wgpu::naga::ShaderStage::Fragment,
            defines: &[],
        },
    });
    let pipeline = build_pipeline(&device, &vertex, &fragment, &bind_group_layout, format);

    let mut event_loop: EventLoop<AppState> = EventLoop::try_new()?;
    let loop_signal = event_loop.get_signal();
    let (commands, command_channel): (calloop::channel::Sender<Command>, Channel<Command>) = channel::channel();
    let outputs_shared = Arc::new(RwLock::new(Vec::new()));

    let mut state = AppState {
        layer_shell,
        registry_state: RegistryState::new(&globals),
        output_state,
        compositor,
        qh: qh.clone(),
        loop_signal: loop_signal.clone(),
        instance,
        device,
        queue,
        format,
        bind_group_layout,
        uniform_buffer,
        bind_group,
        vertex,
        pipeline,
        surfaces: Vec::new(),
        outputs_shared: outputs_shared.clone(),
        params: [[0.0; 4]; MAX_PARAMS],
        start: Instant::now(),
        last_frame: Instant::now(),
        frame: 0,
        paused: false,
        running: true,
    };

    WaylandSource::new(conn.clone(), event_queue).insert(event_loop.handle())?;
    event_loop.handle().insert_source(command_channel, |event, _, state: &mut AppState| {
        if let ChannelEvent::Msg(command) = event {
            handle_command(state, command);
        }
    })?;
    event_loop.handle().insert_source(
        Signals::new(&[Signal::SIGTERM, Signal::SIGINT])?,
        |_, _, state: &mut AppState| {
            state.running = false;
            state.loop_signal.stop();
        },
    )?;

    if let Some(path) = wallpaper {
        state.set_wallpaper(&path)?;
    }
    let outputs: Vec<wl_output::WlOutput> = state.output_state.outputs().collect();
    for output in outputs {
        state.add_output(&conn, output);
    }

    let dbus = DbusConnection::session()?;
    dbus.request_name("org.opaper.Engine")?;
    dbus.object_server().at(
        "/org/opaper/Engine",
        EngineIface { commands: commands.clone(), outputs: outputs_shared },
    )?;

    while state.running {
        event_loop.dispatch(IDLE_TICK, &mut state)?;
    }

    for item in state.surfaces.drain(..) {
        drop(item.surface);
        drop(item.layer);
    }
    Ok(())
}

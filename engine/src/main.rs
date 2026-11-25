mod manifest;
mod shader;

use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::sync::mpsc::channel as reply_channel;
use std::sync::{Arc, RwLock};
use std::time::{Duration, Instant};

mod audio;

use bytemuck::{Pod, Zeroable};
use smithay_client_toolkit::{
    compositor::{CompositorHandler, CompositorState, FrameCallbackData, Region},
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
    i_audio: [f32; 4],
    i_date: [f32; 4],
    i_params: [[f32; 4]; MAX_PARAMS],
}

enum Command {
    SetWallpaper {
        path: PathBuf,
        output: Option<String>,
        reply: std::sync::mpsc::Sender<Result<(), String>>,
    },
    SetParam {
        slot: u8,
        value: [f32; 4],
        output: Option<String>,
    },
    SetFpsLimit(u32),
    GetActiveWallpaper {
        reply: std::sync::mpsc::Sender<String>,
    },
    GetManifest {
        reply: std::sync::mpsc::Sender<String>,
    },
    GetParams {
        reply: std::sync::mpsc::Sender<String>,
    },
    FullscreenScan(String),
    SetPauseOnFullscreen(bool),
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
    pipeline: Option<wgpu::RenderPipeline>,
    params: [[f32; 4]; MAX_PARAMS],
}

struct AppState {
    registry_state: RegistryState,
    output_state: OutputState,
    compositor: CompositorState,
    empty_input_region: Region,
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
    fps_limit: u32,
    paused: bool,
    running: bool,
    plasma_plugin: bool,
    audio_levels: Arc<RwLock<[f32; 4]>>,
    active_wallpaper: Option<PathBuf>,
    active_manifest: Option<String>,
    fullscreen_outputs: HashSet<String>,
    pause_on_fullscreen: bool,
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
        if self.plasma_plugin || self.surfaces.iter().any(|s| s.output == output) {
            return;
        }
        let (width, height) = output_size(&self.output_state, &output);
        let wl_surface = self.compositor.create_surface(&self.qh);
        wl_surface.set_input_region(Some(self.empty_input_region.wl_region()));
        let layer = self.layer_shell.create_layer_surface(
            &self.qh,
            wl_surface,
            Layer::Bottom,
            Some("opaper"),
            Some(&output),
        );
        layer.set_anchor(Anchor::TOP | Anchor::LEFT | Anchor::RIGHT | Anchor::BOTTOM);
        layer.set_keyboard_interactivity(KeyboardInteractivity::None);
        layer.set_exclusive_zone(-1);
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
            pipeline: None,
            params: [[0.0; 4]; MAX_PARAMS],
        });
        self.sync_outputs();
    }

    fn sync_outputs(&self) {
        let names: Vec<String> = if self.plasma_plugin {
            self.output_state
                .outputs()
                .map(|o| output_name(&self.output_state, &o))
                .collect()
        } else {
            self.surfaces
                .iter()
                .map(|s| output_name(&self.output_state, &s.output))
                .collect()
        };
        *self.outputs_shared.write().unwrap() = names;
    }

    fn draw(&mut self, wl_surface: &wl_surface::WlSurface) {
        let Some(index) = self
            .surfaces
            .iter()
            .position(|s| s.layer.wl_surface() == wl_surface)
        else { return };
        let surface_paused = self.paused || (self.pause_on_fullscreen && {
            let oname = output_name(&self.output_state, &self.surfaces[index].output);
            self.fullscreen_outputs.contains(&oname)
        });
        let item = &self.surfaces[index];
        if !item.configured || item.width == 0 || item.height == 0 {
            return;
        }

        item.layer.wl_surface().frame(&self.qh, FrameCallbackData(item.layer.wl_surface().clone()));

        let now = Instant::now();
        if self.fps_limit > 0 {
            let min_frame_time = Duration::from_secs_f64(1.0 / self.fps_limit as f64);
            if now.duration_since(self.last_frame) < min_frame_time {
                return;
            }
        }
        let delta = now.duration_since(self.last_frame);
        self.last_frame = now;
        let elapsed = now.duration_since(self.start);
        let audio = *self.audio_levels.read().unwrap();
        let now_utc = {
            let d = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs();
            let secs_today = (d % 86400) as f32;
            let days = (d / 86400) as i64 + 719468;
            let era = days / 146097;
            let doe = days - era * 146097;
            let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365;
            let y = yoe + era * 400;
            let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
            let mp = (5 * doy + 2) / 153;
            let m = if mp < 10 { mp + 3 } else { mp - 9 };
            let yr = if m <= 2 { y + 1 } else { y };
            let day = doy - (153 * mp + 2) / 5 + 1;
            [yr as f32, m as f32, day as f32, secs_today]
        };
        let globals = Globals {
            i_time: elapsed.as_secs_f32(),
            i_time_delta: delta.as_secs_f32(),
            i_frame: self.frame,
            pad: 0.0,
            i_resolution: [item.width as f32, item.height as f32, 1.0, 0.0],
            i_audio: audio,
            i_date: now_utc,
            i_params: if item.pipeline.is_some() { item.params } else { self.params },
        };
        self.queue.write_buffer(&self.uniform_buffer, 0, bytemuck::bytes_of(&globals));
        self.frame = self.frame.wrapping_add(1);

        let texture = match item.surface.get_current_texture() {
            wgpu::CurrentSurfaceTexture::Success(texture)
            | wgpu::CurrentSurfaceTexture::Suboptimal(texture) => texture,
            _ => return,
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
            if !surface_paused {
                pass.set_pipeline(item.pipeline.as_ref().unwrap_or(&self.pipeline));
                pass.set_bind_group(0, &self.bind_group, &[]);
                pass.draw(0..3, 0..1);
            }
        }
        self.queue.submit(Some(encoder.finish()));
        self.queue.present(texture);
    }

    fn set_wallpaper(&mut self, path: &Path, output: Option<&str>) -> Result<(), String> {
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
        let params = loaded.uniforms().map_err(|e| format!("invalid params: {e}"))?;

        let manifest_json = std::fs::read_to_string(path.join("manifest.json")).unwrap_or_default();
        self.active_wallpaper = Some(path.to_path_buf());
        self.active_manifest = Some(manifest_json);

        if self.plasma_plugin {
            self.pipeline = pipeline;
            self.params = params;
            if plasma::is_running() && plasma::plugin_installed() {
                let shader_name = path.file_name().unwrap_or_default().to_string_lossy();
                let abs = std::fs::canonicalize(path)
                    .unwrap_or_else(|_| path.to_path_buf())
                    .to_string_lossy()
                    .to_string();
                match output {
                    Some(name) => plasma::activate_opaper_on_output(&shader_name, &abs, &params, name),
                    None => plasma::activate_opaper(&shader_name, &abs, &params),
                }
            }
        } else if let Some(name) = output {
            let available: Vec<String> = self
                .surfaces
                .iter()
                .map(|surface| output_name(&self.output_state, &surface.output))
                .collect();
            let surface = self
                .surfaces
                .iter_mut()
                .find(|surface| output_name(&self.output_state, &surface.output) == name)
                .ok_or_else(|| format!("output not found: {name} (available: {})", available.join(", ")))?;
            surface.pipeline = Some(pipeline);
            surface.params = params;
        } else {
            for surface in &mut self.surfaces {
                surface.pipeline = Some(pipeline.clone());
                surface.params = params;
            }
            self.pipeline = pipeline;
            self.params = params;
        }
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
        self.set_wallpaper_on_output(path, "")
    }

    fn set_wallpaper_on_output(&self, path: &str, output: &str) -> fdo::Result<()> {
        let (tx, rx) = reply_channel();
        self.commands
            .send(Command::SetWallpaper {
                path: PathBuf::from(path),
                output: if output.is_empty() { None } else { Some(output.to_owned()) },
                reply: tx,
            })
            .map_err(|_| fdo::Error::Failed("engine is stopping".to_owned()))?;
        rx.recv_timeout(Duration::from_secs(10))
            .map_err(|_| fdo::Error::Timeout("wallpaper load timed out".to_owned()))?
            .map_err(fdo::Error::Failed)
    }

    fn list_outputs(&self) -> Vec<String> {
        self.outputs.read().unwrap().clone()
    }

    fn set_param(&self, slot: u8, r: f64, g: f64, b: f64, a: f64, output: &str) -> fdo::Result<()> {
        if slot as usize >= MAX_PARAMS {
            return Err(fdo::Error::InvalidArgs(format!("slot must be 0..{}", MAX_PARAMS - 1)));
        }
        self.commands
            .send(Command::SetParam {
                slot,
                value: [r as f32, g as f32, b as f32, a as f32],
                output: if output.is_empty() { None } else { Some(output.to_owned()) },
            })
            .map_err(|_| fdo::Error::Failed("engine is stopping".to_owned()))
    }

    fn set_fps_limit(&self, fps: u32) -> fdo::Result<()> {
        self.commands
            .send(Command::SetFpsLimit(fps))
            .map_err(|_| fdo::Error::Failed("engine is stopping".to_owned()))
    }

    fn get_active_wallpaper(&self) -> fdo::Result<String> {
        let (tx, rx) = reply_channel();
        self.commands
            .send(Command::GetActiveWallpaper { reply: tx })
            .map_err(|_| fdo::Error::Failed("engine is stopping".to_owned()))?;
        rx.recv_timeout(Duration::from_secs(2))
            .map_err(|_| fdo::Error::Timeout("timed out".to_owned()))
    }

    fn get_manifest(&self) -> fdo::Result<String> {
        let (tx, rx) = reply_channel();
        self.commands
            .send(Command::GetManifest { reply: tx })
            .map_err(|_| fdo::Error::Failed("engine is stopping".to_owned()))?;
        rx.recv_timeout(Duration::from_secs(2))
            .map_err(|_| fdo::Error::Timeout("timed out".to_owned()))
    }

    fn get_params(&self) -> fdo::Result<String> {
        let (tx, rx) = reply_channel();
        self.commands
            .send(Command::GetParams { reply: tx })
            .map_err(|_| fdo::Error::Failed("engine is stopping".to_owned()))?;
        rx.recv_timeout(Duration::from_secs(2))
            .map_err(|_| fdo::Error::Timeout("timed out".to_owned()))
    }

    fn fullscreen_scan(&self, outputs: &str) {
        let _ = self.commands.send(Command::FullscreenScan(outputs.to_owned()));
    }

    fn set_pause_on_fullscreen(&self, enabled: bool) {
        let _ = self.commands.send(Command::SetPauseOnFullscreen(enabled));
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
        Command::SetWallpaper { path, output, reply } => {
            let result = state.set_wallpaper(&path, output.as_deref());
            let _ = reply.send(result);
        }
        Command::SetParam { slot, value, output } => {
            let idx = slot as usize;
            state.params[idx] = value;
            if state.plasma_plugin {
                match output {
                    Some(ref name) => plasma::update_params_on_output(&state.params, name),
                    None => plasma::update_params(&state.params),
                }
            } else {
                match output {
                    Some(name) => {
                        if let Some(surface) = state.surfaces.iter_mut().find(|s| {
                            output_name(&state.output_state, &s.output) == name
                        }) {
                            surface.params[idx] = value;
                        }
                    }
                    None => {
                        for surface in &mut state.surfaces {
                            surface.params[idx] = value;
                        }
                    }
                }
            }
        }
        Command::SetFpsLimit(fps) => state.fps_limit = fps,
        Command::GetActiveWallpaper { reply } => {
            let path = state
                .active_wallpaper
                .as_ref()
                .map(|p| p.to_string_lossy().to_string())
                .unwrap_or_default();
            let _ = reply.send(path);
        }
        Command::GetManifest { reply } => {
            let manifest = state.active_manifest.clone().unwrap_or_default();
            let _ = reply.send(manifest);
        }
        Command::GetParams { reply } => {
            let params_json = serde_json::to_string(&state.params).unwrap_or_default();
            let _ = reply.send(params_json);
        }
        Command::FullscreenScan(outputs_str) => {
            let new_fs: HashSet<String> = outputs_str
                .split(',')
                .filter(|s| !s.is_empty())
                .map(|s| s.to_owned())
                .collect();
            if state.pause_on_fullscreen && state.plasma_plugin {
                for name in new_fs.difference(&state.fullscreen_outputs) {
                    plasma::set_paused_on_output(name, true);
                }
                for name in state.fullscreen_outputs.difference(&new_fs) {
                    plasma::set_paused_on_output(name, false);
                }
            }
            state.fullscreen_outputs = new_fs;
        }
        Command::SetPauseOnFullscreen(enabled) => {
            if !enabled && state.plasma_plugin {
                for name in &state.fullscreen_outputs {
                    plasma::set_paused_on_output(name, false);
                }
            }
            state.pause_on_fullscreen = enabled;
            if enabled && state.plasma_plugin {
                for name in &state.fullscreen_outputs {
                    plasma::set_paused_on_output(name, true);
                }
            }
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
            self.draw(layer.wl_surface());
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

mod plasma {
    use std::process::Command as Cmd;

    fn eval(script: &str) -> Option<String> {
        let output = Cmd::new("qdbus6")
            .args([
                "org.kde.plasmashell",
                "/PlasmaShell",
                "org.kde.PlasmaShell.evaluateScript",
                script,
            ])
            .output()
            .ok()?;
        let text = String::from_utf8_lossy(&output.stdout).trim().to_string();
        if output.status.success() && !text.is_empty() { Some(text) } else { None }
    }

    pub fn is_running() -> bool {
        Cmd::new("qdbus6")
            .args(["org.kde.plasmashell", "/PlasmaShell"])
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false)
    }

    pub fn get_wallpaper_plugin() -> Option<String> {
        eval("var d = desktops(); if (d.length > 0) print(d[0].wallpaperPlugin);")
    }

    pub fn set_wallpaper_plugin(plugin: &str) {
        let script = format!(
            "var d = desktops(); \
             for (var i = 0; i < d.length; i++) {{ \
                 d[i].wallpaperPlugin = '{plugin}'; \
             }}"
        );
        eval(&script);
    }

    fn params_json(params: &[[f32; 4]; 8]) -> String {
        format!(
            "[[{},{},{},{}],[{},{},{},{}],[{},{},{},{}],[{},{},{},{}],[{},{},{},{}],[{},{},{},{}],[{},{},{},{}],[{},{},{},{}]]",
            params[0][0], params[0][1], params[0][2], params[0][3],
            params[1][0], params[1][1], params[1][2], params[1][3],
            params[2][0], params[2][1], params[2][2], params[2][3],
            params[3][0], params[3][1], params[3][2], params[3][3],
            params[4][0], params[4][1], params[4][2], params[4][3],
            params[5][0], params[5][1], params[5][2], params[5][3],
            params[6][0], params[6][1], params[6][2], params[6][3],
            params[7][0], params[7][1], params[7][2], params[7][3],
        )
    }

    pub fn activate_opaper(shader_name: &str, wallpaper_path: &str, params: &[[f32; 4]; 8]) {
        let pj = params_json(params);
        let script = format!(
            "var d = desktops(); \
             for (var i = 0; i < d.length; i++) {{ \
                 d[i].wallpaperPlugin = 'org.opaper.wallpaper'; \
                 d[i].currentConfigGroup = ['Wallpaper', 'org.opaper.wallpaper', 'General']; \
                 d[i].writeConfig('ShaderName', '{shader_name}'); \
                 d[i].writeConfig('WallpaperPath', '{wallpaper_path}'); \
                 d[i].writeConfig('Params', '{pj}'); \
             }}"
        );
        eval(&script);
    }

    pub fn update_params(params: &[[f32; 4]; 8]) {
        let pj = params_json(params);
        let script = format!(
            "var d = desktops(); \
             for (var i = 0; i < d.length; i++) {{ \
                 d[i].currentConfigGroup = ['Wallpaper', 'org.opaper.wallpaper', 'General']; \
                 d[i].writeConfig('Params', '{pj}'); \
             }}"
        );
        eval(&script);
    }

    pub fn activate_opaper_on_output(shader_name: &str, wallpaper_path: &str, params: &[[f32; 4]; 8], output_name: &str) {
        let Some(&screen_idx) = get_output_screen_map().get(output_name) else { return };
        let pj = params_json(params);
        let script = format!(
            "var d = desktops(); \
             for (var i = 0; i < d.length; i++) {{ \
                 if (d[i].screen == {screen_idx}) {{ \
                     d[i].wallpaperPlugin = 'org.opaper.wallpaper'; \
                     d[i].currentConfigGroup = ['Wallpaper', 'org.opaper.wallpaper', 'General']; \
                     d[i].writeConfig('ShaderName', '{shader_name}'); \
                     d[i].writeConfig('WallpaperPath', '{wallpaper_path}'); \
                     d[i].writeConfig('Params', '{pj}'); \
                 }} \
             }}"
        );
        eval(&script);
    }

    pub fn update_params_on_output(params: &[[f32; 4]; 8], output_name: &str) {
        let Some(&screen_idx) = get_output_screen_map().get(output_name) else { return };
        let pj = params_json(params);
        let script = format!(
            "var d = desktops(); \
             for (var i = 0; i < d.length; i++) {{ \
                 if (d[i].screen == {screen_idx}) {{ \
                     d[i].currentConfigGroup = ['Wallpaper', 'org.opaper.wallpaper', 'General']; \
                     d[i].writeConfig('Params', '{pj}'); \
                 }} \
             }}"
        );
        eval(&script);
    }

    fn get_output_screen_map() -> std::collections::HashMap<String, u32> {
        let mut map = std::collections::HashMap::new();
        let Ok(output) = Cmd::new("qdbus6")
            .args(["org.kde.KWin", "/KWin", "org.kde.KWin.supportInformation"])
            .output()
        else { return map };
        let text = String::from_utf8_lossy(&output.stdout);
        let mut current_screen: Option<u32> = None;
        for line in text.lines() {
            let trimmed = line.trim();
            if trimmed.starts_with("Screen ") && trimmed.ends_with(':') {
                current_screen = trimmed[7..trimmed.len() - 1].parse().ok();
            } else if let Some(name) = trimmed.strip_prefix("Name: ") {
                if let Some(idx) = current_screen.take() {
                    map.insert(name.to_string(), idx);
                }
            }
        }
        map
    }

    pub fn set_paused_on_output(output_name: &str, paused: bool) {
        let Some(&screen_idx) = get_output_screen_map().get(output_name) else { return };
        let val = if paused { "true" } else { "false" };
        let script = format!(
            "var d = desktops(); \
             for (var i = 0; i < d.length; i++) {{ \
                 if (d[i].screen == {screen_idx}) {{ \
                     d[i].currentConfigGroup = ['Wallpaper', 'org.opaper.wallpaper', 'General']; \
                     d[i].writeConfig('Paused', '{val}'); \
                 }} \
             }}"
        );
        eval(&script);
    }

    pub fn plugin_installed() -> bool {
        let home = std::env::var("HOME").unwrap_or_default();
        std::path::Path::new(&format!(
            "{home}/.local/share/plasma/wallpapers/org.opaper.wallpaper/metadata.json"
        ))
        .exists()
            || std::path::Path::new(
                "/usr/share/plasma/wallpapers/org.opaper.wallpaper/metadata.json",
            )
            .exists()
    }

    pub fn push_audio_levels(levels: &[f32; 4]) {
        let val = format!("{:.3} {:.3} {:.3} {:.3}", levels[0], levels[1], levels[2], levels[3]);
        eval(&format!(
            "var d = desktops(); \
             for (var i = 0; i < d.length; i++) {{ \
                 d[i].currentConfigGroup = ['Wallpaper', 'org.opaper.wallpaper', 'General']; \
                 d[i].writeConfig('AudioLevels', '{val}'); \
             }}"
        ));
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let wallpaper = std::env::args().nth(1).map(PathBuf::from);

    let conn = Connection::connect_to_env()?;
    let (globals, event_queue) = registry_queue_init(&conn)?;
    let qh = event_queue.handle();
    let compositor = CompositorState::bind(&globals, &qh)?;
    let layer_shell = LayerShell::bind(&globals, &qh)?;
    let output_state = OutputState::new(&globals, &qh);
    let empty_input_region = Region::new(&compositor)?;

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
    let audio_levels = Arc::new(RwLock::new([0.0f32; 4]));
    audio::start_capture(audio_levels.clone());

    let mut state = AppState {
        layer_shell,
        registry_state: RegistryState::new(&globals),
        output_state,
        compositor,
        empty_input_region,
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
        fps_limit: 60,
        paused: false,
        running: true,
        plasma_plugin: false,
        audio_levels: audio_levels.clone(),
        active_wallpaper: None,
        active_manifest: None,
        fullscreen_outputs: HashSet::new(),
        pause_on_fullscreen: false,
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

    let plasma_running = plasma::is_running();
    state.plasma_plugin = plasma_running;
    if plasma_running && !plasma::plugin_installed() {
        eprintln!("warning: KDE Plasma detected but opaper plugin not installed — run 'make kde-plugin'");
    }
    let plasma_original = if plasma_running {
        plasma::get_wallpaper_plugin()
    } else {
        None
    };

    if let Some(path) = wallpaper {
        state.set_wallpaper(&path, None)?;
    }
    if plasma_running {
        state.sync_outputs();
    } else {
        let outputs: Vec<wl_output::WlOutput> = state.output_state.outputs().collect();
        for output in outputs {
            state.add_output(&conn, output);
        }
    }

    let dbus = DbusConnection::session()?;
    dbus.request_name("org.opaper.Engine")?;
    dbus.object_server().at(
        "/org/opaper/Engine",
        EngineIface { commands: commands.clone(), outputs: outputs_shared },
    )?;

    if state.plasma_plugin {
        let audio = state.audio_levels.clone();
        std::thread::spawn(move || {
            let mut was_active = false;
            loop {
                std::thread::sleep(Duration::from_millis(66));
                let levels = *audio.read().unwrap();
                let active = levels[0] > 0.001;
                if active || was_active {
                    plasma::push_audio_levels(&levels);
                }
                was_active = active;
            }
        });
    }

    while state.running {
        event_loop.dispatch(IDLE_TICK, &mut state)?;
        state.sync_outputs();
    }

    for item in state.surfaces.drain(..) {
        drop(item.surface);
        drop(item.layer);
    }

    if let Some(ref plugin) = plasma_original {
        plasma::set_wallpaper_plugin(plugin);
    }

    Ok(())
}

use std::io::Read;
use std::process::{Child, Command, Stdio};
use std::sync::{Arc, RwLock};

pub fn start_capture(levels: Arc<RwLock<[f32; 4]>>) {
    std::thread::spawn(move || {
        loop {
            if let Err(e) = capture_loop(&levels) {
                eprintln!("audio: {e}, retrying in 2s...");
                *levels.write().unwrap() = [0.0; 4];
                std::thread::sleep(std::time::Duration::from_secs(2));
            }
        }
    });
}

fn get_default_sink() -> Option<String> {
    let out = Command::new("pactl")
        .args(["get-default-sink"])
        .output()
        .ok()?;
    let s = String::from_utf8(out.stdout).ok()?.trim().to_string();
    if s.is_empty() { None } else { Some(s) }
}

fn get_monitor_source_id(sink: &str) -> Option<String> {
    let monitor = format!("{sink}.monitor");
    let out = Command::new("pactl")
        .args(["list", "short", "sources"])
        .output()
        .ok()?;
    let text = String::from_utf8(out.stdout).ok()?;
    for line in text.lines() {
        if line.contains(&monitor) {
            return line.split_whitespace().next().map(|s| s.to_string());
        }
    }
    None
}

fn get_system_volume() -> f32 {
    Command::new("wpctl")
        .args(["get-volume", "@DEFAULT_AUDIO_SINK@"])
        .output()
        .ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .and_then(|t| t.split_whitespace().nth(1)?.parse::<f32>().ok())
        .unwrap_or(1.0)
}

fn start_pw_cat(source_id: &str) -> Result<Child, String> {
    Command::new("pw-cat")
        .args([
            "--record", "--format", "f32",
            "--rate", "44100", "--channels", "1",
            "--target", source_id, "-",
        ])
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .map_err(|e| format!("pw-cat not found: {e}"))
}

fn skip_au_header(stdout: &mut impl Read) -> Result<(), Box<dyn std::error::Error>> {
    let mut header = [0u8; 24];
    stdout.read_exact(&mut header)?;
    let offset = u32::from_be_bytes([header[4], header[5], header[6], header[7]]) as usize;
    if offset > 24 {
        let mut skip = vec![0u8; offset - 24];
        stdout.read_exact(&mut skip)?;
    }
    Ok(())
}

fn capture_loop(levels: &Arc<RwLock<[f32; 4]>>) -> Result<(), Box<dyn std::error::Error>> {
    let sink = get_default_sink().ok_or("no default sink")?;
    let source_id = get_monitor_source_id(&sink).ok_or("no monitor source")?;

    eprintln!("audio: sink={sink} monitor={source_id}");

    let mut child = start_pw_cat(&source_id)?;
    let mut stdout = child.stdout.take().unwrap();
    skip_au_header(&mut stdout)?;

    eprintln!("audio: stream started");

    let samples_per_tick = 1470;
    let mut raw = vec![0u8; samples_per_tick * 4];
    let mut tick = 0u32;
    let mut volume = get_system_volume();
    let mut smooth = [0.0f32; 4];
    const ATTACK: f32 = 0.12;
    const RELEASE: f32 = 0.03;

    loop {
        if stdout.read_exact(&mut raw).is_err() {
            let _ = child.kill();
            return Err("stream ended".into());
        }

        tick += 1;

        if tick % 15 == 0 {
            volume = get_system_volume();
            if let Some(new_sink) = get_default_sink() {
                if new_sink != sink {
                    eprintln!("audio: sink changed to {new_sink}, reconnecting...");
                    let _ = child.kill();
                    return Err("sink changed".into());
                }
            }
        }

        let samples: Vec<f32> = raw
            .chunks_exact(4)
            .map(|b| f32::from_be_bytes([b[0], b[1], b[2], b[3]]))
            .collect();

        let mut analysis = analyze(&samples);
        for v in &mut analysis {
            *v *= volume;
        }

        for i in 0..4 {
            let alpha = if analysis[i] > smooth[i] { ATTACK } else { RELEASE };
            smooth[i] += alpha * (analysis[i] - smooth[i]);
        }

        *levels.write().unwrap() = smooth;
        let _ = std::fs::write(
            "/tmp/opaper-audio",
            format!("{} {} {} {}", smooth[0], smooth[1], smooth[2], smooth[3]),
        );
    }
}

const NOISE_GATE: f32 = 0.008;

fn analyze(samples: &[f32]) -> [f32; 4] {
    let n = samples.len();
    if n == 0 {
        return [0.0; 4];
    }

    let rms: f32 = (samples.iter().map(|s| s * s).sum::<f32>() / n as f32).sqrt();

    if rms < NOISE_GATE {
        return [0.0; 4];
    }

    let level = (rms * 4.0).min(1.0);

    let mut lp = 0.0f32;
    let alpha_bass = 0.02;
    let mut bass_sum = 0.0f32;
    let mut treble_sum = 0.0f32;
    let mut prev = 0.0f32;

    for &s in samples {
        lp += alpha_bass * (s - lp);
        bass_sum += lp * lp;
        let hp = s - prev;
        treble_sum += hp * hp;
        prev = s;
    }

    let bass_rms = (bass_sum / n as f32).sqrt();
    let treble_rms = (treble_sum / n as f32).sqrt();
    let mid_rms = (rms * rms - bass_rms * bass_rms - treble_rms * treble_rms)
        .max(0.0)
        .sqrt();

    let bass = (bass_rms * 10.0).min(1.0);
    let mid = (mid_rms * 5.0).min(1.0);
    let treble = (treble_rms * 4.0).min(1.0);

    [level, bass, mid, treble]
}

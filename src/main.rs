//! strudel-rs — Strudel notation live-performance CLI (scaffold Tasks 1–8).

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};

fn main() {
    // Task 1 smoke: 440 Hz sine for 2 seconds when an output device is available.
    // Headless / missing device: print a message and exit successfully so CI can still build.
    if let Err(e) = play_sine_smoke() {
        eprintln!("strudel-rs: audio smoke skipped: {e}");
        eprintln!("Library modules: backend, transport, mini, code, synth, sound.");
        eprintln!("Run `cargo test` for headless verification (NullBackend).");
    }
}

fn play_sine_smoke() -> Result<(), String> {
    let host = cpal::default_host();
    let device = host
        .default_output_device()
        .ok_or_else(|| "no default output device".to_string())?;
    let config = device
        .default_output_config()
        .map_err(|e| format!("output config: {e}"))?;
    let sr = config.sample_rate().0 as f32;
    let mut phase = 0f32;
    let stream = device
        .build_output_stream(
            &config.into(),
            move |data: &mut [f32], _| {
                for s in data.iter_mut() {
                    *s = (phase * 2.0 * std::f32::consts::PI).sin() * 0.2;
                    phase = (phase + 440.0 / sr) % 1.0;
                }
            },
            |e| eprintln!("stream error: {e}"),
            None,
        )
        .map_err(|e| format!("build stream: {e}"))?;
    stream.play().map_err(|e| format!("play: {e}"))?;
    std::thread::sleep(std::time::Duration::from_secs(2));
    Ok(())
}

//! strudel-rs — Strudel notation live-performance CLI (Tasks 1–12 foundation).

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};

fn main() {
    // Device smoke: 440 Hz sine for 2 seconds when an output device is available.
    // Headless / missing device: print a message and exit successfully so CI can still build.
    if let Err(e) = play_sine_smoke() {
        eprintln!("strudel-rs: audio smoke skipped: {e}");
        eprintln!(
            "Library: backend, transport, mini, code, synth, sound, sample, song, deck, engine."
        );
        eprintln!("Run `cargo test` for headless verification (NullBackend path).");
        eprintln!("Demo song: songs/smoke.strudel — samples/ is CC0 (Sonic Pi sourced).");
    }
}

fn play_sine_smoke() -> Result<(), String> {
    let host = cpal::default_host();
    let device = host
        .default_output_device()
        .ok_or_else(|| "no default output device".to_string())?;
    // cpal 0.18+: sample_rate() is u32; build_output_stream takes StreamConfig by value.
    let config = device
        .default_output_config()
        .map_err(|e| format!("output config: {e}"))?;
    let sr = config.sample_rate() as f32;
    let stream_config: cpal::StreamConfig = config.into();
    let mut phase = 0f32;
    let stream = device
        .build_output_stream(
            stream_config,
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

//! Live TUI voice input: cpal capture → local STT HTTP → text for Hermes.
//!
//! Runs off the audio output and UI threads. Transcripts are delivered as
//! events for the live UI to `HermesHandle::enqueue`.
//!
//! STT is OpenAI-compatible `POST {base}/v1/audio/transcriptions` only
//! (the exhibit box behind `STRUDEL_STT_BASE_URL`, not a cloud vendor).

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{FromSample, Sample, SampleFormat, SizedSample, StreamConfig};
use crossbeam::channel::{unbounded, Receiver, Sender, TryRecvError};
use reqwest::blocking::multipart;
use serde_json::Value;

const DEFAULT_LANGUAGE: &str = "ja";
const DEFAULT_MODEL: &str = "whisper-1";
const DEFAULT_TIMEOUT_SECS: u64 = 30;
const DEFAULT_MAX_RECORDING_SECS: f32 = 7.0;
const DEFAULT_MIN_RECORDING_SECS: f32 = 0.4;
const DEFAULT_SILENCE_THRESHOLD: f32 = 500.0;
const DEFAULT_SILENCE_SECS: f32 = 1.2;
const VAD_START_MS: f32 = 80.0;
const VAD_PREROLL_SECS: f32 = 0.2;
const VAD_WINDOW_SECS: f32 = 0.03;
const TARGET_STT_RATE: u32 = 16_000;
const WORKER_TICK: Duration = Duration::from_millis(40);

/// How recording starts.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VoiceMode {
    /// F12 starts and stops capture (exhibit default).
    Push,
    /// Keep the input stream open and split utterances with RMS.
    Vad,
}

impl VoiceMode {
    pub fn parse(s: &str) -> Self {
        match s.trim().to_ascii_lowercase().as_str() {
            "vad" | "listen" | "auto" => Self::Vad,
            _ => Self::Push,
        }
    }
}

/// STT + capture settings (env-driven).
#[derive(Debug, Clone)]
pub struct SttConfig {
    /// Optional bearer for the local STT server. Empty = no Authorization header.
    pub api_key: String,
    /// Base URL of the local STT server (no trailing slash).
    pub base_url: String,
    pub language: String,
    pub model: String,
    pub timeout: Duration,
    pub max_recording_secs: f32,
    pub min_recording_secs: f32,
    pub mode: VoiceMode,
    /// RMS of i16 samples. Above this counts as voice.
    pub silence_threshold: f32,
    /// Seconds below threshold before an utterance ends (VAD).
    pub silence_secs: f32,
}

impl SttConfig {
    /// Build from env. Returns `None` when `STRUDEL_STT_BASE_URL` is missing.
    pub fn from_env() -> Option<Self> {
        let base_url = env_nonempty("STRUDEL_STT_BASE_URL")?;
        Some(Self::from_parts(
            base_url,
            env_nonempty("STRUDEL_STT_API_KEY").unwrap_or_default(),
            env_nonempty("STRUDEL_STT_LANGUAGE").unwrap_or_else(|| DEFAULT_LANGUAGE.to_string()),
            env_nonempty("STRUDEL_STT_MODEL").unwrap_or_else(|| DEFAULT_MODEL.to_string()),
            env_nonempty("STRUDEL_STT_TIMEOUT_SECS")
                .and_then(|s| s.parse::<u64>().ok())
                .filter(|&n| n > 0)
                .map(Duration::from_secs)
                .unwrap_or(Duration::from_secs(DEFAULT_TIMEOUT_SECS)),
            env_nonempty("STRUDEL_VOICE_MAX_SECS")
                .and_then(|s| s.parse::<f32>().ok())
                .filter(|&n| n > 0.0)
                .unwrap_or(DEFAULT_MAX_RECORDING_SECS),
            VoiceMode::parse(&env_nonempty("STRUDEL_VOICE_MODE").unwrap_or_default()),
            env_nonempty("STRUDEL_VOICE_SILENCE_THRESHOLD")
                .and_then(|s| s.parse::<f32>().ok())
                .filter(|&n| n > 0.0)
                .unwrap_or(DEFAULT_SILENCE_THRESHOLD),
            env_nonempty("STRUDEL_VOICE_SILENCE_SECS")
                .and_then(|s| s.parse::<f32>().ok())
                .filter(|&n| n > 0.0)
                .unwrap_or(DEFAULT_SILENCE_SECS),
        ))
    }

    #[allow(clippy::too_many_arguments)]
    pub fn from_parts(
        base_url: String,
        api_key: String,
        language: String,
        model: String,
        timeout: Duration,
        max_recording_secs: f32,
        mode: VoiceMode,
        silence_threshold: f32,
        silence_secs: f32,
    ) -> Self {
        Self {
            api_key,
            base_url: base_url.trim_end_matches('/').to_string(),
            language,
            model,
            timeout,
            max_recording_secs,
            min_recording_secs: DEFAULT_MIN_RECORDING_SECS,
            mode,
            silence_threshold,
            silence_secs,
        }
    }
}

/// `true` when a non-empty STT base URL is set (cloud keys do not enable voice).
pub fn enable_voice(base_url: Option<&str>) -> bool {
    base_url.map(|s| !s.trim().is_empty()).unwrap_or(false)
}

pub fn transcription_url(base_url: &str) -> String {
    format!("{}/v1/audio/transcriptions", base_url.trim_end_matches('/'))
}

/// Commands from the live UI to the voice worker.
#[derive(Debug)]
enum VoiceCmd {
    Toggle,
    Shutdown,
}

/// Events from the voice worker to the live UI.
#[derive(Debug, Clone)]
pub enum VoiceEvent {
    /// VAD armed; waiting for speech.
    ListeningStarted,
    /// VAD paused via F12.
    ListeningPaused,
    RecordingStarted,
    /// Captured duration in seconds (for logs).
    RecordingStopped {
        secs: f32,
    },
    SttRunning,
    Transcript {
        text: String,
    },
    Failed {
        message: String,
    },
}

/// Background voice worker (serial: one capture → STT at a time).
pub struct VoiceHandle {
    cmd_tx: Sender<VoiceCmd>,
    event_rx: Receiver<VoiceEvent>,
    recording: Arc<AtomicBool>,
    listening: Arc<AtomicBool>,
    stt_busy: Arc<AtomicBool>,
    mode: VoiceMode,
}

impl VoiceHandle {
    /// Spawn the worker. Probes default input device first.
    pub fn start(config: SttConfig) -> Result<Self, String> {
        let host = cpal::default_host();
        let _device = host
            .default_input_device()
            .ok_or_else(|| "no default input device".to_string())?;

        let (cmd_tx, cmd_rx) = unbounded::<VoiceCmd>();
        let (event_tx, event_rx) = unbounded::<VoiceEvent>();
        let recording = Arc::new(AtomicBool::new(false));
        let listening = Arc::new(AtomicBool::new(false));
        let stt_busy = Arc::new(AtomicBool::new(false));
        let rec_flag = Arc::clone(&recording);
        let listen_flag = Arc::clone(&listening);
        let stt_flag = Arc::clone(&stt_busy);
        let mode = config.mode;

        thread::Builder::new()
            .name("voice-worker".into())
            .spawn(move || worker_loop(config, cmd_rx, event_tx, rec_flag, listen_flag, stt_flag))
            .map_err(|e| format!("spawn voice-worker: {e}"))?;

        Ok(Self {
            cmd_tx,
            event_rx,
            recording,
            listening,
            stt_busy,
            mode,
        })
    }

    /// Toggle recording (push) or pause/resume listening (VAD).
    pub fn toggle(&self) {
        let _ = self.cmd_tx.send(VoiceCmd::Toggle);
    }

    pub fn mode(&self) -> VoiceMode {
        self.mode
    }

    pub fn is_recording(&self) -> bool {
        self.recording.load(Ordering::Relaxed)
    }

    pub fn is_listening(&self) -> bool {
        self.listening.load(Ordering::Relaxed)
    }

    pub fn is_stt_busy(&self) -> bool {
        self.stt_busy.load(Ordering::Relaxed)
    }

    pub fn try_recv(&self) -> Option<VoiceEvent> {
        match self.event_rx.try_recv() {
            Ok(e) => Some(e),
            Err(TryRecvError::Empty) | Err(TryRecvError::Disconnected) => None,
        }
    }

    pub fn drain_events(&self) -> Vec<VoiceEvent> {
        let mut out = Vec::new();
        while let Some(e) = self.try_recv() {
            out.push(e);
        }
        out
    }
}

impl Drop for VoiceHandle {
    fn drop(&mut self) {
        let _ = self.cmd_tx.send(VoiceCmd::Shutdown);
    }
}

fn worker_loop(
    config: SttConfig,
    cmd_rx: Receiver<VoiceCmd>,
    event_tx: Sender<VoiceEvent>,
    recording_flag: Arc<AtomicBool>,
    listening_flag: Arc<AtomicBool>,
    stt_flag: Arc<AtomicBool>,
) {
    match config.mode {
        VoiceMode::Push => worker_push(config, cmd_rx, event_tx, recording_flag, stt_flag),
        VoiceMode::Vad => worker_vad(
            config,
            cmd_rx,
            event_tx,
            recording_flag,
            listening_flag,
            stt_flag,
        ),
    }
}

fn worker_push(
    config: SttConfig,
    cmd_rx: Receiver<VoiceCmd>,
    event_tx: Sender<VoiceEvent>,
    recording_flag: Arc<AtomicBool>,
    stt_flag: Arc<AtomicBool>,
) {
    let mut active: Option<MicBuffer> = None;

    loop {
        if active.is_some() {
            let should_stop = match cmd_rx.recv_timeout(WORKER_TICK) {
                Ok(VoiceCmd::Shutdown) => {
                    let _ = stop_capture(&mut active, &recording_flag);
                    break;
                }
                Ok(VoiceCmd::Toggle) => true,
                Err(crossbeam::channel::RecvTimeoutError::Timeout) => active
                    .as_ref()
                    .map(|c| c.elapsed() >= config.max_recording_secs)
                    .unwrap_or(false),
                Err(crossbeam::channel::RecvTimeoutError::Disconnected) => break,
            };
            if !should_stop {
                continue;
            }
            let secs = active.as_ref().map(|c| c.elapsed()).unwrap_or(0.0);
            let captured = stop_capture(&mut active, &recording_flag);
            let _ = event_tx.send(VoiceEvent::RecordingStopped { secs });
            process_capture_result(&config, captured, secs, &event_tx, &stt_flag);
            continue;
        }

        match cmd_rx.recv() {
            Ok(VoiceCmd::Shutdown) | Err(_) => break,
            Ok(VoiceCmd::Toggle) => {
                if stt_flag.load(Ordering::Relaxed) {
                    let _ = event_tx.send(VoiceEvent::Failed {
                        message: "STT 処理中…少し待ってね".into(),
                    });
                    continue;
                }
                match start_mic(&config, false) {
                    Ok(cap) => {
                        recording_flag.store(true, Ordering::Relaxed);
                        let _ = event_tx.send(VoiceEvent::RecordingStarted);
                        active = Some(cap);
                    }
                    Err(e) => {
                        let _ = event_tx.send(VoiceEvent::Failed {
                            message: format!("録音開始失敗: {e}"),
                        });
                    }
                }
            }
        }
    }
}

fn worker_vad(
    config: SttConfig,
    cmd_rx: Receiver<VoiceCmd>,
    event_tx: Sender<VoiceEvent>,
    recording_flag: Arc<AtomicBool>,
    listening_flag: Arc<AtomicBool>,
    stt_flag: Arc<AtomicBool>,
) {
    let mut mic: Option<MicBuffer> = None;
    let mut vad = VadState::new();
    let mut last_len: usize = 0;
    let mut speech_started: Option<Instant> = None;
    let end_ms = config.silence_secs * 1000.0;

    if let Err(e) = arm_vad(
        &config,
        &mut mic,
        &mut vad,
        &mut last_len,
        &listening_flag,
        &event_tx,
    ) {
        let _ = event_tx.send(VoiceEvent::Failed {
            message: format!("VAD 開始失敗: {e}"),
        });
        return;
    }

    loop {
        let cmd = match cmd_rx.recv_timeout(WORKER_TICK) {
            Ok(c) => Some(c),
            Err(crossbeam::channel::RecvTimeoutError::Timeout) => None,
            Err(crossbeam::channel::RecvTimeoutError::Disconnected) => break,
        };

        match cmd {
            Some(VoiceCmd::Shutdown) => {
                listening_flag.store(false, Ordering::Relaxed);
                recording_flag.store(false, Ordering::Relaxed);
                break;
            }
            Some(VoiceCmd::Toggle) => {
                if vad.speaking {
                    speech_started = None;
                    finish_vad_utterance(
                        &config,
                        &mut mic,
                        &mut vad,
                        &mut last_len,
                        &recording_flag,
                        &listening_flag,
                        &stt_flag,
                        &event_tx,
                        true,
                    );
                } else if listening_flag.load(Ordering::Relaxed) {
                    listening_flag.store(false, Ordering::Relaxed);
                    mic = None;
                    vad.reset();
                    last_len = 0;
                    speech_started = None;
                    let _ = event_tx.send(VoiceEvent::ListeningPaused);
                } else if stt_flag.load(Ordering::Relaxed) {
                    let _ = event_tx.send(VoiceEvent::Failed {
                        message: "STT 処理中…少し待ってね".into(),
                    });
                } else if let Err(e) = arm_vad(
                    &config,
                    &mut mic,
                    &mut vad,
                    &mut last_len,
                    &listening_flag,
                    &event_tx,
                ) {
                    let _ = event_tx.send(VoiceEvent::Failed {
                        message: format!("VAD 開始失敗: {e}"),
                    });
                }
            }
            None => {
                if stt_flag.load(Ordering::Relaxed) || !listening_flag.load(Ordering::Relaxed) {
                    continue;
                }
                let Some(buf) = mic.as_ref() else {
                    continue;
                };
                let snap = buf.snapshot();
                let rate = buf.sample_rate.max(1);
                let new = snap.len().saturating_sub(last_len);
                last_len = snap.len();
                let dt_ms = new as f32 / rate as f32 * 1000.0;
                if dt_ms <= 0.0 {
                    continue;
                }
                let win_n = ((VAD_WINDOW_SECS * rate as f32) as usize).max(1);
                let window = if snap.len() > win_n {
                    &snap[snap.len() - win_n..]
                } else {
                    snap.as_slice()
                };
                let rms = rms_i16(window);
                let decision = vad.step(rms, config.silence_threshold, dt_ms, VAD_START_MS, end_ms);
                match decision {
                    VadDecision::Start => {
                        speech_started = Some(Instant::now());
                        recording_flag.store(true, Ordering::Relaxed);
                        let _ = event_tx.send(VoiceEvent::RecordingStarted);
                    }
                    VadDecision::End => {
                        speech_started = None;
                        finish_vad_utterance(
                            &config,
                            &mut mic,
                            &mut vad,
                            &mut last_len,
                            &recording_flag,
                            &listening_flag,
                            &stt_flag,
                            &event_tx,
                            true,
                        );
                    }
                    VadDecision::None => {
                        if vad.speaking {
                            let over_max = speech_started
                                .map(|t| t.elapsed().as_secs_f32() >= config.max_recording_secs)
                                .unwrap_or(false);
                            if over_max {
                                speech_started = None;
                                finish_vad_utterance(
                                    &config,
                                    &mut mic,
                                    &mut vad,
                                    &mut last_len,
                                    &recording_flag,
                                    &listening_flag,
                                    &stt_flag,
                                    &event_tx,
                                    true,
                                );
                            }
                        } else {
                            let preroll = ((VAD_PREROLL_SECS * rate as f32) as usize).max(1);
                            if let Some(buf) = mic.as_ref() {
                                buf.keep_last(preroll);
                                last_len = buf.len();
                            }
                        }
                    }
                }
            }
        }
    }
}

fn arm_vad(
    config: &SttConfig,
    mic: &mut Option<MicBuffer>,
    vad: &mut VadState,
    last_len: &mut usize,
    listening_flag: &AtomicBool,
    event_tx: &Sender<VoiceEvent>,
) -> Result<(), String> {
    *mic = Some(start_mic(config, true)?);
    vad.reset();
    *last_len = 0;
    listening_flag.store(true, Ordering::Relaxed);
    let _ = event_tx.send(VoiceEvent::ListeningStarted);
    Ok(())
}

#[allow(clippy::too_many_arguments)]
fn finish_vad_utterance(
    config: &SttConfig,
    mic: &mut Option<MicBuffer>,
    vad: &mut VadState,
    last_len: &mut usize,
    recording_flag: &AtomicBool,
    listening_flag: &AtomicBool,
    stt_flag: &AtomicBool,
    event_tx: &Sender<VoiceEvent>,
    rearm: bool,
) {
    let secs = mic.as_ref().map(|c| c.elapsed()).unwrap_or(0.0);
    let captured = stop_capture(mic, recording_flag);
    vad.reset();
    *last_len = 0;
    listening_flag.store(false, Ordering::Relaxed);
    let _ = event_tx.send(VoiceEvent::RecordingStopped { secs });
    process_capture_result(config, captured, secs, event_tx, stt_flag);
    if rearm {
        if let Err(e) = arm_vad(config, mic, vad, last_len, listening_flag, event_tx) {
            let _ = event_tx.send(VoiceEvent::Failed {
                message: format!("VAD 再開失敗: {e}"),
            });
        }
    }
}

// --- RMS VAD ----------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VadDecision {
    None,
    Start,
    End,
}

#[derive(Debug, Clone)]
pub struct VadState {
    pub speaking: bool,
    voiced_ms: f32,
    silence_ms: f32,
}

impl Default for VadState {
    fn default() -> Self {
        Self::new()
    }
}

impl VadState {
    pub fn new() -> Self {
        Self {
            speaking: false,
            voiced_ms: 0.0,
            silence_ms: 0.0,
        }
    }

    pub fn reset(&mut self) {
        self.speaking = false;
        self.voiced_ms = 0.0;
        self.silence_ms = 0.0;
    }

    /// Advance by one audio chunk. `dt_ms` is the chunk duration in milliseconds.
    pub fn step(
        &mut self,
        rms: f32,
        threshold: f32,
        dt_ms: f32,
        start_ms: f32,
        end_ms: f32,
    ) -> VadDecision {
        let dt_ms = dt_ms.max(0.0);
        let loud = rms >= threshold;
        if !self.speaking {
            if loud {
                self.voiced_ms += dt_ms;
                self.silence_ms = 0.0;
                if self.voiced_ms >= start_ms {
                    self.speaking = true;
                    self.silence_ms = 0.0;
                    return VadDecision::Start;
                }
            } else {
                self.voiced_ms = 0.0;
            }
            VadDecision::None
        } else if loud {
            self.silence_ms = 0.0;
            VadDecision::None
        } else {
            self.silence_ms += dt_ms;
            if self.silence_ms >= end_ms {
                self.speaking = false;
                self.voiced_ms = 0.0;
                self.silence_ms = 0.0;
                VadDecision::End
            } else {
                VadDecision::None
            }
        }
    }
}

pub fn rms_i16(samples: &[i16]) -> f32 {
    if samples.is_empty() {
        return 0.0;
    }
    let sum: f64 = samples
        .iter()
        .map(|&s| {
            let x = f64::from(s);
            x * x
        })
        .sum();
    (sum / samples.len() as f64).sqrt() as f32
}

// --- Capture ----------------------------------------------------------------

struct MicBuffer {
    samples: Arc<Mutex<Vec<i16>>>,
    sample_rate: u32,
    started: Instant,
    /// Keep stream alive while recording / listening.
    _stream: cpal::Stream,
    max_samples: usize,
}

impl MicBuffer {
    fn elapsed(&self) -> f32 {
        self.started.elapsed().as_secs_f32()
    }

    fn len(&self) -> usize {
        self.samples.lock().unwrap_or_else(|e| e.into_inner()).len()
    }

    fn snapshot(&self) -> Vec<i16> {
        self.samples
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .clone()
    }

    fn keep_last(&self, n: usize) {
        let mut guard = self.samples.lock().unwrap_or_else(|e| e.into_inner());
        if guard.len() > n {
            let start = guard.len() - n;
            guard.drain(0..start);
        }
    }

    fn take_samples(&self) -> (Vec<i16>, u32) {
        let mut guard = self.samples.lock().unwrap_or_else(|e| e.into_inner());
        if guard.len() > self.max_samples {
            guard.truncate(self.max_samples);
        }
        let out = std::mem::take(&mut *guard);
        (out, self.sample_rate)
    }
}

struct CaptureResult {
    samples: Vec<i16>,
    rate: u32,
}

fn start_mic(config: &SttConfig, always_on: bool) -> Result<MicBuffer, String> {
    let host = cpal::default_host();
    let device = host
        .default_input_device()
        .ok_or_else(|| "no default input device".to_string())?;
    let supported = device
        .default_input_config()
        .map_err(|e| format!("input config: {e}"))?;
    let sample_format = supported.sample_format();
    let stream_config: StreamConfig = supported.config();
    let channels = stream_config.channels as usize;
    let sample_rate = stream_config.sample_rate;
    let extra = if always_on { 1.0 } else { 0.5 };
    let max_samples = ((config.max_recording_secs + extra) * sample_rate as f32).ceil() as usize
        + sample_rate as usize;

    let samples = Arc::new(Mutex::new(Vec::with_capacity(max_samples.min(48_000 * 8))));
    let samples_cb = Arc::clone(&samples);
    let err_fn = |e| eprintln!("strudel-rs voice input stream error: {e}");

    let stream = match sample_format {
        SampleFormat::F32 => build_input_stream::<f32>(
            &device,
            stream_config,
            channels,
            samples_cb,
            max_samples,
            err_fn,
        )?,
        SampleFormat::I16 => build_input_stream::<i16>(
            &device,
            stream_config,
            channels,
            samples_cb,
            max_samples,
            err_fn,
        )?,
        SampleFormat::U16 => build_input_stream::<u16>(
            &device,
            stream_config,
            channels,
            samples_cb,
            max_samples,
            err_fn,
        )?,
        other => return Err(format!("unsupported input sample format: {other:?}")),
    };

    stream
        .play()
        .map_err(|e| format!("input stream play: {e}"))?;

    Ok(MicBuffer {
        samples,
        sample_rate,
        started: Instant::now(),
        _stream: stream,
        max_samples,
    })
}

fn build_input_stream<T>(
    device: &cpal::Device,
    config: StreamConfig,
    channels: usize,
    samples: Arc<Mutex<Vec<i16>>>,
    max_samples: usize,
    err_fn: impl FnMut(cpal::Error) + Send + 'static,
) -> Result<cpal::Stream, String>
where
    T: Sample + SizedSample + Send + 'static,
    i16: FromSample<T>,
{
    let ch = channels.max(1);
    device
        .build_input_stream(
            config,
            move |data: &[T], _| {
                let mut guard = match samples.lock() {
                    Ok(g) => g,
                    Err(e) => e.into_inner(),
                };
                if guard.len() >= max_samples {
                    return;
                }
                let frames = data.len() / ch;
                for f in 0..frames {
                    if guard.len() >= max_samples {
                        break;
                    }
                    let s = data[f * ch];
                    guard.push(s.to_sample::<i16>());
                }
            },
            err_fn,
            None,
        )
        .map_err(|e| format!("build input stream: {e}"))
}

fn stop_capture(active: &mut Option<MicBuffer>, recording_flag: &AtomicBool) -> CaptureResult {
    recording_flag.store(false, Ordering::Relaxed);
    if let Some(cap) = active.take() {
        let (samples, rate) = cap.take_samples();
        CaptureResult { samples, rate }
    } else {
        CaptureResult {
            samples: Vec::new(),
            rate: TARGET_STT_RATE,
        }
    }
}

fn process_capture_result(
    config: &SttConfig,
    captured: CaptureResult,
    secs: f32,
    event_tx: &Sender<VoiceEvent>,
    stt_flag: &AtomicBool,
) {
    if secs < config.min_recording_secs || captured.samples.len() < 160 {
        let _ = event_tx.send(VoiceEvent::Failed {
            message: "録音が短すぎます".into(),
        });
        return;
    }

    stt_flag.store(true, Ordering::Relaxed);
    let _ = event_tx.send(VoiceEvent::SttRunning);

    let (pcm, rate) = downsample_mono(&captured.samples, captured.rate, TARGET_STT_RATE);
    let wav = pcm_i16_to_wav(&pcm, rate);

    match transcribe_wav_bytes(config, &wav) {
        Ok(text) => {
            let text = text.trim().to_string();
            if text.is_empty() {
                let _ = event_tx.send(VoiceEvent::Failed {
                    message: "聞き取れませんでした".into(),
                });
            } else {
                let _ = event_tx.send(VoiceEvent::Transcript { text });
            }
        }
        Err(e) => {
            let _ = event_tx.send(VoiceEvent::Failed {
                message: format!("STT: {e}"),
            });
        }
    }
    stt_flag.store(false, Ordering::Relaxed);
}

// --- WAV / resample / STT ---------------------------------------------------

/// Build a mono 16-bit LE WAV in memory.
pub fn pcm_i16_to_wav(samples: &[i16], sample_rate: u32) -> Vec<u8> {
    let data_bytes = samples.len() * 2;
    let mut out = Vec::with_capacity(44 + data_bytes);
    let sample_rate = sample_rate.max(1);
    let byte_rate = sample_rate * 2; // mono i16
    let block_align: u16 = 2;
    let bits_per_sample: u16 = 16;
    let channels: u16 = 1;

    out.extend_from_slice(b"RIFF");
    out.extend_from_slice(&(36 + data_bytes as u32).to_le_bytes());
    out.extend_from_slice(b"WAVE");
    out.extend_from_slice(b"fmt ");
    out.extend_from_slice(&16u32.to_le_bytes());
    out.extend_from_slice(&1u16.to_le_bytes());
    out.extend_from_slice(&channels.to_le_bytes());
    out.extend_from_slice(&sample_rate.to_le_bytes());
    out.extend_from_slice(&byte_rate.to_le_bytes());
    out.extend_from_slice(&block_align.to_le_bytes());
    out.extend_from_slice(&bits_per_sample.to_le_bytes());
    out.extend_from_slice(b"data");
    out.extend_from_slice(&(data_bytes as u32).to_le_bytes());
    for s in samples {
        out.extend_from_slice(&s.to_le_bytes());
    }
    out
}

/// Integer-step downsample (or pass-through / naive upsample by hold).
pub fn downsample_mono(samples: &[i16], from_rate: u32, to_rate: u32) -> (Vec<i16>, u32) {
    let from_rate = from_rate.max(1);
    let to_rate = to_rate.max(1);
    if from_rate == to_rate || samples.is_empty() {
        return (samples.to_vec(), from_rate);
    }
    if from_rate > to_rate {
        let step = (from_rate as f64 / to_rate as f64).round().max(1.0) as usize;
        let out: Vec<i16> = samples.iter().step_by(step).copied().collect();
        let actual = (from_rate as f64 / step as f64).round() as u32;
        return (out, actual.max(1));
    }
    let factor = (to_rate as f64 / from_rate as f64).round().max(1.0) as usize;
    let mut out = Vec::with_capacity(samples.len() * factor);
    for &s in samples {
        for _ in 0..factor {
            out.push(s);
        }
    }
    (out, to_rate)
}

pub fn parse_transcript_json(body: &str) -> Result<String, String> {
    let v: Value = serde_json::from_str(body).map_err(|e| format!("json: {e}"))?;
    Ok(v.get("text")
        .and_then(|t| t.as_str())
        .unwrap_or("")
        .to_string())
}

/// POST `{base}/v1/audio/transcriptions` with multipart WAV.
pub fn transcribe_wav_bytes(config: &SttConfig, wav: &[u8]) -> Result<String, String> {
    let client = reqwest::blocking::Client::builder()
        .timeout(config.timeout)
        .build()
        .map_err(|e| format!("http client: {e}"))?;

    let url = transcription_url(&config.base_url);
    let part = multipart::Part::bytes(wav.to_vec())
        .file_name("audio.wav")
        .mime_str("audio/wav")
        .map_err(|e| format!("multipart: {e}"))?;

    let mut form = multipart::Form::new()
        .part("file", part)
        .text("model", config.model.clone())
        .text("response_format", "json");
    if !config.language.is_empty() {
        form = form.text("language", config.language.clone());
    }

    let mut req = client.post(&url).multipart(form);
    if !config.api_key.is_empty() {
        req = req.bearer_auth(&config.api_key);
    }

    let res = req.send().map_err(|e| format!("request: {e}"))?;

    let status = res.status();
    let body = res.text().map_err(|e| format!("read body: {e}"))?;
    if !status.is_success() {
        let detail = truncate(&body, 300);
        return Err(format!("HTTP {status}: {detail}"));
    }

    parse_transcript_json(&body)
}

fn env_nonempty(key: &str) -> Option<String> {
    std::env::var(key).ok().and_then(|s| {
        let t = s.trim().to_string();
        if t.is_empty() {
            None
        } else {
            Some(t)
        }
    })
}

fn truncate(s: &str, max: usize) -> String {
    let mut out = String::new();
    for (i, c) in s.chars().enumerate() {
        if i >= max {
            out.push('…');
            break;
        }
        out.push(c);
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wav_header_riff_and_sizes() {
        let samples = [0i16, 1000, -1000, 0];
        let wav = pcm_i16_to_wav(&samples, 16_000);
        assert!(wav.len() >= 44 + samples.len() * 2);
        assert_eq!(&wav[0..4], b"RIFF");
        assert_eq!(&wav[8..12], b"WAVE");
        assert_eq!(&wav[12..16], b"fmt ");
        assert_eq!(&wav[36..40], b"data");
        let data_size = u32::from_le_bytes(wav[40..44].try_into().unwrap());
        assert_eq!(data_size as usize, samples.len() * 2);
    }

    #[test]
    fn downsample_halves_rate() {
        let src: Vec<i16> = (0..100).map(|i| i as i16).collect();
        let (out, rate) = downsample_mono(&src, 32_000, 16_000);
        assert!(out.len() <= src.len());
        assert!(rate <= 32_000);
        assert!(!out.is_empty());
    }

    #[test]
    fn enable_voice_requires_base_url() {
        assert!(!enable_voice(None));
        assert!(!enable_voice(Some("")));
        assert!(!enable_voice(Some("   ")));
        assert!(enable_voice(Some("http://192.168.1.10:8090")));
    }

    #[test]
    fn transcription_url_openai_path() {
        assert_eq!(
            transcription_url("http://192.168.1.10:8090"),
            "http://192.168.1.10:8090/v1/audio/transcriptions"
        );
        assert_eq!(
            transcription_url("http://192.168.1.10:8090/"),
            "http://192.168.1.10:8090/v1/audio/transcriptions"
        );
    }

    #[test]
    fn parse_transcript_json_reads_text() {
        assert_eq!(
            parse_transcript_json(r#"{"text":"暗くして"}"#).unwrap(),
            "暗くして"
        );
        assert_eq!(parse_transcript_json(r#"{"text":""}"#).unwrap(), "");
        assert_eq!(parse_transcript_json(r#"{}"#).unwrap(), "");
        assert!(parse_transcript_json("not-json").is_err());
    }

    #[test]
    fn voice_mode_parse() {
        assert_eq!(VoiceMode::parse("push"), VoiceMode::Push);
        assert_eq!(VoiceMode::parse("vad"), VoiceMode::Vad);
        assert_eq!(VoiceMode::parse("LISTEN"), VoiceMode::Vad);
        assert_eq!(VoiceMode::parse(""), VoiceMode::Push);
        assert_eq!(VoiceMode::parse("nope"), VoiceMode::Push);
    }

    #[test]
    fn rms_zero_and_constant() {
        assert_eq!(rms_i16(&[]), 0.0);
        assert_eq!(rms_i16(&[0, 0, 0]), 0.0);
        let r = rms_i16(&[1000, 1000, 1000, 1000]);
        assert!((r - 1000.0).abs() < 0.01, "rms={r}");
    }

    #[test]
    fn vad_silence_does_not_start() {
        let mut v = VadState::new();
        for _ in 0..20 {
            let d = v.step(10.0, 500.0, 40.0, 80.0, 1200.0);
            assert_eq!(d, VadDecision::None);
            assert!(!v.speaking);
        }
    }

    #[test]
    fn vad_loud_then_silence_starts_and_ends() {
        let mut v = VadState::new();
        assert_eq!(v.step(800.0, 500.0, 40.0, 80.0, 200.0), VadDecision::None);
        assert_eq!(v.step(800.0, 500.0, 40.0, 80.0, 200.0), VadDecision::Start);
        assert!(v.speaking);
        assert_eq!(v.step(800.0, 500.0, 40.0, 80.0, 200.0), VadDecision::None);
        assert_eq!(v.step(10.0, 500.0, 40.0, 80.0, 200.0), VadDecision::None);
        assert_eq!(v.step(10.0, 500.0, 160.0, 80.0, 200.0), VadDecision::End);
        assert!(!v.speaking);
    }

    #[test]
    fn from_parts_strips_slash() {
        let cfg = SttConfig::from_parts(
            "http://127.0.0.1:8090/".into(),
            String::new(),
            "ja".into(),
            "whisper-1".into(),
            Duration::from_secs(30),
            7.0,
            VoiceMode::Push,
            500.0,
            1.2,
        );
        assert_eq!(cfg.base_url, "http://127.0.0.1:8090");
        assert!(cfg.api_key.is_empty());
        assert_eq!(
            transcription_url(&cfg.base_url),
            "http://127.0.0.1:8090/v1/audio/transcriptions"
        );
    }
}

//! HTTP API (REST + SSE) — external control plane for the live engine.
//!
//! Binds to `127.0.0.1` only. Default port is [`DEFAULT_API_PORT`] (10000s range).

use std::convert::Infallible;
use std::path::Path;
use std::sync::{Arc, Mutex};

use axum::extract::State;
use axum::http::StatusCode;
use axum::response::sse::{Event, Sse};
use axum::routing::{get, post, put};
use axum::{Json, Router};
use crossbeam::channel::Sender;
use serde::{Deserialize, Serialize};

use crate::code::parse_code;
use crate::engine::{Command, Engine};
use crate::song::{parse_song, resolve_song_path, Song, Track};

// Re-export for callers/tests that used api::sanitize_song_path.
pub use crate::song::sanitize_song_path;

/// Default listen port (10000s; avoids commonly busy 7878).
pub const DEFAULT_API_PORT: u16 = 17878;

/// Environment variable for default port override (`STRUDEL_API_PORT`).
pub const ENV_API_PORT: &str = "STRUDEL_API_PORT";

/// Environment variable for full base URL (MCP / scripts): e.g. `http://127.0.0.1:17878`.
pub const ENV_API_BASE: &str = "STRUDEL_API";

#[derive(Clone)]
pub struct AppState {
    pub tx: Sender<Command>,
    pub engine: Arc<Mutex<Engine>>,
}

/// Channel EQ slider positions (0..=1, 0.5 = flat).
#[derive(Clone, Debug, Serialize, PartialEq)]
pub struct EqBands {
    pub hi: f32,
    pub mid: f32,
    pub lo: f32,
}

impl Default for EqBands {
    fn default() -> Self {
        Self {
            hi: 0.5,
            mid: 0.5,
            lo: 0.5,
        }
    }
}

impl EqBands {
    fn from_arr(p: [f32; 3]) -> Self {
        Self {
            hi: p[0],
            mid: p[1],
            lo: p[2],
        }
    }
}

#[derive(Clone, Debug, Default, Serialize, PartialEq)]
pub struct StatusInfo {
    pub deck_a: Option<String>,
    pub deck_b: Option<String>,
    pub bpm: f64,
    pub playing: bool,
    /// Shared transport bar index (0-based).
    pub bar: u64,
    /// Per-deck song position (1-based; reflects head/cue offset).
    pub song_bar_a: u64,
    pub song_bar_b: u64,
    pub gain_a: f32,
    pub gain_b: f32,
    pub eq_a: EqBands,
    pub eq_b: EqBands,
    /// Master LPF cutoff Hz; `null` = bypass.
    pub lpf_hz: Option<f32>,
    /// Master HPF cutoff Hz; `null` = bypass.
    pub hpf_hz: Option<f32>,
    /// Equal-power crossfader 0=A … 1=B.
    pub crossfader: f32,
}

#[derive(Deserialize)]
pub struct CodeReq {
    pub code: String,
    pub deck: Option<String>,
}

#[derive(Deserialize)]
pub struct LoadReq {
    pub path: String,
    pub deck: String,
}

#[derive(Deserialize)]
pub struct XFadeReq {
    pub to: String,
    pub bars: Option<u32>,
}

#[derive(Deserialize)]
pub struct BpmReq {
    pub bpm: f64,
}

#[derive(Deserialize)]
pub struct MuteReq {
    pub deck: String,
    pub track: String,
    pub muted: bool,
}

#[derive(Deserialize)]
pub struct HeadReq {
    pub deck: String,
    /// 1-based song bar (1 = first bar). Applies at next transport bar boundary.
    pub bar: u64,
}

/// Partial EQ update. At least one of hi/mid/lo required. Values 0..=1 (0.5 = flat).
#[derive(Deserialize)]
pub struct MixerEqReq {
    pub deck: String,
    pub hi: Option<f32>,
    pub mid: Option<f32>,
    pub lo: Option<f32>,
}

/// Master filter field: omit = no change; `null` = bypass; number = Hz.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub enum FilterField {
    #[default]
    Absent,
    Bypass,
    Hz(f32),
}

impl<'de> Deserialize<'de> for FilterField {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        use serde::de::{self, Visitor};
        use std::fmt;

        struct Fv;
        impl<'de> Visitor<'de> for Fv {
            type Value = FilterField;

            fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
                f.write_str("a positive Hz number or null")
            }

            fn visit_unit<E: de::Error>(self) -> Result<Self::Value, E> {
                Ok(FilterField::Bypass)
            }

            fn visit_none<E: de::Error>(self) -> Result<Self::Value, E> {
                Ok(FilterField::Bypass)
            }

            fn visit_f64<E: de::Error>(self, v: f64) -> Result<Self::Value, E> {
                let hz = v as f32;
                if !(hz.is_finite() && hz > 0.0) {
                    return Err(E::custom(format!(
                        "filter Hz must be positive finite, got {v}"
                    )));
                }
                Ok(FilterField::Hz(hz))
            }

            fn visit_i64<E: de::Error>(self, v: i64) -> Result<Self::Value, E> {
                self.visit_f64(v as f64)
            }

            fn visit_u64<E: de::Error>(self, v: u64) -> Result<Self::Value, E> {
                self.visit_f64(v as f64)
            }
        }

        deserializer.deserialize_any(Fv)
    }
}

/// Master filter. Field absent = leave unchanged; JSON `null` = bypass; number = set Hz.
#[derive(Deserialize)]
pub struct MixerFilterReq {
    #[serde(default)]
    pub lpf: FilterField,
    #[serde(default)]
    pub hpf: FilterField,
}

#[derive(Deserialize)]
pub struct MixerCrossfaderReq {
    /// 0 = full A, 1 = full B (immediate; cancels multi-bar xfade).
    pub pos: f32,
}

#[derive(Serialize)]
pub struct ErrRes {
    pub error: String,
}

fn bad(e: impl Into<String>) -> (StatusCode, Json<ErrRes>) {
    (StatusCode::BAD_REQUEST, Json(ErrRes { error: e.into() }))
}

fn snapshot(engine: &Arc<Mutex<Engine>>) -> StatusInfo {
    let Ok(e) = engine.lock() else {
        return StatusInfo::default();
    };
    let deck_a = e.decks[0].song_title().map(|s| s.to_string());
    let deck_b = e.decks[1].song_title().map(|s| s.to_string());
    let playing = deck_a.is_some() || deck_b.is_some();
    let bar = e.transport.bar_index();
    StatusInfo {
        deck_a,
        deck_b,
        bpm: e.transport.bpm,
        playing,
        bar,
        song_bar_a: e.decks[0].song_bar_1based(bar),
        song_bar_b: e.decks[1].song_bar_1based(bar),
        gain_a: e.mixer.gain_a,
        gain_b: e.mixer.gain_b,
        eq_a: EqBands::from_arr(e.mixer.deck_eq(0)),
        eq_b: EqBands::from_arr(e.mixer.deck_eq(1)),
        lpf_hz: e.mixer.lpf_hz,
        hpf_hz: e.mixer.hpf_hz,
        crossfader: e.mixer.crossfader_pos(),
    }
}

pub fn deck_idx(s: &str) -> Result<usize, String> {
    match s {
        "A" | "a" => Ok(0),
        "B" | "b" => Ok(1),
        _ => Err(format!("deck must be A or B: {s}")),
    }
}

fn song_from_code(code: &str) -> Result<Song, String> {
    let pattern = parse_code(code)?;
    Ok(Song {
        title: "api".into(),
        meta: Default::default(),
        bpm: None,
        path: String::new(),
        source: code.to_string(),
        tracks: vec![Track {
            name: "main".into(),
            muted: false,
            code: pattern,
        }],
    })
}

/// One-shot pattern load (applies from next bar via Command queue).
async fn put_code(
    State(s): State<AppState>,
    Json(r): Json<CodeReq>,
) -> Result<StatusCode, (StatusCode, Json<ErrRes>)> {
    let deck = deck_idx(r.deck.as_deref().unwrap_or("A")).map_err(bad)?;
    let song = song_from_code(&r.code).map_err(bad)?;
    s.tx.send(Command::LoadSong {
        deck,
        song: Box::new(song),
    })
    .map_err(|e| bad(e.to_string()))?;
    Ok(StatusCode::ACCEPTED)
}

async fn load_song(
    State(s): State<AppState>,
    Json(r): Json<LoadReq>,
) -> Result<StatusCode, (StatusCode, Json<ErrRes>)> {
    let deck = deck_idx(&r.deck).map_err(bad)?;
    let path = resolve_song_path(&r.path).map_err(bad)?;
    let text = std::fs::read_to_string(&path).map_err(|e| bad(format!("read: {e}")))?;
    let path_str = path.to_string_lossy();
    let song = parse_song(&text, &path_str).map_err(bad)?;
    s.tx.send(Command::LoadSong {
        deck,
        song: Box::new(song),
    })
    .map_err(|e| bad(e.to_string()))?;
    Ok(StatusCode::ACCEPTED)
}

async fn xfade(
    State(s): State<AppState>,
    Json(r): Json<XFadeReq>,
) -> Result<StatusCode, (StatusCode, Json<ErrRes>)> {
    let to_deck = deck_idx(&r.to).map_err(bad)?;
    let bars = r.bars.unwrap_or(4).max(1);
    s.tx.send(Command::XFade { to_deck, bars })
        .map_err(|e| bad(e.to_string()))?;
    Ok(StatusCode::ACCEPTED)
}

async fn set_bpm(
    State(s): State<AppState>,
    Json(r): Json<BpmReq>,
) -> Result<StatusCode, (StatusCode, Json<ErrRes>)> {
    if !(r.bpm.is_finite() && r.bpm > 0.0) {
        return Err(bad(format!(
            "bpm must be a positive finite number: {}",
            r.bpm
        )));
    }
    s.tx.send(Command::SetBpm(r.bpm))
        .map_err(|e| bad(e.to_string()))?;
    Ok(StatusCode::ACCEPTED)
}

async fn mute(
    State(s): State<AppState>,
    Json(r): Json<MuteReq>,
) -> Result<StatusCode, (StatusCode, Json<ErrRes>)> {
    let deck = deck_idx(&r.deck).map_err(bad)?;
    if r.track.trim().is_empty() {
        return Err(bad("track name is empty"));
    }
    s.tx.send(Command::SetTrackMute {
        deck,
        track: r.track,
        muted: r.muted,
    })
    .map_err(|e| bad(e.to_string()))?;
    Ok(StatusCode::ACCEPTED)
}

async fn head(
    State(s): State<AppState>,
    Json(r): Json<HeadReq>,
) -> Result<StatusCode, (StatusCode, Json<ErrRes>)> {
    let deck = deck_idx(&r.deck).map_err(bad)?;
    if r.bar < 1 {
        return Err(bad("bar must be >= 1 (1 = first bar of the song)"));
    }
    s.tx.send(Command::Head { deck, bar: r.bar })
        .map_err(|e| bad(e.to_string()))?;
    Ok(StatusCode::ACCEPTED)
}

async fn hush(State(s): State<AppState>) -> StatusCode {
    let _ = s.tx.send(Command::Hush);
    StatusCode::NO_CONTENT
}

async fn mixer_eq(
    State(s): State<AppState>,
    Json(r): Json<MixerEqReq>,
) -> Result<StatusCode, (StatusCode, Json<ErrRes>)> {
    let deck = deck_idx(&r.deck).map_err(bad)?;
    let bands: [(u8, Option<f32>); 3] = [(0, r.hi), (1, r.mid), (2, r.lo)];
    if bands.iter().all(|(_, v)| v.is_none()) {
        return Err(bad("provide at least one of hi, mid, lo (0..=1, 0.5=flat)"));
    }
    for (band, val) in bands {
        let Some(v) = val else { continue };
        if !v.is_finite() {
            return Err(bad(format!("eq band {band} must be finite")));
        }
        s.tx.send(Command::SetDeckEq {
            deck,
            band,
            value: v.clamp(0.0, 1.0),
        })
        .map_err(|e| bad(e.to_string()))?;
    }
    Ok(StatusCode::NO_CONTENT)
}

async fn mixer_filter(
    State(s): State<AppState>,
    Json(r): Json<MixerFilterReq>,
) -> Result<StatusCode, (StatusCode, Json<ErrRes>)> {
    if r.lpf == FilterField::Absent && r.hpf == FilterField::Absent {
        return Err(bad("provide lpf and/or hpf (Hz number, or null to bypass)"));
    }
    match r.lpf {
        FilterField::Absent => {}
        FilterField::Bypass => {
            s.tx.send(Command::SetMixerLpf(None))
                .map_err(|e| bad(e.to_string()))?;
        }
        FilterField::Hz(hz) => {
            s.tx.send(Command::SetMixerLpf(Some(hz)))
                .map_err(|e| bad(e.to_string()))?;
        }
    }
    match r.hpf {
        FilterField::Absent => {}
        FilterField::Bypass => {
            s.tx.send(Command::SetMixerHpf(None))
                .map_err(|e| bad(e.to_string()))?;
        }
        FilterField::Hz(hz) => {
            s.tx.send(Command::SetMixerHpf(Some(hz)))
                .map_err(|e| bad(e.to_string()))?;
        }
    }
    Ok(StatusCode::NO_CONTENT)
}

async fn mixer_crossfader(
    State(s): State<AppState>,
    Json(r): Json<MixerCrossfaderReq>,
) -> Result<StatusCode, (StatusCode, Json<ErrRes>)> {
    if !r.pos.is_finite() {
        return Err(bad(format!("pos must be finite: {}", r.pos)));
    }
    s.tx.send(Command::SetCrossfader(r.pos.clamp(0.0, 1.0)))
        .map_err(|e| bad(e.to_string()))?;
    Ok(StatusCode::NO_CONTENT)
}

async fn get_status(State(s): State<AppState>) -> Json<StatusInfo> {
    Json(snapshot(&s.engine))
}

async fn health() -> &'static str {
    "ok"
}

async fn events(
    State(s): State<AppState>,
) -> Sse<impl tokio_stream::Stream<Item = Result<Event, Infallible>>> {
    let engine = Arc::clone(&s.engine);
    let st = async_stream::stream! {
        let mut last = String::new();
        loop {
            let info = snapshot(&engine);
            let cur = serde_json::to_string(&info).unwrap_or_else(|_| "{}".into());
            if cur != last {
                last = cur.clone();
                yield Ok(Event::default().data(cur));
            }
            tokio::time::sleep(std::time::Duration::from_millis(250)).await;
        }
    };
    Sse::new(st)
}

pub fn router(state: AppState) -> Router {
    Router::new()
        .route("/health", get(health))
        .route("/code", put(put_code))
        .route("/song/load", post(load_song))
        .route("/xfade", post(xfade))
        .route("/bpm", post(set_bpm))
        .route("/mute", post(mute))
        .route("/head", post(head))
        .route("/hush", post(hush))
        .route("/mixer/eq", post(mixer_eq))
        .route("/mixer/filter", post(mixer_filter))
        .route("/mixer/crossfader", post(mixer_crossfader))
        .route("/status", get(get_status))
        .route("/events", get(events))
        .with_state(state)
}

/// Resolve listen port: CLI override → env → default.
pub fn resolve_port(cli_port: Option<u16>) -> u16 {
    if let Some(p) = cli_port {
        return p;
    }
    if let Ok(s) = std::env::var(ENV_API_PORT) {
        if let Ok(p) = s.parse::<u16>() {
            if p > 0 {
                return p;
            }
        }
    }
    DEFAULT_API_PORT
}

/// Default base URL for MCP / scripts.
pub fn default_api_base() -> String {
    if let Ok(base) = std::env::var(ENV_API_BASE) {
        let t = base.trim().trim_end_matches('/').to_string();
        if !t.is_empty() {
            return t;
        }
    }
    format!("http://127.0.0.1:{}", resolve_port(None))
}

/// Block the current thread serving HTTP on `127.0.0.1:port`.
pub fn serve_blocking(state: AppState, port: u16) -> Result<(), String> {
    let rt = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .map_err(|e| format!("tokio runtime: {e}"))?;
    rt.block_on(async move {
        let addr = std::net::SocketAddr::from(([127, 0, 0, 1], port));
        let listener = tokio::net::TcpListener::bind(addr)
            .await
            .map_err(|e| format!("bind {addr}: {e}"))?;
        eprintln!("API on http://127.0.0.1:{port}");
        axum::serve(listener, router(state))
            .await
            .map_err(|e| format!("api serve: {e}"))
    })
}

/// Spawn a background OS thread that runs the HTTP server.
pub fn spawn_server(state: AppState, port: u16) -> std::thread::JoinHandle<()> {
    std::thread::Builder::new()
        .name("strudel-api".into())
        .spawn(move || {
            if let Err(e) = serve_blocking(state, port) {
                eprintln!("API server stopped: {e}");
            }
        })
        .expect("spawn API thread")
}

/// Helper for tests / docs: join path display without allocating when not needed.
#[allow(dead_code)]
pub fn path_display(p: &Path) -> String {
    p.display().to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::Body;
    use axum::http::{Request, StatusCode};
    use crossbeam::channel::unbounded;
    use http_body_util::BodyExt;
    use tower::ServiceExt;

    fn test_state() -> (AppState, crossbeam::channel::Receiver<Command>) {
        let (tx, rx) = unbounded();
        let engine = Arc::new(Mutex::new(Engine::new(44100, 120.0)));
        (AppState { tx, engine }, rx)
    }

    async fn json_body(res: axum::response::Response) -> String {
        let bytes = res.into_body().collect().await.unwrap().to_bytes();
        String::from_utf8(bytes.to_vec()).unwrap()
    }

    #[tokio::test]
    async fn health_ok() {
        let (state, _) = test_state();
        let app = router(state);
        let res = app
            .oneshot(
                Request::builder()
                    .uri("/health")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(res.status(), StatusCode::OK);
        assert_eq!(json_body(res).await, "ok");
    }

    #[tokio::test]
    async fn put_code_accepts_valid_and_queues_load() {
        let (state, rx) = test_state();
        let app = router(state);
        let body = r#"{"code":"note(\"c3 e3 g3\").s(\"triangle\")","deck":"A"}"#;
        let res = app
            .oneshot(
                Request::builder()
                    .method("PUT")
                    .uri("/code")
                    .header("content-type", "application/json")
                    .body(Body::from(body))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(res.status(), StatusCode::ACCEPTED);
        let cmd = rx.try_recv().expect("command queued");
        match cmd {
            Command::LoadSong { deck, song } => {
                assert_eq!(deck, 0);
                assert_eq!(song.title, "api");
                assert_eq!(song.tracks.len(), 1);
            }
            _ => panic!("expected LoadSong"),
        }
    }

    #[tokio::test]
    async fn put_code_bad_parse_returns_400_no_command() {
        let (state, rx) = test_state();
        let app = router(state);
        let body = r#"{"code":"note(\"!!!\"","deck":"A"}"#;
        let res = app
            .oneshot(
                Request::builder()
                    .method("PUT")
                    .uri("/code")
                    .header("content-type", "application/json")
                    .body(Body::from(body))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(res.status(), StatusCode::BAD_REQUEST);
        let text = json_body(res).await;
        assert!(text.contains("error"), "{text}");
        assert!(rx.try_recv().is_err(), "must not queue on parse error");
    }

    #[tokio::test]
    async fn load_missing_file_returns_400() {
        let (state, rx) = test_state();
        let app = router(state);
        let body = r#"{"path":"songs/__no_such_file__.strudel","deck":"A"}"#;
        let res = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/song/load")
                    .header("content-type", "application/json")
                    .body(Body::from(body))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(res.status(), StatusCode::BAD_REQUEST);
        assert!(rx.try_recv().is_err());
    }

    #[tokio::test]
    async fn load_rejects_parent_dir() {
        let (state, rx) = test_state();
        let app = router(state);
        let body = r#"{"path":"../secret.strudel","deck":"A"}"#;
        let res = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/song/load")
                    .header("content-type", "application/json")
                    .body(Body::from(body))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(res.status(), StatusCode::BAD_REQUEST);
        let text = json_body(res).await;
        assert!(text.contains(".."), "{text}");
        assert!(rx.try_recv().is_err());
    }

    #[tokio::test]
    async fn bad_deck_returns_400() {
        let (state, _) = test_state();
        let app = router(state);
        let body = r#"{"to":"C","bars":4}"#;
        let res = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/xfade")
                    .header("content-type", "application/json")
                    .body(Body::from(body))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(res.status(), StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn bad_bpm_returns_400() {
        let (state, rx) = test_state();
        let app = router(state);
        let body = r#"{"bpm":-1}"#;
        let res = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/bpm")
                    .header("content-type", "application/json")
                    .body(Body::from(body))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(res.status(), StatusCode::BAD_REQUEST);
        assert!(rx.try_recv().is_err());
    }

    #[tokio::test]
    async fn hush_returns_204() {
        let (state, rx) = test_state();
        let app = router(state);
        let res = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/hush")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(res.status(), StatusCode::NO_CONTENT);
        assert!(matches!(rx.try_recv().unwrap(), Command::Hush));
    }

    #[tokio::test]
    async fn head_accepts_and_queues() {
        let (state, rx) = test_state();
        let app = router(state);
        let body = r#"{"deck":"B","bar":33}"#;
        let res = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/head")
                    .header("content-type", "application/json")
                    .body(Body::from(body))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(res.status(), StatusCode::ACCEPTED);
        match rx.try_recv().unwrap() {
            Command::Head { deck, bar } => {
                assert_eq!(deck, 1);
                assert_eq!(bar, 33);
            }
            _ => panic!("expected Head"),
        }
    }

    #[tokio::test]
    async fn head_rejects_bar_zero() {
        let (state, rx) = test_state();
        let app = router(state);
        let body = r#"{"deck":"A","bar":0}"#;
        let res = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/head")
                    .header("content-type", "application/json")
                    .body(Body::from(body))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(res.status(), StatusCode::BAD_REQUEST);
        assert!(rx.try_recv().is_err());
    }

    #[tokio::test]
    async fn mixer_eq_queues_set_deck_eq() {
        let (state, rx) = test_state();
        let app = router(state);
        let body = r#"{"deck":"B","lo":0.2,"hi":0.8}"#;
        let res = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/mixer/eq")
                    .header("content-type", "application/json")
                    .body(Body::from(body))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(res.status(), StatusCode::NO_CONTENT);
        let mut got = Vec::new();
        while let Ok(c) = rx.try_recv() {
            got.push(c);
        }
        assert_eq!(got.len(), 2);
        match &got[0] {
            Command::SetDeckEq { deck, band, value } => {
                assert_eq!(*deck, 1);
                assert_eq!(*band, 0); // hi first in send order
                assert!((*value - 0.8).abs() < 1e-5);
            }
            _ => panic!("expected SetDeckEq hi"),
        }
        match &got[1] {
            Command::SetDeckEq { deck, band, value } => {
                assert_eq!(*deck, 1);
                assert_eq!(*band, 2); // lo
                assert!((*value - 0.2).abs() < 1e-5);
            }
            _ => panic!("expected SetDeckEq lo"),
        }
    }

    #[tokio::test]
    async fn mixer_eq_requires_a_band() {
        let (state, rx) = test_state();
        let app = router(state);
        let body = r#"{"deck":"A"}"#;
        let res = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/mixer/eq")
                    .header("content-type", "application/json")
                    .body(Body::from(body))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(res.status(), StatusCode::BAD_REQUEST);
        assert!(rx.try_recv().is_err());
    }

    #[tokio::test]
    async fn mixer_filter_and_crossfader() {
        let (state, rx) = test_state();
        let app = router(state.clone());
        let res = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/mixer/filter")
                    .header("content-type", "application/json")
                    .body(Body::from(r#"{"lpf":4000,"hpf":null}"#))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(res.status(), StatusCode::NO_CONTENT);
        match rx.try_recv().unwrap() {
            Command::SetMixerLpf(Some(hz)) => assert!((hz - 4000.0).abs() < 1e-3),
            _ => panic!("expected SetMixerLpf"),
        }
        match rx.try_recv().unwrap() {
            Command::SetMixerHpf(None) => {}
            _ => panic!("expected SetMixerHpf(None)"),
        }

        let app = router(state);
        let res = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/mixer/crossfader")
                    .header("content-type", "application/json")
                    .body(Body::from(r#"{"pos":0.35}"#))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(res.status(), StatusCode::NO_CONTENT);
        match rx.try_recv().unwrap() {
            Command::SetCrossfader(p) => assert!((p - 0.35).abs() < 1e-5),
            _ => panic!("expected SetCrossfader"),
        }
    }

    #[test]
    fn sanitize_path_ok() {
        assert!(sanitize_song_path("songs/smoke.strudel").is_ok());
        assert!(sanitize_song_path("../secret.strudel").is_err());
        assert!(sanitize_song_path("..\\secret.strudel").is_err());
        assert!(sanitize_song_path("songs/../../etc/passwd").is_err());
        assert!(sanitize_song_path("").is_err());
    }

    #[test]
    fn deck_idx_ok() {
        assert_eq!(deck_idx("A").unwrap(), 0);
        assert_eq!(deck_idx("b").unwrap(), 1);
        assert!(deck_idx("C").is_err());
    }
}

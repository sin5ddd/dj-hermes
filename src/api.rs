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
use crate::mixer::{FillKind, MixAction, MixCommand, MixGrid, MixStatus};
use crate::session::SessionKind;
use crate::song::{
    ensure_user_songs_dir, parse_song, resolve_song_path, resolve_user_song_save_path,
    song_listing, Song, Track, MAX_SONG_CONTENT_BYTES,
};
use axum::extract::Query;

// Re-export for callers/tests that used api::sanitize_song_path.
pub use crate::song::sanitize_song_path;

/// Default listen port (10000s; avoids commonly busy 7878).
pub const DEFAULT_API_PORT: u16 = 17878;

/// Environment variable for default port override (`DJ_HERMES_API_PORT`).
pub const ENV_API_PORT: &str = "DJ_HERMES_API_PORT";

/// Environment variable for full base URL (MCP / scripts): e.g. `http://127.0.0.1:17878`.
pub const ENV_API_BASE: &str = "DJ_HERMES_API";

#[derive(Clone)]
pub struct AppState {
    pub tx: Sender<Command>,
    pub engine: Arc<Mutex<Engine>>,
    /// Play hides mix MCP tools and defaults omitted `deck` to A.
    pub session: SessionKind,
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
    /// Active mix macro, if any.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mix: Option<MixStatusInfo>,
}

#[derive(Clone, Debug, Serialize, PartialEq)]
pub struct MixStatusInfo {
    #[serde(rename = "move")]
    pub move_name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub kind: Option<String>,
    pub to: String,
    pub bars_left: u32,
}

impl MixStatusInfo {
    fn from_mixer(s: &MixStatus) -> Self {
        Self {
            move_name: s.move_name.clone(),
            kind: s.kind.clone(),
            to: s.to.clone(),
            bars_left: s.bars_left,
        }
    }
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

/// Save a song under `~/.config/dj-hermes/songs/` only (does not change playback).
#[derive(Deserialize)]
pub struct SaveSongReq {
    /// Basename (e.g. `visitor-dark` or `visitor-dark.strudel`).
    pub name: String,
    /// Full `.strudel` source. Omit (or empty) to snapshot the `deck` source.
    pub content: Option<String>,
    /// Snapshot source when `content` is omitted. Never loads after save.
    pub deck: Option<String>,
    /// Default `true`. When `false`, existing file → 409.
    pub overwrite: Option<bool>,
}

/// Apply full `.strudel` source onto a deck without writing disk.
#[derive(Deserialize)]
pub struct ApplySongReq {
    pub content: String,
    pub deck: String,
}

/// Query for `GET /song`.
#[derive(Deserialize)]
pub struct GetSongQuery {
    pub deck: String,
}

#[derive(Serialize)]
pub struct TrackInfo {
    pub name: String,
    pub code: String,
    pub muted: bool,
}

#[derive(Serialize)]
pub struct GetSongRes {
    pub deck: String,
    pub title: String,
    pub path: String,
    pub source: String,
    pub tracks: Vec<TrackInfo>,
    pub bpm: Option<f64>,
}

#[derive(Deserialize)]
pub struct PatchTrackReq {
    pub deck: String,
    pub track: String,
    /// `replace` | `remove` | `append`
    pub op: String,
    pub code: Option<String>,
    /// Optional new/append track label.
    pub name: Option<String>,
}

#[derive(Deserialize)]
pub struct EditMethodReq {
    pub deck: String,
    pub track: String,
    /// `set` | `add` | `remove`
    pub op: String,
    pub method: String,
    /// Inside-parens args (omit or empty for remove).
    pub args: Option<String>,
}

#[derive(Serialize)]
pub struct SongEditRes {
    pub deck: String,
    pub title: String,
    pub source: String,
    pub tracks: Vec<TrackInfo>,
}

#[derive(Serialize)]
pub struct SaveSongRes {
    pub path: String,
    pub name: String,
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

#[derive(Debug, Deserialize)]
pub struct MixReq {
    /// `long` | `cut` | `fill` | `hold`
    #[serde(rename = "move")]
    pub move_name: String,
    pub to: Option<String>,
    pub bars: Option<u32>,
    pub eq: Option<bool>,
    pub reset_eq: Option<bool>,
    pub kind: Option<String>,
    pub grid: Option<String>,
    pub mute_track: Option<String>,
    pub phrase: Option<u32>,
}

#[derive(Serialize)]
pub struct ErrRes {
    pub error: String,
}

fn bad(e: impl Into<String>) -> (StatusCode, Json<ErrRes>) {
    (StatusCode::BAD_REQUEST, Json(ErrRes { error: e.into() }))
}

fn conflict(e: impl Into<String>) -> (StatusCode, Json<ErrRes>) {
    (StatusCode::CONFLICT, Json(ErrRes { error: e.into() }))
}

pub(crate) fn snapshot(engine: &Arc<Mutex<Engine>>) -> StatusInfo {
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
        mix: e.mixer.mix_status().map(MixStatusInfo::from_mixer),
    }
}

fn parse_phrase(v: u32) -> Result<u32, String> {
    match v {
        1 | 4 | 8 => Ok(v),
        other => Err(format!("phrase must be 1, 4, or 8: {other}")),
    }
}

pub(crate) fn mix_command_from_req(r: &MixReq) -> Result<Command, String> {
    let move_l = r.move_name.trim().to_ascii_lowercase();
    if move_l == "hold" {
        return Ok(Command::HoldXFade);
    }
    let action = MixAction::parse(&r.move_name)?;
    let to_s =
        r.to.as_deref()
            .ok_or("to required (A or B) unless move=hold")?;
    let to_deck = deck_idx(to_s)?;
    let fill = match action {
        MixAction::Fill => {
            let k = r.kind.as_deref().ok_or(
                "kind required for move=fill (delay|lpf|flash|riser|switch|echo|hpf|roll|drop)",
            )?;
            Some(FillKind::parse(k)?)
        }
        _ => None,
    };
    let grid = match r.grid.as_deref() {
        None => MixGrid::Eighth,
        Some(s) => MixGrid::parse(s)?,
    };
    let phrase = parse_phrase(r.phrase.unwrap_or(1))?;
    let bars = r
        .bars
        .unwrap_or_else(|| action.default_bars(fill))
        .clamp(1, 32);
    Ok(Command::Mix(MixCommand {
        action,
        to_deck,
        bars,
        eq: r.eq.unwrap_or(true),
        reset_eq: r.reset_eq.unwrap_or(true),
        fill,
        grid,
        mute_track: r
            .mute_track
            .as_ref()
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty()),
        phrase,
    }))
}

fn mix_requires_both_decks(cmd: &Command) -> bool {
    match cmd {
        Command::Mix(m) if m.action == MixAction::Long => true,
        Command::Mix(m) if m.fill == Some(FillKind::Switch) => true,
        _ => false,
    }
}

fn mix_target_deck(cmd: &Command) -> Option<usize> {
    match cmd {
        Command::Mix(m) => Some(m.to_deck),
        _ => None,
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
            midi_ch: None,
            midi_msb: None,
            midi_lsb: None,
            midi_pc: None,
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

#[derive(Deserialize, Default)]
pub struct ListSongsQuery {
    pub genre: Option<String>,
}

#[derive(Serialize)]
pub struct GenreInfo {
    pub name: String,
    pub count: usize,
}

#[derive(Serialize)]
pub struct ListSongsRes {
    /// Basenames in `~/.config/dj-hermes/songs/` (MCP save target).
    pub user_library: Vec<String>,
    /// Slot refs (`house/01`) when `?genre=` is set; empty in the default listing.
    pub bundled: Vec<String>,
    /// Bundled genre folders and counts.
    pub genres: Vec<GenreInfo>,
    /// How to pass `path` to `/song/load` / `dj_hermes_load_song`.
    pub load_hint: String,
}

/// List loadable songs (user library + bundled genres). `?genre=house` lists numbers.
async fn list_songs(Query(q): Query<ListSongsQuery>) -> Json<ListSongsRes> {
    let listing = song_listing(q.genre.as_deref());
    Json(ListSongsRes {
        user_library: listing.user_library,
        bundled: listing.bundled,
        genres: listing
            .genres
            .into_iter()
            .map(|g| GenreInfo {
                name: g.name,
                count: g.count,
            })
            .collect(),
        load_hint: listing.load_hint.into(),
    })
}

fn song_edit_res(deck: usize, song: &Song) -> SongEditRes {
    SongEditRes {
        deck: if deck == 0 { "A" } else { "B" }.into(),
        title: song.title.clone(),
        source: song.source.clone(),
        tracks: song
            .track_sources()
            .into_iter()
            .map(|t| TrackInfo {
                name: t.name,
                code: t.code,
                muted: t.muted,
            })
            .collect(),
    }
}

/// Snapshot the loaded song source + per-track chains for partial edit workflows.
async fn get_song(
    State(s): State<AppState>,
    Query(q): Query<GetSongQuery>,
) -> Result<Json<GetSongRes>, (StatusCode, Json<ErrRes>)> {
    let deck = deck_idx(&q.deck).map_err(bad)?;
    let e = s.engine.lock().map_err(|e| bad(e.to_string()))?;
    let song = e.decks[deck]
        .song_ref()
        .ok_or_else(|| bad(format!("deck {} has no song loaded", q.deck)))?;
    Ok(Json(GetSongRes {
        deck: if deck == 0 { "A" } else { "B" }.into(),
        title: song.title.clone(),
        path: song.path.clone(),
        source: song.source.clone(),
        tracks: song
            .track_sources()
            .into_iter()
            .map(|t| TrackInfo {
                name: t.name,
                code: t.code,
                muted: t.muted,
            })
            .collect(),
        bpm: song.bpm,
    }))
}

/// Patch one track (replace / remove / append), then load on the next bar.
async fn patch_track(
    State(s): State<AppState>,
    Json(r): Json<PatchTrackReq>,
) -> Result<Json<SongEditRes>, (StatusCode, Json<ErrRes>)> {
    let deck = deck_idx(&r.deck).map_err(bad)?;
    let song = {
        let e = s.engine.lock().map_err(|e| bad(e.to_string()))?;
        e.decks[deck]
            .song_ref()
            .cloned()
            .ok_or_else(|| bad(format!("deck {} has no song loaded", r.deck)))?
    };
    let patched = song
        .patch_track(&r.track, &r.op, r.code.as_deref(), r.name.as_deref())
        .map_err(bad)?;
    let res = song_edit_res(deck, &patched);
    // Control-plane: publish new source immediately so sequential edits compose.
    {
        let mut e = s.engine.lock().map_err(|e| bad(e.to_string()))?;
        e.decks[deck].set_song_data(patched.clone());
    }
    s.tx.send(Command::LoadSong {
        deck,
        song: Box::new(patched),
    })
    .map_err(|e| bad(e.to_string()))?;
    Ok(Json(res))
}

/// Edit one method on a track chain (`set` / `add` / `remove`), then load next bar.
async fn edit_method(
    State(s): State<AppState>,
    Json(r): Json<EditMethodReq>,
) -> Result<Json<SongEditRes>, (StatusCode, Json<ErrRes>)> {
    let deck = deck_idx(&r.deck).map_err(bad)?;
    let song = {
        let e = s.engine.lock().map_err(|e| bad(e.to_string()))?;
        e.decks[deck]
            .song_ref()
            .cloned()
            .ok_or_else(|| bad(format!("deck {} has no song loaded", r.deck)))?
    };
    let patched = song
        .edit_method(&r.track, &r.op, &r.method, r.args.as_deref().unwrap_or(""))
        .map_err(bad)?;
    let res = song_edit_res(deck, &patched);
    {
        let mut e = s.engine.lock().map_err(|e| bad(e.to_string()))?;
        e.decks[deck].set_song_data(patched.clone());
    }
    s.tx.send(Command::LoadSong {
        deck,
        song: Box::new(patched),
    })
    .map_err(|e| bad(e.to_string()))?;
    Ok(Json(res))
}

fn save_content_from_req(
    s: &AppState,
    r: &SaveSongReq,
) -> Result<String, (StatusCode, Json<ErrRes>)> {
    match r.content.as_deref() {
        Some(c) if !c.is_empty() => Ok(c.to_string()),
        _ => {
            let deck_s = r
                .deck
                .as_deref()
                .ok_or_else(|| bad("content or deck required"))?;
            let deck = deck_idx(deck_s).map_err(bad)?;
            let e = s.engine.lock().map_err(|e| bad(e.to_string()))?;
            let song = e.decks[deck]
                .song_ref()
                .ok_or_else(|| bad(format!("deck {deck_s} has no song loaded")))?;
            Ok(song.source.clone())
        }
    }
}

/// Persist a song into the user library (`~/.config/dj-hermes/songs/` only).
/// Does not change playback.
async fn save_song(
    State(s): State<AppState>,
    Json(r): Json<SaveSongReq>,
) -> Result<(StatusCode, Json<SaveSongRes>), (StatusCode, Json<ErrRes>)> {
    let content = save_content_from_req(&s, &r)?;
    if content.len() > MAX_SONG_CONTENT_BYTES {
        return Err(bad(format!(
            "content too large ({} bytes, max {MAX_SONG_CONTENT_BYTES})",
            content.len()
        )));
    }
    let path = resolve_user_song_save_path(&r.name).map_err(bad)?;
    let file_name = path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("song.strudel")
        .to_string();
    let overwrite = r.overwrite.unwrap_or(true);
    if path.is_file() && !overwrite {
        return Err(conflict(format!(
            "song already exists: {} (set overwrite=true to replace)",
            path.display()
        )));
    }

    // Validate before touching the filesystem.
    let path_str = path.to_string_lossy().into_owned();
    let _song = parse_song(&content, &path_str).map_err(bad)?;

    ensure_user_songs_dir().map_err(bad)?;
    std::fs::write(&path, content.as_bytes()).map_err(|e| bad(format!("write: {e}")))?;

    Ok((
        StatusCode::OK,
        Json(SaveSongRes {
            path: path_str,
            name: file_name,
        }),
    ))
}

/// Load full `.strudel` source onto a deck (next bar). Does not write disk.
async fn apply_song(
    State(s): State<AppState>,
    Json(r): Json<ApplySongReq>,
) -> Result<Json<SongEditRes>, (StatusCode, Json<ErrRes>)> {
    if r.content.len() > MAX_SONG_CONTENT_BYTES {
        return Err(bad(format!(
            "content too large ({} bytes, max {MAX_SONG_CONTENT_BYTES})",
            r.content.len()
        )));
    }
    let deck = deck_idx(&r.deck).map_err(bad)?;
    let song = parse_song(&r.content, "").map_err(bad)?;
    {
        let mut e = s.engine.lock().map_err(|e| bad(e.to_string()))?;
        e.decks[deck].set_song_data(song.clone());
    }
    s.tx.send(Command::LoadSong {
        deck,
        song: Box::new(song.clone()),
    })
    .map_err(|e| bad(e.to_string()))?;
    Ok(Json(song_edit_res(deck, &song)))
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

async fn mix(
    State(s): State<AppState>,
    Json(r): Json<MixReq>,
) -> Result<StatusCode, (StatusCode, Json<ErrRes>)> {
    let cmd = mix_command_from_req(&r).map_err(bad)?;
    if !matches!(cmd, Command::HoldXFade) {
        let (a, b) = {
            let e = s.engine.lock().map_err(|e| bad(e.to_string()))?;
            (
                e.decks[0].song_title().is_some(),
                e.decks[1].song_title().is_some(),
            )
        };
        if mix_requires_both_decks(&cmd) && !(a && b) {
            return Err(bad("both decks need a song loaded for long mix / switch"));
        }
        if let Some(to) = mix_target_deck(&cmd) {
            let loaded = if to == 1 { b } else { a };
            if !loaded {
                return Err(bad("target deck has no song loaded"));
            }
        }
    }
    let status = if matches!(cmd, Command::HoldXFade) {
        StatusCode::NO_CONTENT
    } else {
        StatusCode::ACCEPTED
    };
    s.tx.send(cmd).map_err(|e| bad(e.to_string()))?;
    Ok(status)
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
        .route("/song/apply", post(apply_song))
        .route("/song/save", post(save_song))
        .route("/song", get(get_song))
        .route("/song/patch_track", post(patch_track))
        .route("/song/edit_method", post(edit_method))
        .route("/songs", get(list_songs))
        .route("/xfade", post(xfade))
        .route("/bpm", post(set_bpm))
        .route("/mute", post(mute))
        .route("/head", post(head))
        .route("/hush", post(hush))
        .route("/mixer/eq", post(mixer_eq))
        .route("/mixer/filter", post(mixer_filter))
        .route("/mixer/crossfader", post(mixer_crossfader))
        .route("/mix", post(mix))
        .route("/status", get(get_status))
        .route("/events", get(events))
        // Hermes Streamable HTTP MCP (in-process tools → Command channel).
        .route("/mcp", post(crate::mcp::streamable_http_post))
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
#[allow(clippy::await_holding_lock)] // HOME mutex serializes tests that call .await
mod tests {
    use super::*;
    use crate::mixer::{FillKind, MixAction, MixGrid};
    use axum::body::Body;
    use axum::http::{Request, StatusCode};
    use crossbeam::channel::unbounded;
    use http_body_util::BodyExt;
    use tower::ServiceExt;

    fn lock_home() -> std::sync::MutexGuard<'static, ()> {
        crate::song::lock_test_home()
    }

    fn test_state() -> (AppState, crossbeam::channel::Receiver<Command>) {
        let (tx, rx) = unbounded();
        let engine = Arc::new(Mutex::new(Engine::new(44100, 120.0)));
        (
            AppState {
                tx,
                engine,
                session: SessionKind::Dj,
            },
            rx,
        )
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
    async fn mcp_streamable_http_initialize_and_tools_list() {
        let (state, _) = test_state();
        let app = router(state);
        let init_body = r#"{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2025-03-26","capabilities":{},"clientInfo":{"name":"t","version":"0"}}}"#;
        let res = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/mcp")
                    .header("content-type", "application/json")
                    .header("accept", "application/json, text/event-stream")
                    .body(Body::from(init_body))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(res.status(), StatusCode::OK);
        let ct = res
            .headers()
            .get("content-type")
            .and_then(|v| v.to_str().ok())
            .unwrap_or("");
        assert!(ct.starts_with("application/json"), "{ct}");
        let body = json_body(res).await;
        let v: serde_json::Value = serde_json::from_str(&body).unwrap();
        assert_eq!(v["result"]["serverInfo"]["name"], "dj-hermes");
        assert_eq!(v["result"]["protocolVersion"], "2025-03-26");
    }

    #[tokio::test]
    async fn mcp_notification_returns_202() {
        let (state, _) = test_state();
        let app = router(state);
        let body = r#"{"jsonrpc":"2.0","method":"notifications/initialized"}"#;
        let res = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/mcp")
                    .header("content-type", "application/json")
                    .body(Body::from(body))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(res.status(), StatusCode::ACCEPTED);
    }

    #[tokio::test]
    async fn mcp_tools_call_eq_queues_command() {
        let (state, rx) = test_state();
        let app = router(state);
        let body = r#"{"jsonrpc":"2.0","id":9,"method":"tools/call","params":{"name":"dj_hermes_mixer_eq","arguments":{"deck":"B","hi":0.8}}}"#;
        let res = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/mcp")
                    .header("content-type", "application/json")
                    .body(Body::from(body))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(res.status(), StatusCode::OK);
        let body = json_body(res).await;
        let v: serde_json::Value = serde_json::from_str(&body).unwrap();
        assert_eq!(v["result"]["isError"], false);
        match rx.try_recv().unwrap() {
            Command::SetDeckEq { deck, band, value } => {
                assert_eq!(deck, 1);
                assert_eq!(band, 0);
                assert!((value - 0.8).abs() < 1e-5);
            }
            _ => panic!("expected SetDeckEq"),
        }
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
    async fn save_song_writes_user_library_without_load() {
        let _home_guard = lock_home();
        let home = std::env::temp_dir().join(format!("dj_hermes_save_home_{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&home);
        std::fs::create_dir_all(&home).unwrap();
        // Force user_songs_dir under temp home (HOME wins over USERPROFILE).
        std::env::set_var("HOME", &home);

        let (state, rx) = test_state();
        let app = router(state);
        let content = "// @title save-test\nsetcpm(30)\n$: s(\"bd*4\").gain(0.9)\n";
        let body = serde_json::json!({
            "name": "visitor-dark",
            "content": content,
            "deck": "B",
            "overwrite": true
        })
        .to_string();
        let res = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/song/save")
                    .header("content-type", "application/json")
                    .body(Body::from(body))
                    .unwrap(),
            )
            .await
            .unwrap();
        let status = res.status();
        let body_text = json_body(res).await;
        assert_eq!(status, StatusCode::OK, "{body_text}");
        let expected = home
            .join(".config")
            .join("dj-hermes")
            .join("songs")
            .join("visitor-dark.strudel");
        assert!(expected.is_file(), "{}", expected.display());
        let written = std::fs::read_to_string(&expected).unwrap();
        assert!(written.contains("save-test"), "{written}");
        assert!(rx.try_recv().is_err(), "save must not queue LoadSong");

        // Traversal rejected
        let (state, rx) = test_state();
        let app = router(state);
        let bad = r#"{"name":"../escape","content":"setcpm(30)\n$: s(\"bd\")\n"}"#;
        let res = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/song/save")
                    .header("content-type", "application/json")
                    .body(Body::from(bad))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(res.status(), StatusCode::BAD_REQUEST);
        assert!(rx.try_recv().is_err());

        // Bad parse: no write of invalid path name that would replace — use new name
        let (state, rx) = test_state();
        let app = router(state);
        let bad_body = r#"{"name":"broken","content":"note(\"","overwrite":true}"#;
        let res = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/song/save")
                    .header("content-type", "application/json")
                    .body(Body::from(bad_body))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(res.status(), StatusCode::BAD_REQUEST);
        assert!(rx.try_recv().is_err());
        let broken = home
            .join(".config")
            .join("dj-hermes")
            .join("songs")
            .join("broken.strudel");
        assert!(!broken.is_file(), "must not write on parse error");

        let _ = std::fs::remove_dir_all(&home);
    }

    #[tokio::test]
    async fn apply_song_queues_load_without_write() {
        let _home_guard = lock_home();
        let home =
            std::env::temp_dir().join(format!("dj_hermes_apply_home_{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&home);
        std::fs::create_dir_all(&home).unwrap();
        std::env::set_var("HOME", &home);

        let (state, rx) = test_state();
        let app = router(state.clone());
        let content = "// @title mem\nsetcpm(30)\n$: s(\"bd*4\")\n";
        let body = serde_json::json!({ "content": content, "deck": "A" }).to_string();
        let res = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/song/apply")
                    .header("content-type", "application/json")
                    .body(Body::from(body))
                    .unwrap(),
            )
            .await
            .unwrap();
        let status = res.status();
        let text = json_body(res).await;
        assert_eq!(status, StatusCode::OK, "{text}");
        match rx.try_recv().expect("LoadSong") {
            Command::LoadSong { deck, song } => {
                assert_eq!(deck, 0);
                assert_eq!(song.title, "mem");
            }
            _ => panic!("expected LoadSong"),
        }
        let lib = home.join(".config").join("dj-hermes").join("songs");
        if lib.is_dir() {
            let leftover: Vec<_> = std::fs::read_dir(&lib)
                .unwrap()
                .filter_map(|e| e.ok())
                .filter(|e| e.path().extension().and_then(|x| x.to_str()) == Some("strudel"))
                .collect();
            assert!(leftover.is_empty(), "apply must not write: {leftover:?}");
        }

        let app = router(state);
        let res = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/song/apply")
                    .header("content-type", "application/json")
                    .body(Body::from(r#"{"content":"note(\"","deck":"A"}"#))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(res.status(), StatusCode::BAD_REQUEST);
        assert!(rx.try_recv().is_err(), "must not queue on parse error");

        let _ = std::fs::remove_dir_all(&home);
    }

    #[tokio::test]
    async fn save_song_snapshots_deck_without_load() {
        let _home_guard = lock_home();
        let home = std::env::temp_dir().join(format!("dj_hermes_snap_home_{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&home);
        std::fs::create_dir_all(&home).unwrap();
        std::env::set_var("HOME", &home);

        let (state, rx) = test_state();
        let content = "// @title snap-src\nsetcpm(30)\n$: s(\"bd*4\")\n";
        let app = router(state.clone());
        let res = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/song/apply")
                    .header("content-type", "application/json")
                    .body(Body::from(
                        serde_json::json!({ "content": content, "deck": "A" }).to_string(),
                    ))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(res.status(), StatusCode::OK);
        let _ = rx.try_recv();

        let app = router(state);
        let res = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/song/save")
                    .header("content-type", "application/json")
                    .body(Body::from(r#"{"name":"snap-mem","deck":"A"}"#))
                    .unwrap(),
            )
            .await
            .unwrap();
        let status = res.status();
        let text = json_body(res).await;
        assert_eq!(status, StatusCode::OK, "{text}");
        let expected = home
            .join(".config")
            .join("dj-hermes")
            .join("songs")
            .join("snap-mem.strudel");
        let written = std::fs::read_to_string(&expected).unwrap();
        assert!(written.contains("snap-src"), "{written}");
        assert!(rx.try_recv().is_err(), "snapshot save must not LoadSong");

        let _ = std::fs::remove_dir_all(&home);
    }

    #[tokio::test]
    async fn patch_track_save_true_does_not_write() {
        let _home_guard = lock_home();
        let home =
            std::env::temp_dir().join(format!("dj_hermes_patch_home_{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&home);
        std::fs::create_dir_all(&home).unwrap();
        std::env::set_var("HOME", &home);

        let (state, rx) = test_state();
        let content = "// @title p\nsetcpm(30)\n$: s(\"bd*4\")\n";
        let app = router(state.clone());
        let res = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/song/apply")
                    .header("content-type", "application/json")
                    .body(Body::from(
                        serde_json::json!({ "content": content, "deck": "A" }).to_string(),
                    ))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(res.status(), StatusCode::OK);
        let _ = rx.try_recv();

        let app = router(state);
        let body = serde_json::json!({
            "deck": "A",
            "track": "0",
            "op": "replace",
            "code": "s(\"bd*4\").gain(0.5)",
            "save": true
        })
        .to_string();
        let res = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/song/patch_track")
                    .header("content-type", "application/json")
                    .body(Body::from(body))
                    .unwrap(),
            )
            .await
            .unwrap();
        let status = res.status();
        let text = json_body(res).await;
        assert_eq!(status, StatusCode::OK, "{text}");
        let lib = home.join(".config").join("dj-hermes").join("songs");
        if lib.is_dir() {
            let leftover: Vec<_> = std::fs::read_dir(&lib)
                .unwrap()
                .filter_map(|e| e.ok())
                .collect();
            assert!(leftover.is_empty(), "patch must not write: {leftover:?}");
        }

        let _ = std::fs::remove_dir_all(&home);
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

    #[tokio::test]
    async fn mix_hold_no_song_ok() {
        let (state, rx) = test_state();
        let app = router(state);
        let res = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/mix")
                    .header("content-type", "application/json")
                    .body(Body::from(r#"{"move":"hold"}"#))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(res.status(), StatusCode::NO_CONTENT);
        assert!(matches!(rx.try_recv().unwrap(), Command::HoldXFade));
    }

    #[tokio::test]
    async fn mix_long_requires_both_decks() {
        let (state, rx) = test_state();
        let app = router(state);
        let res = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/mix")
                    .header("content-type", "application/json")
                    .body(Body::from(r#"{"move":"long","to":"B"}"#))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(res.status(), StatusCode::BAD_REQUEST);
        assert!(rx.try_recv().is_err());
    }

    #[tokio::test]
    async fn mix_fill_switch_queues() {
        let (state, rx) = test_state();
        {
            let song = parse_song(
                r#"---
b: note("c3").s("sawtooth").gain(0.8)
"#,
                "t",
            )
            .unwrap();
            let mut e = state.engine.lock().unwrap();
            e.load_song_immediate(0, song.clone());
            e.load_song_immediate(1, song);
        }
        let app = router(state);
        let res = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/mix")
                    .header("content-type", "application/json")
                    .body(Body::from(
                        r#"{"move":"fill","kind":"switch","to":"A","grid":"8n"}"#,
                    ))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(res.status(), StatusCode::ACCEPTED);
        match rx.try_recv().unwrap() {
            Command::Mix(m) => {
                assert_eq!(m.action, MixAction::Fill);
                assert_eq!(m.fill, Some(FillKind::Switch));
                assert_eq!(m.to_deck, 0);
                assert_eq!(m.grid, MixGrid::Eighth);
            }
            _ => panic!("expected Mix"),
        }
    }

    #[tokio::test]
    async fn mix_fill_echo_queues() {
        let (state, rx) = test_state();
        {
            let song = parse_song(
                r#"---
b: note("c3").s("sawtooth").gain(0.8)
"#,
                "t",
            )
            .unwrap();
            let mut e = state.engine.lock().unwrap();
            e.load_song_immediate(0, song.clone());
            e.load_song_immediate(1, song);
        }
        let app = router(state);
        let res = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/mix")
                    .header("content-type", "application/json")
                    .body(Body::from(r#"{"move":"fill","kind":"echo","to":"B"}"#))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(res.status(), StatusCode::ACCEPTED);
        match rx.try_recv().unwrap() {
            Command::Mix(m) => {
                assert_eq!(m.action, MixAction::Fill);
                assert_eq!(m.fill, Some(FillKind::Echo));
                assert_eq!(m.to_deck, 1);
            }
            _ => panic!("expected Mix"),
        }
    }

    #[test]
    fn sanitize_path_ok() {
        assert!(sanitize_song_path("songs/house-01.strudel").is_ok());
        assert!(sanitize_song_path("songs/house/01.strudel").is_ok());
        assert!(sanitize_song_path("house/01").is_ok());
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

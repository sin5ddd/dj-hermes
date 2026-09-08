//! MCP server for dj-hermes.
//!
//! **Primary (Hermes):** Streamable HTTP on the play/dj API — `POST /mcp`.
//! Tools run in-process (Command channel + engine snapshot). No audio devices.
//!
//! **Deprecated (debug):** `dj-hermes mcp` stdio bridge → REST (`DJ_HERMES_API`).
//! Prefer HTTP; keep stdio only for manual NDJSON smoke tests.
//!
//! Stdio framing (deprecated path):
//! - **NDJSON** — one JSON object per line
//! - **Content-Length** — LSP-style headers + body
//!
//! Implemented by hand (not `rmcp`) to stay small. See plan Risk #6.

use std::io::{self, BufRead, Read, Write};

use axum::extract::State;
use axum::http::{header, StatusCode};
use axum::response::{IntoResponse, Response};
use serde_json::{json, Map, Value};

use crate::api::{deck_idx, default_api_base, snapshot, AppState, StatusInfo, DEFAULT_API_PORT};
use crate::engine::Command;
use crate::session::SessionKind;
use crate::song::{
    ensure_user_songs_dir, is_genre_slug, parse_song, resolve_song_path,
    resolve_user_song_save_path, song_listing, MAX_SONG_CONTENT_BYTES,
};

const PROTOCOL_VERSION: &str = "2025-03-26";
const SERVER_NAME: &str = "dj-hermes";
const SERVER_VERSION: &str = env!("CARGO_PKG_VERSION");

/// Wire framing for one MCP stdio session (deprecated debug path).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Framing {
    /// `{"jsonrpc":...}\n`
    Ndjson,
    /// `Content-Length: N\r\n\r\n{...}`
    ContentLength,
}

/// How tools execute for one RPC session.
enum ToolBackend<'a> {
    /// Stdio bridge: HTTP to play process REST API.
    Http {
        client: &'a reqwest::blocking::Client,
        base: &'a str,
    },
    /// In-process: same process as play/dj API.
    Local { state: &'a AppState },
}

impl ToolBackend<'_> {
    fn session(&self) -> SessionKind {
        match self {
            ToolBackend::Local { state } => state.session,
            // Stdio debug bridge lists the full DJ tool set.
            ToolBackend::Http { .. } => SessionKind::Dj,
        }
    }
}

const MIX_TOOL_NAMES: &[&str] = &[
    "dj_hermes_mixer_eq",
    "dj_hermes_mixer_filter",
    "dj_hermes_mixer_fx",
    "dj_hermes_mixer_crossfader",
    "dj_hermes_xfade",
    "dj_hermes_mix",
];

fn is_mix_tool(name: &str) -> bool {
    MIX_TOOL_NAMES.contains(&name)
}

// ── Streamable HTTP (primary) ───────────────────────────────────────────────

/// `POST /mcp` — MCP Streamable HTTP (JSON request → JSON response).
///
/// Notifications (no `id`) → 202 Accepted with empty body.
/// Requests → 200 `application/json` JSON-RPC response.
pub async fn streamable_http_post(
    State(state): State<AppState>,
    body: axum::body::Bytes,
) -> Response {
    let msg: Value = match serde_json::from_slice(&body) {
        Ok(v) => v,
        Err(e) => {
            return (
                StatusCode::BAD_REQUEST,
                [(header::CONTENT_TYPE, "application/json")],
                json!({
                    "jsonrpc": "2.0",
                    "id": null,
                    "error": { "code": -32700, "message": format!("parse error: {e}") }
                })
                .to_string(),
            )
                .into_response();
        }
    };

    let backend = ToolBackend::Local { state: &state };
    match handle_rpc(&msg, &backend) {
        None => StatusCode::ACCEPTED.into_response(),
        Some(response) => (
            StatusCode::OK,
            [(header::CONTENT_TYPE, "application/json")],
            response.to_string(),
        )
            .into_response(),
    }
}

// ── Stdio bridge (deprecated) ───────────────────────────────────────────────

/// Run MCP server until stdin EOF. Blocks the calling thread.
///
/// **Deprecated:** use Hermes `url: http://127.0.0.1:17878/mcp` against a running
/// play/dj process. Kept for manual `printf | dj-hermes mcp` debugging.
pub fn run() -> Result<(), String> {
    eprintln!("dj-hermes mcp: stdio bridge is deprecated; prefer POST /mcp on the play API");
    let base = default_api_base();
    let client = reqwest::blocking::Client::builder()
        .timeout(std::time::Duration::from_secs(10))
        .build()
        .map_err(|e| format!("http client: {e}"))?;

    let stdin = io::stdin();
    let mut stdin = stdin.lock();
    let mut stdout = io::stdout();

    let mut framing: Option<Framing> = None;

    while let Some((msg, detected)) = read_message(&mut stdin)? {
        if framing.is_none() {
            framing = Some(detected);
        }
        let mode = framing.unwrap_or(detected);
        let backend = ToolBackend::Http {
            client: &client,
            base: &base,
        };
        if let Some(response) = handle_rpc(&msg, &backend) {
            write_message(&mut stdout, &response, mode)?;
        }
    }
    Ok(())
}

// ── RPC core ────────────────────────────────────────────────────────────────

/// Handle one JSON-RPC message. Returns `None` for notifications (no reply).
fn handle_rpc(msg: &Value, backend: &ToolBackend<'_>) -> Option<Value> {
    let id = msg.get("id").cloned();
    let method = msg
        .get("method")
        .and_then(|m| m.as_str())
        .unwrap_or("")
        .to_string();

    // notifications/initialized etc. — no response without id
    id.as_ref()?;

    let result = match method.as_str() {
        "initialize" => {
            let params = msg.get("params").cloned().unwrap_or(json!({}));
            Ok(initialize_result(&params))
        }
        "ping" => Ok(json!({})),
        "tools/list" => Ok(tools_list(backend.session())),
        "tools/call" => {
            let params = msg.get("params").cloned().unwrap_or(json!({}));
            tools_call(backend, params)
        }
        "resources/list" => Ok(json!({ "resources": [] })),
        "prompts/list" => Ok(json!({ "prompts": [] })),
        other => Err(rpc_error(-32601, format!("method not found: {other}"))),
    };

    Some(match result {
        Ok(value) => json!({
            "jsonrpc": "2.0",
            "id": id,
            "result": value,
        }),
        Err(err_obj) => json!({
            "jsonrpc": "2.0",
            "id": id,
            "error": err_obj,
        }),
    })
}

fn initialize_result(params: &Value) -> Value {
    // Echo a version the client understands when possible.
    let requested = params
        .get("protocolVersion")
        .and_then(|v| v.as_str())
        .unwrap_or(PROTOCOL_VERSION);
    let version = if requested.starts_with("2024") || requested.starts_with("2025") {
        requested
    } else {
        PROTOCOL_VERSION
    };
    json!({
        "protocolVersion": version,
        "capabilities": {
            "tools": { "listChanged": false }
        },
        "serverInfo": {
            "name": SERVER_NAME,
            "version": SERVER_VERSION
        }
    })
}

fn tools_list(session: SessionKind) -> Value {
    // Order: Mixer → Deck → Transport
    let mut v = json!({
        "tools": [
            {
                "name": "dj_hermes_mixer_eq",
                "description": "Mixer: set deck A/B channel EQ (Hi/Mid/Lo). Values 0..=1, 1.0=0 dB (right/flat), 0=band kill. No boost. Immediate. Provide at least one of hi/mid/lo.",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "deck": { "type": "string", "description": "A or B" },
                        "hi": { "type": "number", "description": "High 0..=1 (1.0 = 0 dB, 0 = kill)" },
                        "mid": { "type": "number", "description": "Mid 0..=1 (1.0 = 0 dB, 0 = kill)" },
                        "lo": { "type": "number", "description": "Low 0..=1 (1.0 = 0 dB, 0 = kill)" }
                    },
                    "required": ["deck"]
                }
            },
            {
                "name": "dj_hermes_mixer_filter",
                "description": "Mixer: post-mix LPF/HPF (held). Pass Hz number to set, or null to bypass. Immediate. Fill jobs overlay then restore this. Provide at least one of lpf/hpf.",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "lpf": { "description": "Low-pass cutoff Hz, or null to bypass", "anyOf": [ {"type": "number"}, {"type": "null"} ] },
                        "hpf": { "description": "High-pass cutoff Hz, or null to bypass", "anyOf": [ {"type": "number"}, {"type": "null"} ] }
                    }
                }
            },
            {
                "name": "dj_hermes_mixer_fx",
                "description": "Mixer: post-mix delay wet on the mixed line (held). Immediate. Fill delay/echo overlay then restore this. delay=0 bypasses.",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "delay": { "type": "number", "description": "Wet 0..=1" },
                        "feedback": { "type": "number", "description": "Feedback 0..=1 (optional)" }
                    },
                    "required": ["delay"]
                }
            },
            {
                "name": "dj_hermes_mixer_repeat",
                "description": "Mixer: time-repeat playhead (quarter/eighth/16th/32nd notes, not minutes). Immediate. Loops the current note-value slice for one absolute bar then off. Hits and MIDI follow the relative playhead. div=null or off clears. Not fill kind=roll (PCM then cut-in).",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "div": { "description": "4n | 8n | 16n | 32n, or null to clear", "anyOf": [ {"type": "string"}, {"type": "null"} ] }
                    }
                }
            },
            {
                "name": "dj_hermes_mixer_tape",
                "description": "Mixer: tape-stop. Immediate. Slows the mixed output (post compressor) and the playhead over len (1n=whole/1 bar, 2n=half, 4n=quarter, 8n=eighth), then snaps back. reps=2 runs that shot twice in a row (hype double-stab). on=false cancels now (then mix cut if switching songs). Mutually exclusive with mixer_repeat (last-wins). --midi-only: fewer Note Ons, SEQTRAK pitch does not drop.",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "on": { "type": "boolean", "description": "true starts, false clears" },
                        "len": { "type": "string", "description": "1n | 2n | 4n | 8n (default 1n)" },
                        "reps": { "type": "integer", "description": "consecutive shots 1..=8 (default 1)" }
                    },
                    "required": ["on"]
                }
            },
            {
                "name": "dj_hermes_mixer_crossfader",
                "description": "Mixer: set equal-power crossfader position immediately (0=full A, 1=full B). Cancels multi-bar xfade animation.",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "pos": { "type": "number", "description": "0..=1" }
                    },
                    "required": ["pos"]
                }
            },
            {
                "name": "dj_hermes_xfade",
                "description": "Mixer: crossfade to deck A or B over N bars (starts next bar).",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "to": { "type": "string", "description": "A or B" },
                        "bars": { "type": "integer", "description": "default 4" }
                    },
                    "required": ["to"]
                }
            },
            {
                "name": "dj_hermes_mix",
                "description": "DJ mix move in one call. Prefer this over calling mixer_eq multiple times. move=long: EQ bass-swap + xfade. move=cut: next-bar 100% fader, optional EQ reset. move=fill: delay|lpf|flash|riser|switch|echo|hpf|roll|drop then cut-in. move=hold: freeze xfade. Switch is AB 100:0 chops (not flash). echo=delay wet/fb ramp then cut. hpf=high-pass sweep then cut. roll=beat-repeat then cut. drop=impact one-shot then cut.",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "move": { "type": "string", "description": "long | cut | fill | hold" },
                        "to": { "type": "string", "description": "A or B (required unless hold)" },
                        "bars": { "type": "integer", "description": "long default 8, fill default 1 (riser 8)" },
                        "eq": { "type": "boolean", "description": "long: apply bass-swap EQ (default true)" },
                        "reset_eq": { "type": "boolean", "description": "cut/fill: flatten EQ at the end (default true)" },
                        "kind": { "type": "string", "description": "fill only: delay | lpf | flash | riser | switch | echo | hpf | roll | drop" },
                        "grid": { "type": "string", "description": "switch/flash/roll: 8n or 4n (default 8n)" },
                        "mute_track": { "type": "string", "description": "optional track name to mute on the outgoing deck" },
                        "phrase": { "type": "integer", "description": "1, 4, or 8 — start on that bar boundary (default 1)" }
                    },
                    "required": ["move"]
                }
            },
            {
                "name": "dj_hermes_set_bpm",
                "description": "Mixer: set master BPM (applies next bar).",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "bpm": { "type": "number" }
                    },
                    "required": ["bpm"]
                }
            },
            {
                "name": "dj_hermes_load_song",
                "description": "Deck: load a song onto a deck (next bar). Bundled demos: path=\"house/01\" or legacy \"house-01\". User library: bare basename (visitor-dnb). Searches ~/.config/dj-hermes/songs/ first, then songs/<genre>/<nn>.strudel. After dj_hermes_save_song, load with the same basename (no songs/ prefix). Example: path=\"house/01\", deck=\"A\".",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "path": {
                            "type": "string",
                            "description": "house/01, house-01, or user-library basename visitor-dnb"
                        },
                        "deck": { "type": "string", "description": "A or B" }
                    },
                    "required": ["path", "deck"]
                }
            },
            {
                "name": "dj_hermes_apply_song",
                "description": "Deck: parse full .strudel source and load onto a deck (next bar). Does NOT write disk. Use for new songs and large rewrites. Persist with dj_hermes_save_song only when asked to keep the song. content MUST use setcpm (or setcps) and one or more `$:` track lines — never stack(...), never .cpm().",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "content": {
                            "type": "string",
                            "description": "Full song source: setcpm(N) or setcpm(BPM/4), then lines `$: <chain>` (max 256KiB). No stack(), no .cpm()."
                        },
                        "deck": { "type": "string", "description": "A or B" }
                    },
                    "required": ["content", "deck"]
                }
            },
            {
                "name": "dj_hermes_list_songs",
                "description": "Deck: list songs. Default returns genres (name + count) and user_library. Pass genre=\"house\" to list bundled slot refs (house/01) in bundled. Use those with dj_hermes_load_song.",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "genre": {
                            "type": "string",
                            "description": "Optional genre folder (house, chill-pop). When set, bundled lists house/01 …"
                        }
                    }
                }
            },
            {
                "name": "dj_hermes_save_song",
                "description": "Deck: persist a .strudel song into the user library ONLY (~/.config/dj-hermes/songs/<name>.strudel). Basename only (no paths). Validates before write. Does not change playback. To play, use dj_hermes_apply_song or dj_hermes_load_song. If content omitted, deck is required and the current deck source is written. content MUST use setcpm (or setcps) and one or more `$:` track lines — never stack(...), never .cpm().",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "name": {
                            "type": "string",
                            "description": "Basename e.g. visitor-dark or visitor-dark.strudel (ASCII letters/digits/._- only)"
                        },
                        "content": {
                            "type": "string",
                            "description": "Optional full song source: setcpm(N) or setcpm(BPM/4), then lines `$: <chain>` (max 256KiB). Omit to snapshot deck. No stack(), no .cpm()."
                        },
                        "deck": {
                            "type": "string",
                            "description": "A or B — required when content is omitted (snapshot only; does not load)"
                        },
                        "overwrite": {
                            "type": "boolean",
                            "description": "Default true; false → fail if file exists"
                        }
                    },
                    "required": ["name"]
                }
            },
            {
                "name": "dj_hermes_get_song",
                "description": "Deck: read the currently loaded song on a deck (full source + per-track chains). Use before dj_hermes_edit_method / dj_hermes_patch_track so other parts are not rewritten.",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "deck": { "type": "string", "description": "A or B" }
                    },
                    "required": ["deck"]
                }
            },
            {
                "name": "dj_hermes_patch_track",
                "description": "Deck: replace/remove/append one `$:` track on the loaded song (other tracks preserved). Applies next bar. Prefer this or dj_hermes_edit_method over full dj_hermes_apply_song for live edits.",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "deck": { "type": "string", "description": "A or B" },
                        "track": { "type": "string", "description": "Track name (e.g. bass) or 0-based index" },
                        "op": { "type": "string", "description": "replace | remove | append" },
                        "code": { "type": "string", "description": "Full track chain for replace/append, e.g. note(\"c2\").s(\"saw\").lpf(400)" },
                        "name": { "type": "string", "description": "Optional label for append/rename on replace" }
                    },
                    "required": ["deck", "track", "op"]
                }
            },
            {
                "name": "dj_hermes_edit_method",
                "description": "Deck: set/add/remove one method on a track chain (e.g. method=lpf, args=400 or sine.rangex(500,4000)). Same-name methods: last wins for set/remove. Applies next bar. Prefer over rewriting the whole song.",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "deck": { "type": "string", "description": "A or B" },
                        "track": { "type": "string", "description": "Track name or 0-based index" },
                        "op": { "type": "string", "description": "set | add | remove" },
                        "method": { "type": "string", "description": "Method name without dot, e.g. lpf, gain, scale" },
                        "args": { "type": "string", "description": "Inside-parens args (omit for remove)" },

                    },
                    "required": ["deck", "track", "op", "method"]
                }
            },
            {
                "name": "dj_hermes_mute",
                "description": "Deck: mute or unmute a track on a deck (next bar).",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "deck": { "type": "string" },
                        "track": { "type": "string" },
                        "muted": { "type": "boolean" }
                    },
                    "required": ["deck", "track", "muted"]
                }
            },
            {
                "name": "dj_hermes_head",
                "description": "Deck: cue a deck to a 1-based song bar at the next transport bar boundary (DJ head-out). REPL: `b head 33`.",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "deck": { "type": "string", "description": "A or B" },
                        "bar": { "type": "integer", "description": "Song bar number (1 = first bar)" }
                    },
                    "required": ["deck", "bar"]
                }
            },
            {
                "name": "dj_hermes_hush",
                "description": "Transport: stop all sound immediately.",
                "inputSchema": {
                    "type": "object",
                    "properties": {}
                }
            },
            {
                "name": "dj_hermes_status",
                "description": "Transport: get decks, BPM, bars, mixer gains, EQ (eq_a/eq_b), filters, delay_wet, repeat, tape, crossfader, muted_a/muted_b.",
                "inputSchema": {
                    "type": "object",
                    "properties": {}
                }
            }
        ]
    });
    if session == SessionKind::Play {
        if let Some(arr) = v["tools"].as_array_mut() {
            arr.retain(|t| t["name"].as_str().map(|n| !is_mix_tool(n)).unwrap_or(true));
            for t in arr.iter_mut() {
                play_relax_deck_schema(t);
            }
        }
    }
    v
}

/// Play: `deck` is optional (always A). Do not use `Value` IndexMut here —
/// missing keys would be inserted as JSON `null`, and xAI rejects
/// `"required": null` (`/required: null is not of type "array"`).
fn play_relax_deck_schema(tool: &mut Value) {
    let Some(schema) = tool.get_mut("inputSchema").and_then(Value::as_object_mut) else {
        return;
    };
    let drop_required = schema
        .get_mut("required")
        .and_then(Value::as_array_mut)
        .map(|req| {
            req.retain(|x| x.as_str() != Some("deck"));
            req.is_empty()
        })
        .unwrap_or(false);
    if drop_required {
        schema.remove("required");
    }
    if let Some(deck) = schema
        .get_mut("properties")
        .and_then(Value::as_object_mut)
        .and_then(|p| p.get_mut("deck"))
        .and_then(Value::as_object_mut)
    {
        deck.insert(
            "description".into(),
            json!("optional in play; always deck A"),
        );
    }
}

fn tools_call(backend: &ToolBackend<'_>, params: Value) -> Result<Value, Value> {
    let name = params
        .get("name")
        .and_then(|v| v.as_str())
        .ok_or_else(|| rpc_error(-32602, "missing tool name"))?;
    let args = params
        .get("arguments")
        .cloned()
        .unwrap_or_else(|| json!({}));

    match backend {
        ToolBackend::Http { client, base } => tools_call_http(client, base, name, &args),
        ToolBackend::Local { state } => tools_call_local(state, name, &args),
    }
}

// ── HTTP bridge tool exec (stdio) ───────────────────────────────────────────

fn tools_call_http(
    client: &reqwest::blocking::Client,
    base: &str,
    name: &str,
    args: &Value,
) -> Result<Value, Value> {
    let outcome = match name {
        "dj_hermes_mixer_eq" => {
            let deck = arg_str(args, "deck")?;
            let mut body = Map::new();
            body.insert("deck".into(), json!(deck));
            for key in ["hi", "mid", "lo"] {
                if let Some(v) = args.get(key) {
                    body.insert(key.into(), v.clone());
                }
            }
            if !["hi", "mid", "lo"].iter().any(|k| body.contains_key(*k)) {
                return Err(rpc_error(
                    -32602,
                    "mixer_eq: provide at least one of hi, mid, lo",
                ));
            }
            http_post(client, &format!("{base}/mixer/eq"), Value::Object(body))
        }
        "dj_hermes_mixer_filter" => {
            let mut body = Map::new();
            for key in ["lpf", "hpf"] {
                if let Some(v) = args.get(key) {
                    body.insert(key.into(), v.clone());
                }
            }
            if body.is_empty() {
                return Err(rpc_error(
                    -32602,
                    "mixer_filter: provide lpf and/or hpf (number or null)",
                ));
            }
            http_post(client, &format!("{base}/mixer/filter"), Value::Object(body))
        }
        "dj_hermes_mixer_fx" => {
            let delay = args
                .get("delay")
                .and_then(|v| v.as_f64())
                .ok_or_else(|| rpc_error(-32602, "delay required (0..=1)"))?;
            let mut body = Map::new();
            body.insert("delay".into(), json!(delay));
            if let Some(v) = args.get("feedback") {
                body.insert("feedback".into(), v.clone());
            }
            http_post(client, &format!("{base}/mixer/fx"), Value::Object(body))
        }
        "dj_hermes_mixer_repeat" => {
            let mut body = Map::new();
            match args.get("div") {
                None | Some(Value::Null) => {
                    body.insert("div".into(), Value::Null);
                }
                Some(v) => {
                    body.insert("div".into(), v.clone());
                }
            }
            http_post(client, &format!("{base}/mixer/repeat"), Value::Object(body))
        }
        "dj_hermes_mixer_tape" => {
            let on = args
                .get("on")
                .and_then(|v| v.as_bool())
                .ok_or_else(|| rpc_error(-32602, "on required (boolean)"))?;
            let mut body = Map::new();
            body.insert("on".into(), json!(on));
            if let Some(v) = args.get("len") {
                body.insert("len".into(), v.clone());
            }
            if let Some(v) = args.get("reps") {
                body.insert("reps".into(), v.clone());
            }
            http_post(client, &format!("{base}/mixer/tape"), Value::Object(body))
        }
        "dj_hermes_mixer_crossfader" => {
            let pos = args
                .get("pos")
                .and_then(|v| v.as_f64())
                .ok_or_else(|| rpc_error(-32602, "pos required (0..=1)"))?;
            http_post(
                client,
                &format!("{base}/mixer/crossfader"),
                json!({ "pos": pos }),
            )
        }
        "dj_hermes_xfade" => {
            let to = arg_str(args, "to")?;
            let bars = args.get("bars").and_then(|v| v.as_u64()).unwrap_or(4);
            http_post(
                client,
                &format!("{base}/xfade"),
                json!({ "to": to, "bars": bars }),
            )
        }
        "dj_hermes_mix" => {
            let mut body = Map::new();
            let mv = arg_str(args, "move")?;
            body.insert("move".into(), json!(mv));
            for key in [
                "to",
                "bars",
                "eq",
                "reset_eq",
                "kind",
                "grid",
                "mute_track",
                "phrase",
            ] {
                if let Some(v) = args.get(key) {
                    body.insert(key.into(), v.clone());
                }
            }
            http_post(client, &format!("{base}/mix"), Value::Object(body))
        }
        "dj_hermes_set_bpm" => {
            let bpm = args
                .get("bpm")
                .and_then(|v| v.as_f64())
                .ok_or_else(|| rpc_error(-32602, "bpm required"))?;
            http_post(client, &format!("{base}/bpm"), json!({ "bpm": bpm }))
        }
        "dj_hermes_load_song" => {
            let path = arg_str(args, "path")?;
            let deck = arg_str(args, "deck")?;
            http_post(
                client,
                &format!("{base}/song/load"),
                json!({ "path": path, "deck": deck }),
            )
        }
        "dj_hermes_apply_song" => {
            let content = arg_str(args, "content")?;
            let deck = arg_str(args, "deck")?;
            http_post(
                client,
                &format!("{base}/song/apply"),
                json!({ "content": content, "deck": deck }),
            )
        }
        "dj_hermes_list_songs" => {
            let url = match args.get("genre").and_then(|v| v.as_str()) {
                Some(g) if is_genre_slug(g) => format!("{base}/songs?genre={g}"),
                _ => format!("{base}/songs"),
            };
            http_get(client, &url)
        }
        "dj_hermes_save_song" => {
            let name = arg_str(args, "name")?;
            let mut body = Map::new();
            body.insert("name".into(), json!(name));
            if let Some(content) = args.get("content").and_then(|v| v.as_str()) {
                body.insert("content".into(), json!(content));
            }
            if let Some(deck) = args.get("deck").and_then(|v| v.as_str()) {
                body.insert("deck".into(), json!(deck));
            }
            if let Some(ow) = args.get("overwrite").and_then(|v| v.as_bool()) {
                body.insert("overwrite".into(), json!(ow));
            }
            http_post(client, &format!("{base}/song/save"), Value::Object(body))
        }
        "dj_hermes_get_song" => {
            let deck = arg_str(args, "deck")?;
            http_get(client, &format!("{base}/song?deck={deck}"))
        }
        "dj_hermes_patch_track" => {
            let deck = arg_str(args, "deck")?;
            let track = arg_str(args, "track")?;
            let op = arg_str(args, "op")?;
            let mut body = Map::new();
            body.insert("deck".into(), json!(deck));
            body.insert("track".into(), json!(track));
            body.insert("op".into(), json!(op));
            if let Some(code) = args.get("code").and_then(|v| v.as_str()) {
                body.insert("code".into(), json!(code));
            }
            if let Some(name) = args.get("name").and_then(|v| v.as_str()) {
                body.insert("name".into(), json!(name));
            }
            http_post(
                client,
                &format!("{base}/song/patch_track"),
                Value::Object(body),
            )
        }
        "dj_hermes_edit_method" => {
            let deck = arg_str(args, "deck")?;
            let track = arg_str(args, "track")?;
            let op = arg_str(args, "op")?;
            let method = arg_str(args, "method")?;
            let mut body = Map::new();
            body.insert("deck".into(), json!(deck));
            body.insert("track".into(), json!(track));
            body.insert("op".into(), json!(op));
            body.insert("method".into(), json!(method));
            if let Some(a) = args.get("args").and_then(|v| v.as_str()) {
                body.insert("args".into(), json!(a));
            }
            http_post(
                client,
                &format!("{base}/song/edit_method"),
                Value::Object(body),
            )
        }
        "dj_hermes_mute" => {
            let deck = arg_str(args, "deck")?;
            let track = arg_str(args, "track")?;
            let muted = args
                .get("muted")
                .and_then(|v| v.as_bool())
                .ok_or_else(|| rpc_error(-32602, "muted required"))?;
            http_post(
                client,
                &format!("{base}/mute"),
                json!({ "deck": deck, "track": track, "muted": muted }),
            )
        }
        "dj_hermes_head" => {
            let deck = arg_str(args, "deck")?;
            let bar = args
                .get("bar")
                .and_then(|v| v.as_u64())
                .ok_or_else(|| rpc_error(-32602, "bar required (positive integer, 1-based)"))?;
            if bar < 1 {
                return Err(rpc_error(-32602, "bar must be >= 1 (1 = first bar)"));
            }
            http_post(
                client,
                &format!("{base}/head"),
                json!({ "deck": deck, "bar": bar }),
            )
        }
        "dj_hermes_hush" => http_post_empty(client, &format!("{base}/hush")),
        "dj_hermes_status" => http_get(client, &format!("{base}/status")),
        other => return Err(rpc_error(-32602, format!("unknown tool: {other}"))),
    };

    match outcome {
        Ok(text) => Ok(tool_text_result(text, false)),
        Err(e) => Ok(tool_text_result(
            format_tool_http_error(base, &e, name),
            true,
        )),
    }
}

// ── In-process tool exec (HTTP /mcp) ────────────────────────────────────────

fn tools_call_local(state: &AppState, name: &str, args: &Value) -> Result<Value, Value> {
    if state.session == SessionKind::Play && is_mix_tool(name) {
        return Ok(tool_text_result(
            "this tool is dj-only; play is a single-deck session (deck A)".into(),
            true,
        ));
    }
    let outcome = match name {
        "dj_hermes_mixer_eq" => local_mixer_eq(state, args),
        "dj_hermes_mixer_filter" => local_mixer_filter(state, args),
        "dj_hermes_mixer_fx" => local_mixer_fx(state, args),
        "dj_hermes_mixer_repeat" => local_mixer_repeat(state, args),
        "dj_hermes_mixer_tape" => local_mixer_tape(state, args),
        "dj_hermes_mixer_crossfader" => local_mixer_crossfader(state, args),
        "dj_hermes_xfade" => local_xfade(state, args),
        "dj_hermes_mix" => local_mix(state, args),
        "dj_hermes_set_bpm" => local_set_bpm(state, args),
        "dj_hermes_load_song" => local_load_song(state, args),
        "dj_hermes_apply_song" => local_apply_song(state, args),
        "dj_hermes_list_songs" => local_list_songs(args),
        "dj_hermes_save_song" => local_save_song(state, args),
        "dj_hermes_get_song" => local_get_song(state, args),
        "dj_hermes_patch_track" => local_patch_track(state, args),
        "dj_hermes_edit_method" => local_edit_method(state, args),
        "dj_hermes_mute" => local_mute(state, args),
        "dj_hermes_head" => local_head(state, args),
        "dj_hermes_hush" => {
            let _ = state.tx.send(Command::Hush);
            Ok("ok (204)".into())
        }
        "dj_hermes_status" => {
            let info: StatusInfo = snapshot(&state.engine, Some(&state.deck_paths));
            serde_json::to_string(&info).map_err(|e| e.to_string())
        }
        other => return Err(rpc_error(-32602, format!("unknown tool: {other}"))),
    };

    match outcome {
        Ok(text) => Ok(tool_text_result(text, false)),
        Err(e) => Ok(tool_text_result(format_tool_local_error(&e, name), true)),
    }
}

fn send_cmd(state: &AppState, cmd: Command) -> Result<(), String> {
    state.tx.send(cmd).map_err(|e| e.to_string())
}

/// Deck argument: required in DJ; omitted → A in play; B is rejected in play.
fn arg_deck(state: &AppState, args: &Value) -> Result<usize, String> {
    match args
        .get("deck")
        .and_then(|v| v.as_str())
        .filter(|s| !s.is_empty())
    {
        None => {
            if state.session == SessionKind::Play {
                Ok(0)
            } else {
                Err("deck required (string)".into())
            }
        }
        Some(s) => {
            let d = deck_idx(s)?;
            if state.session == SessionKind::Play && d != 0 {
                Err("play is single-deck (A only); use dj-hermes dj for deck B".into())
            } else {
                Ok(d)
            }
        }
    }
}

fn local_mixer_eq(state: &AppState, args: &Value) -> Result<String, String> {
    let deck_s = args
        .get("deck")
        .and_then(|v| v.as_str())
        .ok_or_else(|| "deck required (string)".to_string())?;
    let deck = deck_idx(deck_s)?;
    let bands: [(u8, Option<f64>); 3] = [
        (0, args.get("hi").and_then(|v| v.as_f64())),
        (1, args.get("mid").and_then(|v| v.as_f64())),
        (2, args.get("lo").and_then(|v| v.as_f64())),
    ];
    if bands.iter().all(|(_, v)| v.is_none()) {
        return Err("mixer_eq: provide at least one of hi, mid, lo".into());
    }
    for (band, val) in bands {
        let Some(v) = val else { continue };
        if !v.is_finite() {
            return Err(format!("eq band {band} must be finite"));
        }
        send_cmd(
            state,
            Command::SetDeckEq {
                deck,
                band,
                value: (v as f32).clamp(0.0, 1.0),
            },
        )?;
    }
    Ok("ok".into())
}

fn local_mixer_filter(state: &AppState, args: &Value) -> Result<String, String> {
    let has_lpf = args.as_object().is_some_and(|m| m.contains_key("lpf"));
    let has_hpf = args.as_object().is_some_and(|m| m.contains_key("hpf"));
    if !has_lpf && !has_hpf {
        return Err("mixer_filter: provide lpf and/or hpf (number or null)".into());
    }
    if has_lpf {
        let hz = opt_hz(args.get("lpf"))?;
        send_cmd(state, Command::SetMixerLpf(hz))?;
    }
    if has_hpf {
        let hz = opt_hz(args.get("hpf"))?;
        send_cmd(state, Command::SetMixerHpf(hz))?;
    }
    Ok("ok".into())
}

fn local_mixer_fx(state: &AppState, args: &Value) -> Result<String, String> {
    let delay = args
        .get("delay")
        .and_then(|v| v.as_f64())
        .ok_or_else(|| "delay required (0..=1)".to_string())?;
    if !delay.is_finite() {
        return Err("delay wet must be finite 0..=1".into());
    }
    let feedback = match args.get("feedback") {
        None | Some(Value::Null) => None,
        Some(v) => {
            let n = v
                .as_f64()
                .ok_or_else(|| "feedback must be a number".to_string())?;
            if !n.is_finite() {
                return Err("feedback must be finite 0..=1".into());
            }
            Some((n as f32).clamp(0.0, 1.0))
        }
    };
    send_cmd(
        state,
        Command::SetMixerDelay {
            wet: (delay as f32).clamp(0.0, 1.0),
            feedback,
        },
    )?;
    Ok("ok".into())
}

fn local_mixer_tape(state: &AppState, args: &Value) -> Result<String, String> {
    let on = match args.get("on") {
        Some(Value::Bool(b)) => *b,
        None | Some(Value::Null) => return Err("on required (boolean)".into()),
        Some(_) => return Err("on must be a boolean".into()),
    };
    if !on {
        send_cmd(state, Command::SetTapeStop(None))?;
        return Ok("ok".into());
    }
    let len = match args.get("len") {
        None | Some(Value::Null) => None,
        Some(Value::String(s)) => Some(s.as_str()),
        Some(_) => return Err("len must be a string (1n|2n|4n|8n)".into()),
    };
    let reps = match args.get("reps") {
        None | Some(Value::Null) => None,
        Some(v) => {
            let n = v
                .as_u64()
                .ok_or_else(|| "reps must be an integer 1..=8".to_string())?;
            Some(n.to_string())
        }
    };
    let spec = match (len, reps.as_deref()) {
        (None, None) => crate::transport::TapeSpec::ONE_BAR,
        (Some(l), r) => crate::transport::TapeSpec::parse(l, r)?,
        (None, Some(r)) => crate::transport::TapeSpec::parse("1n", Some(r))?,
    };
    send_cmd(state, Command::SetTapeStop(Some(spec)))?;
    Ok("ok".into())
}

fn local_mixer_repeat(state: &AppState, args: &Value) -> Result<String, String> {
    let cmd = match args.get("div") {
        None | Some(Value::Null) => Command::SetTimeRepeat(None),
        Some(Value::String(s)) => {
            let t = s.trim();
            if t.is_empty()
                || t.eq_ignore_ascii_case("off")
                || t == "0"
                || t.eq_ignore_ascii_case("none")
            {
                Command::SetTimeRepeat(None)
            } else {
                let d = crate::transport::RepeatDiv::parse(t)?;
                Command::SetTimeRepeat(Some(d))
            }
        }
        Some(_) => return Err("div must be a string (4n|8n|16n|32n) or null".into()),
    };
    send_cmd(state, cmd)?;
    Ok("ok".into())
}

fn opt_hz(v: Option<&Value>) -> Result<Option<f32>, String> {
    match v {
        None | Some(Value::Null) => Ok(None),
        Some(x) => {
            let n = x
                .as_f64()
                .ok_or_else(|| "filter Hz must be number or null".to_string())?;
            if !(n.is_finite() && n > 0.0) {
                return Err(format!("filter Hz must be positive finite, got {n}"));
            }
            Ok(Some(n as f32))
        }
    }
}

fn local_mixer_crossfader(state: &AppState, args: &Value) -> Result<String, String> {
    let pos = args
        .get("pos")
        .and_then(|v| v.as_f64())
        .ok_or_else(|| "pos required (0..=1)".to_string())?;
    if !pos.is_finite() {
        return Err(format!("pos must be finite: {pos}"));
    }
    send_cmd(state, Command::SetCrossfader((pos as f32).clamp(0.0, 1.0)))?;
    Ok("ok".into())
}

fn local_mix(state: &AppState, args: &Value) -> Result<String, String> {
    let move_name = args
        .get("move")
        .and_then(|v| v.as_str())
        .ok_or_else(|| "move required (long|cut|fill|hold)".to_string())?;
    let req = crate::api::MixReq {
        move_name: move_name.to_string(),
        to: args
            .get("to")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string()),
        bars: args.get("bars").and_then(|v| v.as_u64()).map(|n| n as u32),
        eq: args.get("eq").and_then(|v| v.as_bool()),
        reset_eq: args.get("reset_eq").and_then(|v| v.as_bool()),
        kind: args
            .get("kind")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string()),
        grid: args
            .get("grid")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string()),
        mute_track: args
            .get("mute_track")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string()),
        phrase: args
            .get("phrase")
            .and_then(|v| v.as_u64())
            .map(|n| n as u32),
    };
    let cmd = crate::api::mix_command_from_req(&req)?;
    if !matches!(cmd, Command::HoldXFade) {
        let e = state
            .engine
            .lock()
            .map_err(|e| format!("engine lock: {e}"))?;
        let a = e.decks[0].song_title().is_some();
        let b = e.decks[1].song_title().is_some();
        drop(e);
        let both = matches!(
            &cmd,
            Command::Mix(m)
                if m.action == crate::mixer::MixAction::Long
                    || m.fill == Some(crate::mixer::FillKind::Switch)
        );
        if both && !(a && b) {
            return Err("both decks need a song loaded for long mix / switch".into());
        }
        if let Command::Mix(m) = &cmd {
            let loaded = if m.to_deck == 1 { b } else { a };
            if !loaded {
                return Err("target deck has no song loaded".into());
            }
        }
    }
    send_cmd(state, cmd)?;
    Ok("ok".into())
}

fn local_xfade(state: &AppState, args: &Value) -> Result<String, String> {
    let to = args
        .get("to")
        .and_then(|v| v.as_str())
        .ok_or_else(|| "to required (string)".to_string())?;
    let to_deck = deck_idx(to)?;
    let bars = args
        .get("bars")
        .and_then(|v| v.as_u64())
        .unwrap_or(4)
        .max(1) as u32;
    send_cmd(state, Command::XFade { to_deck, bars })?;
    Ok("ok (202)".into())
}

fn local_set_bpm(state: &AppState, args: &Value) -> Result<String, String> {
    let bpm = args
        .get("bpm")
        .and_then(|v| v.as_f64())
        .ok_or_else(|| "bpm required".to_string())?;
    if !(bpm.is_finite() && bpm > 0.0) {
        return Err(format!("bpm must be a positive finite number: {bpm}"));
    }
    send_cmd(state, Command::SetBpm(bpm))?;
    Ok("ok (202)".into())
}

fn local_load_song(state: &AppState, args: &Value) -> Result<String, String> {
    let path = args
        .get("path")
        .and_then(|v| v.as_str())
        .ok_or_else(|| "path required (string)".to_string())?;
    let deck = arg_deck(state, args)?;
    let resolved = resolve_song_path(path)?;
    let text = std::fs::read_to_string(&resolved).map_err(|e| format!("read: {e}"))?;
    let path_str = resolved.to_string_lossy();
    let song = parse_song(&text, &path_str)?;
    send_cmd(
        state,
        Command::LoadSong {
            deck,
            song: Box::new(song),
        },
    )?;
    if let Ok(mut dp) = state.deck_paths.lock() {
        dp[deck] = Some(resolved.clone());
    }
    Ok("ok (202)".into())
}

fn local_list_songs(args: &Value) -> Result<String, String> {
    let genre = args.get("genre").and_then(|v| v.as_str());
    let listing = song_listing(genre);
    let genres: Vec<Value> = listing
        .genres
        .into_iter()
        .map(|g| json!({ "name": g.name, "count": g.count }))
        .collect();
    let body = json!({
        "user_library": listing.user_library,
        "bundled": listing.bundled,
        "genres": genres,
        "load_hint": listing.load_hint,
    });
    serde_json::to_string(&body).map_err(|e| e.to_string())
}

fn local_get_song(state: &AppState, args: &Value) -> Result<String, String> {
    let deck = arg_deck(state, args)?;
    let label = if deck == 0 { "A" } else { "B" };
    let e = state
        .engine
        .lock()
        .map_err(|e| format!("engine lock: {e}"))?;
    let song = e.decks[deck]
        .song_ref()
        .ok_or_else(|| format!("deck {label} has no song loaded"))?;
    let tracks: Vec<Value> = song
        .track_sources()
        .into_iter()
        .map(|t| {
            json!({
                "name": t.name,
                "code": t.code,
                "muted": t.muted,
            })
        })
        .collect();
    let body = json!({
        "deck": if deck == 0 { "A" } else { "B" },
        "title": song.title,
        "path": song.path,
        "source": song.source,
        "tracks": tracks,
        "bpm": song.bpm,
    });
    serde_json::to_string(&body).map_err(|e| e.to_string())
}

fn local_apply_patched_song(
    state: &AppState,
    deck: usize,
    patched: crate::song::Song,
) -> Result<String, String> {
    let tracks: Vec<Value> = patched
        .track_sources()
        .into_iter()
        .map(|t| {
            json!({
                "name": t.name,
                "code": t.code,
                "muted": t.muted,
            })
        })
        .collect();
    let body = json!({
        "deck": if deck == 0 { "A" } else { "B" },
        "title": patched.title,
        "source": patched.source,
        "tracks": tracks,
    });

    {
        let mut e = state
            .engine
            .lock()
            .map_err(|e| format!("engine lock: {e}"))?;
        e.decks[deck].set_song_data(patched.clone());
    }
    send_cmd(
        state,
        Command::LoadSong {
            deck,
            song: Box::new(patched),
        },
    )?;
    serde_json::to_string(&body).map_err(|e| e.to_string())
}

fn local_patch_track(state: &AppState, args: &Value) -> Result<String, String> {
    let deck = arg_deck(state, args)?;
    let label = if deck == 0 { "A" } else { "B" };
    let track = args
        .get("track")
        .and_then(|v| v.as_str())
        .ok_or_else(|| "track required (string)".to_string())?;
    let op = args
        .get("op")
        .and_then(|v| v.as_str())
        .ok_or_else(|| "op required (string)".to_string())?;
    let code = args.get("code").and_then(|v| v.as_str());
    let name = args.get("name").and_then(|v| v.as_str());

    let song = {
        let e = state
            .engine
            .lock()
            .map_err(|e| format!("engine lock: {e}"))?;
        e.decks[deck]
            .song_ref()
            .cloned()
            .ok_or_else(|| format!("deck {label} has no song loaded"))?
    };
    let patched = song.patch_track(track, op, code, name)?;
    local_apply_patched_song(state, deck, patched)
}

fn local_edit_method(state: &AppState, args: &Value) -> Result<String, String> {
    let deck = arg_deck(state, args)?;
    let label = if deck == 0 { "A" } else { "B" };
    let track = args
        .get("track")
        .and_then(|v| v.as_str())
        .ok_or_else(|| "track required (string)".to_string())?;
    let op = args
        .get("op")
        .and_then(|v| v.as_str())
        .ok_or_else(|| "op required (string)".to_string())?;
    let method = args
        .get("method")
        .and_then(|v| v.as_str())
        .ok_or_else(|| "method required (string)".to_string())?;
    let method_args = args.get("args").and_then(|v| v.as_str()).unwrap_or("");

    let song = {
        let e = state
            .engine
            .lock()
            .map_err(|e| format!("engine lock: {e}"))?;
        e.decks[deck]
            .song_ref()
            .cloned()
            .ok_or_else(|| format!("deck {label} has no song loaded"))?
    };
    let patched = song.edit_method(track, op, method, method_args)?;
    local_apply_patched_song(state, deck, patched)
}

fn local_apply_song(state: &AppState, args: &Value) -> Result<String, String> {
    let content = args
        .get("content")
        .and_then(|v| v.as_str())
        .ok_or_else(|| "content required (string)".to_string())?;
    if content.len() > MAX_SONG_CONTENT_BYTES {
        return Err(format!(
            "content too large ({} bytes, max {MAX_SONG_CONTENT_BYTES})",
            content.len()
        ));
    }
    let deck = arg_deck(state, args)?;
    let song = parse_song(content, "")?;
    local_apply_patched_song(state, deck, song)
}

fn local_save_song(state: &AppState, args: &Value) -> Result<String, String> {
    let name = args
        .get("name")
        .and_then(|v| v.as_str())
        .ok_or_else(|| "name required (string)".to_string())?;
    let content = match args.get("content").and_then(|v| v.as_str()) {
        Some(c) if !c.is_empty() => c.to_string(),
        _ => {
            if args.get("deck").and_then(|v| v.as_str()).is_none()
                && state.session != SessionKind::Play
            {
                return Err("content or deck required".into());
            }
            let deck = arg_deck(state, args)?;
            let label = if deck == 0 { "A" } else { "B" };
            let e = state
                .engine
                .lock()
                .map_err(|e| format!("engine lock: {e}"))?;
            let song = e.decks[deck]
                .song_ref()
                .ok_or_else(|| format!("deck {label} has no song loaded"))?;
            song.source.clone()
        }
    };
    if content.len() > MAX_SONG_CONTENT_BYTES {
        return Err(format!(
            "content too large ({} bytes, max {MAX_SONG_CONTENT_BYTES})",
            content.len()
        ));
    }
    let path = resolve_user_song_save_path(name)?;
    let file_name = path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("song.strudel")
        .to_string();
    let overwrite = args
        .get("overwrite")
        .and_then(|v| v.as_bool())
        .unwrap_or(true);
    if path.is_file() && !overwrite {
        return Err(format!(
            "song already exists: {} (set overwrite=true to replace)",
            path.display()
        ));
    }
    let path_str = path.to_string_lossy().into_owned();
    let _song = parse_song(&content, &path_str)?;
    ensure_user_songs_dir()?;
    std::fs::write(&path, content.as_bytes()).map_err(|e| format!("write: {e}"))?;

    let res = json!({
        "path": path_str,
        "name": file_name,
    });
    serde_json::to_string(&res).map_err(|e| e.to_string())
}

fn local_mute(state: &AppState, args: &Value) -> Result<String, String> {
    let deck = arg_deck(state, args)?;
    let track = args
        .get("track")
        .and_then(|v| v.as_str())
        .ok_or_else(|| "track required (string)".to_string())?;
    let muted = args
        .get("muted")
        .and_then(|v| v.as_bool())
        .ok_or_else(|| "muted required".to_string())?;
    if track.trim().is_empty() {
        return Err("track name is empty".into());
    }
    send_cmd(
        state,
        Command::SetTrackMute {
            deck,
            track: track.to_string(),
            muted,
        },
    )?;
    Ok("ok (202)".into())
}

fn local_head(state: &AppState, args: &Value) -> Result<String, String> {
    let deck = arg_deck(state, args)?;
    let bar = args
        .get("bar")
        .and_then(|v| v.as_u64())
        .ok_or_else(|| "bar required (positive integer, 1-based)".to_string())?;
    if bar < 1 {
        return Err("bar must be >= 1 (1 = first bar)".into());
    }
    send_cmd(state, Command::Head { deck, bar })?;
    Ok("ok (202)".into())
}

fn parse_content_retry_tool(tool: &str) -> &'static str {
    if tool == "dj_hermes_apply_song" {
        "dj_hermes_apply_song"
    } else {
        "dj_hermes_save_song"
    }
}

fn format_tool_local_error(err: &str, tool: &str) -> String {
    let lower = err.to_ascii_lowercase();
    let mut out = format!("error: {err}");
    if lower.contains("song not found") {
        out.push_str(
            "\nHint: bundled path is house/01 or legacy house-01; user-library is a basename (visitor-dnb). \
Call dj_hermes_list_songs (optional genre=house), then dj_hermes_load_song(path=..., deck=A|B).",
        );
    } else {
        let looks_like_song_parse = (lower.contains("expected")
            && (lower.contains("$:") || lower.contains("name: code")))
            || lower.contains("unexpected char");
        if looks_like_song_parse {
            let retry = parse_content_retry_tool(tool);
            out.push_str(&format!(
                "\nHint: song content must use setcpm(N) (or setcpm(BPM/4)) and `$: <chain>` lines. \
Do not use stack(...) or .cpm(). Retry {retry} with corrected content.",
            ));
        }
    }
    out
}

// ── Shared helpers ──────────────────────────────────────────────────────────

/// Map HTTP / transport errors to model-facing text.
fn format_tool_http_error(base: &str, err: &str, tool: &str) -> String {
    let lower = err.to_ascii_lowercase();
    let is_http_status = lower.contains("http 4") || lower.contains("http 5");
    let is_connect = !is_http_status
        && (lower.contains("connect")
            || lower.contains("connection")
            || lower.contains("timed out")
            || lower.contains("timeout")
            || lower.contains("dns")
            || lower.contains("refused")
            || lower.contains("error sending request")
            || lower.contains("tcp"));

    if is_connect {
        return format!(
            "error: {err}\n(Is play running with API on {base}? Default port {DEFAULT_API_PORT})"
        );
    }

    let mut out = format!("error: {err}");
    if lower.contains("song not found") {
        out.push_str(
            "\nHint: bundled path is house/01 or legacy house-01; user-library is a basename (visitor-dnb). \
Call dj_hermes_list_songs (optional genre=house), then dj_hermes_load_song(path=..., deck=A|B).",
        );
    } else {
        let looks_like_song_parse = (lower.contains("expected")
            && (lower.contains("$:") || lower.contains("name: code")))
            || (lower.contains("unexpected char") && lower.contains("http 400"));
        if looks_like_song_parse {
            let retry = parse_content_retry_tool(tool);
            out.push_str(&format!(
                "\nHint: song content must use setcpm(N) (or setcpm(BPM/4)) and `$: <chain>` lines. \
Do not use stack(...) or .cpm(). Retry {retry} with corrected content.",
            ));
        }
    }
    out
}

fn tool_text_result(text: String, is_error: bool) -> Value {
    json!({
        "content": [{ "type": "text", "text": text }],
        "isError": is_error
    })
}

fn arg_str(args: &Value, key: &str) -> Result<String, Value> {
    args.get(key)
        .and_then(|v| v.as_str())
        .map(|s| s.to_string())
        .ok_or_else(|| rpc_error(-32602, format!("{key} required (string)")))
}

fn http_post(client: &reqwest::blocking::Client, url: &str, body: Value) -> Result<String, String> {
    let res = client
        .post(url)
        .json(&body)
        .send()
        .map_err(|e| e.to_string())?;
    status_text(res)
}

fn http_post_empty(client: &reqwest::blocking::Client, url: &str) -> Result<String, String> {
    let res = client.post(url).send().map_err(|e| e.to_string())?;
    status_text(res)
}

fn http_get(client: &reqwest::blocking::Client, url: &str) -> Result<String, String> {
    let res = client.get(url).send().map_err(|e| e.to_string())?;
    status_text(res)
}

fn status_text(res: reqwest::blocking::Response) -> Result<String, String> {
    let status = res.status();
    let text = res.text().unwrap_or_default();
    if status.is_success() || status.as_u16() == 202 || status.as_u16() == 204 {
        if text.is_empty() {
            Ok(format!("ok ({status})"))
        } else {
            Ok(text)
        }
    } else {
        Err(format!("HTTP {status}: {text}"))
    }
}

fn rpc_error(code: i64, message: impl Into<String>) -> Value {
    json!({ "code": code, "message": message.into() })
}

/// Read one MCP message. Detects NDJSON or Content-Length framing.
fn read_message(stdin: &mut impl BufRead) -> Result<Option<(Value, Framing)>, String> {
    let mut headers = Map::new();
    let mut line = String::new();
    loop {
        line.clear();
        let n = stdin
            .read_line(&mut line)
            .map_err(|e| format!("stdin read: {e}"))?;
        if n == 0 {
            return Ok(None);
        }
        let t = line.trim_end_matches(['\r', '\n']);
        if t.is_empty() {
            break;
        }
        if t.starts_with('{') {
            let v: Value = serde_json::from_str(t).map_err(|e| format!("json parse: {e}"))?;
            return Ok(Some((v, Framing::Ndjson)));
        }
        if let Some((k, v)) = t.split_once(':') {
            headers.insert(k.trim().to_ascii_lowercase(), json!(v.trim()));
        }
    }

    if headers.is_empty() {
        return Ok(None);
    }

    let len = headers
        .get("content-length")
        .and_then(|v| v.as_str())
        .and_then(|s| s.parse::<usize>().ok())
        .ok_or_else(|| "missing Content-Length".to_string())?;

    let mut buf = vec![0u8; len];
    Read::read_exact(stdin, &mut buf).map_err(|e| format!("body read: {e}"))?;
    let v: Value = serde_json::from_slice(&buf).map_err(|e| format!("json body: {e}"))?;
    Ok(Some((v, Framing::ContentLength)))
}

fn write_message(stdout: &mut impl Write, value: &Value, framing: Framing) -> Result<(), String> {
    match framing {
        Framing::Ndjson => {
            let body = serde_json::to_string(value).map_err(|e| format!("json encode: {e}"))?;
            writeln!(stdout, "{body}").map_err(|e| format!("stdout: {e}"))?;
            stdout.flush().map_err(|e| format!("stdout flush: {e}"))?;
        }
        Framing::ContentLength => {
            let body = serde_json::to_vec(value).map_err(|e| format!("json encode: {e}"))?;
            write!(stdout, "Content-Length: {}\r\n\r\n", body.len())
                .map_err(|e| format!("stdout: {e}"))?;
            stdout
                .write_all(&body)
                .map_err(|e| format!("stdout body: {e}"))?;
            stdout.flush().map_err(|e| format!("stdout flush: {e}"))?;
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;
    use std::sync::{Arc, Mutex};

    use crossbeam::channel::unbounded;

    use crate::engine::{Command, Engine};

    fn test_state() -> (AppState, crossbeam::channel::Receiver<Command>) {
        let (tx, rx) = unbounded();
        let engine = Arc::new(Mutex::new(Engine::new(48_000, 120.0)));
        (
            AppState {
                tx,
                engine,
                session: SessionKind::Dj,
                deck_paths: crate::cmd::new_deck_paths(),
            },
            rx,
        )
    }

    fn test_state_play() -> (AppState, crossbeam::channel::Receiver<Command>) {
        let (tx, rx) = unbounded();
        let engine = Arc::new(Mutex::new(Engine::new(48_000, 120.0)));
        (
            AppState {
                tx,
                engine,
                session: SessionKind::Play,
                deck_paths: crate::cmd::new_deck_paths(),
            },
            rx,
        )
    }

    #[test]
    fn tools_list_mixer_deck_transport_no_set_code() {
        let v = tools_list(SessionKind::Dj);
        let tools = v["tools"].as_array().unwrap();
        assert_eq!(tools.len(), 20);
        let names: Vec<_> = tools.iter().filter_map(|t| t["name"].as_str()).collect();
        assert!(!names.contains(&"dj_hermes_set_code"));
        assert_eq!(names[0], "dj_hermes_mixer_eq");
        assert_eq!(names[1], "dj_hermes_mixer_filter");
        assert_eq!(names[2], "dj_hermes_mixer_fx");
        assert_eq!(names[3], "dj_hermes_mixer_repeat");
        assert_eq!(names[4], "dj_hermes_mixer_tape");
        assert_eq!(names[5], "dj_hermes_mixer_crossfader");
        assert!(names.contains(&"dj_hermes_mixer_fx"));
        assert!(names.contains(&"dj_hermes_mixer_repeat"));
        assert!(names.contains(&"dj_hermes_mixer_tape"));
        assert!(names.contains(&"dj_hermes_xfade"));
        assert!(names.contains(&"dj_hermes_mix"));
        assert!(names.contains(&"dj_hermes_set_bpm"));
        assert!(names.contains(&"dj_hermes_load_song"));
        assert!(names.contains(&"dj_hermes_apply_song"));
        assert!(names.contains(&"dj_hermes_list_songs"));
        assert!(names.contains(&"dj_hermes_save_song"));
        assert!(names.contains(&"dj_hermes_get_song"));
        assert!(names.contains(&"dj_hermes_patch_track"));
        assert!(names.contains(&"dj_hermes_edit_method"));
        assert!(names.contains(&"dj_hermes_head"));
        assert!(names.contains(&"dj_hermes_status"));
        let descs: Vec<_> = tools
            .iter()
            .filter_map(|t| t["description"].as_str())
            .collect();
        assert!(descs.iter().any(|d| d.starts_with("Mixer:")));
        assert!(descs.iter().any(|d| d.starts_with("Deck:")));
        assert!(descs.iter().any(|d| d.starts_with("Transport:")));
        let save = tools
            .iter()
            .find(|t| t["name"] == "dj_hermes_save_song")
            .unwrap();
        let save_desc = save["description"].as_str().unwrap();
        assert!(
            save_desc.contains("setcpm") && save_desc.contains("$:"),
            "save description should show required format: {save_desc}"
        );
        assert!(
            save_desc.contains("stack") || save_desc.contains("never stack"),
            "save description should forbid stack: {save_desc}"
        );
    }

    #[test]
    fn format_tool_http_error_connect_gets_play_hint() {
        let msg = format_tool_http_error(
            "http://127.0.0.1:17878",
            "error sending request for url (http://127.0.0.1:17878/song/save): connection refused",
            "dj_hermes_save_song",
        );
        assert!(msg.contains("Is play running"), "{msg}");
        assert!(msg.contains("17878"), "{msg}");
    }

    #[test]
    fn format_tool_http_error_parse_400_no_play_hint() {
        let msg = format_tool_http_error(
            "http://127.0.0.1:17878",
            "HTTP 400 Bad Request: {\"error\":\"line 1: expected 'name: code' or '$: code'\"}",
            "dj_hermes_apply_song",
        );
        assert!(
            !msg.contains("Is play running"),
            "parse 400 must not look like connection failure: {msg}"
        );
        assert!(msg.contains("setcpm") || msg.contains("$:"), "{msg}");
        assert!(msg.contains("stack") || msg.contains("Hint:"), "{msg}");
    }

    #[test]
    fn format_tool_http_error_not_found_hints_basename() {
        let msg = format_tool_http_error(
            "http://127.0.0.1:17878",
            "HTTP 400 Bad Request: {\"error\":\"song not found: songs/house-track.strudel (tried: songs/house-track.strudel)\"}",
            "dj_hermes_load_song",
        );
        assert!(!msg.contains("Is play running"), "{msg}");
        assert!(
            !msg.contains("setcpm"),
            "load not-found must not get save-format hint: {msg}"
        );
        assert!(
            msg.contains("bare basename") || msg.contains("list_songs"),
            "{msg}"
        );
    }

    #[test]
    fn handle_rpc_initialize_and_tools_list() {
        let (state, _rx) = test_state();
        let backend = ToolBackend::Local { state: &state };

        let init = json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "initialize",
            "params": {
                "protocolVersion": "2025-03-26",
                "capabilities": {},
                "clientInfo": { "name": "t", "version": "0" }
            }
        });
        let resp = handle_rpc(&init, &backend).expect("init reply");
        assert_eq!(resp["id"], 1);
        assert_eq!(resp["result"]["protocolVersion"], "2025-03-26");
        assert_eq!(resp["result"]["serverInfo"]["name"], "dj-hermes");

        let list = json!({
            "jsonrpc": "2.0",
            "id": 2,
            "method": "tools/list"
        });
        let resp = handle_rpc(&list, &backend).expect("list reply");
        assert_eq!(resp["result"]["tools"].as_array().unwrap().len(), 20);
    }

    #[test]
    fn tools_list_play_omits_mix_and_defaults_deck() {
        let v = tools_list(SessionKind::Play);
        let tools = v["tools"].as_array().unwrap();
        let names: Vec<_> = tools.iter().filter_map(|t| t["name"].as_str()).collect();
        assert_eq!(names.len(), 14);
        for mix in MIX_TOOL_NAMES {
            assert!(!names.contains(mix), "{mix} should be hidden in play");
        }
        assert!(names.contains(&"dj_hermes_mixer_repeat"));
        assert!(names.contains(&"dj_hermes_mixer_tape"));
        assert!(names.contains(&"dj_hermes_apply_song"));
        assert!(names.contains(&"dj_hermes_set_bpm"));
        let load = tools
            .iter()
            .find(|t| t["name"] == "dj_hermes_load_song")
            .unwrap();
        let req = load["inputSchema"]["required"].as_array().unwrap();
        assert!(!req.iter().any(|x| x.as_str() == Some("deck")), "{req:?}");
        assert!(req.iter().any(|x| x.as_str() == Some("path")), "{req:?}");

        for t in tools {
            let name = t["name"].as_str().unwrap_or("?");
            match t.get("inputSchema").and_then(|s| s.get("required")) {
                None => {}
                Some(Value::Array(a)) => {
                    assert!(
                        !a.is_empty(),
                        "{name}: empty required must be omitted (xAI rejects null/empty)"
                    );
                    assert!(
                        !a.iter().any(|x| x.as_str() == Some("deck")),
                        "{name}: deck must not stay required in play"
                    );
                }
                other => panic!("{name}: required must be a JSON array or absent, got {other:?}"),
            }
        }
        let list = tools
            .iter()
            .find(|t| t["name"] == "dj_hermes_list_songs")
            .unwrap();
        assert!(
            list.get("inputSchema")
                .and_then(|s| s.get("required"))
                .is_none(),
            "list_songs must not gain required:null via IndexMut"
        );
        let get = tools
            .iter()
            .find(|t| t["name"] == "dj_hermes_get_song")
            .unwrap();
        assert!(
            get.get("inputSchema")
                .and_then(|s| s.get("required"))
                .is_none(),
            "get_song required was only deck; omit after play relax"
        );
    }

    #[test]
    fn play_apply_omits_deck_and_rejects_mix() {
        let (state, rx) = test_state_play();
        let backend = ToolBackend::Local { state: &state };
        let apply = json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "tools/call",
            "params": {
                "name": "dj_hermes_apply_song",
                "arguments": {
                    "content": "// @title t\nsetcpm(30)\n$: s(\"bd*4\")\n"
                }
            }
        });
        let resp = handle_rpc(&apply, &backend).unwrap();
        assert_eq!(resp["result"]["isError"], false, "{resp}");
        match rx.try_recv().unwrap() {
            Command::LoadSong { deck, .. } => assert_eq!(deck, 0),
            _ => panic!("expected LoadSong"),
        }

        let mix = json!({
            "jsonrpc": "2.0",
            "id": 2,
            "method": "tools/call",
            "params": {
                "name": "dj_hermes_xfade",
                "arguments": { "to": "B" }
            }
        });
        let resp = handle_rpc(&mix, &backend).unwrap();
        assert_eq!(resp["result"]["isError"], true);
        let text = resp["result"]["content"][0]["text"].as_str().unwrap();
        assert!(text.contains("dj-only"), "{text}");
        assert!(rx.try_recv().is_err());

        let b = json!({
            "jsonrpc": "2.0",
            "id": 3,
            "method": "tools/call",
            "params": {
                "name": "dj_hermes_get_song",
                "arguments": { "deck": "B" }
            }
        });
        let resp = handle_rpc(&b, &backend).unwrap();
        assert_eq!(resp["result"]["isError"], true);
        let text = resp["result"]["content"][0]["text"].as_str().unwrap();
        assert!(text.contains("single-deck"), "{text}");
    }

    #[test]
    fn handle_rpc_notification_no_reply() {
        let (state, _rx) = test_state();
        let backend = ToolBackend::Local { state: &state };
        let n = json!({
            "jsonrpc": "2.0",
            "method": "notifications/initialized"
        });
        assert!(handle_rpc(&n, &backend).is_none());
    }

    #[test]
    fn local_tools_call_status_and_eq() {
        let (state, rx) = test_state();
        let backend = ToolBackend::Local { state: &state };

        let status_msg = json!({
            "jsonrpc": "2.0",
            "id": 3,
            "method": "tools/call",
            "params": { "name": "dj_hermes_status", "arguments": {} }
        });
        let resp = handle_rpc(&status_msg, &backend).unwrap();
        assert_eq!(resp["result"]["isError"], false);
        let text = resp["result"]["content"][0]["text"].as_str().unwrap();
        assert!(text.contains("bpm") || text.contains("deck"), "{text}");

        let eq_msg = json!({
            "jsonrpc": "2.0",
            "id": 4,
            "method": "tools/call",
            "params": {
                "name": "dj_hermes_mixer_eq",
                "arguments": { "deck": "A", "lo": 0.3 }
            }
        });
        let resp = handle_rpc(&eq_msg, &backend).unwrap();
        assert_eq!(resp["result"]["isError"], false);
        match rx.try_recv().unwrap() {
            Command::SetDeckEq { deck, band, value } => {
                assert_eq!(deck, 0);
                assert_eq!(band, 2);
                assert!((value - 0.3).abs() < 1e-5);
            }
            _ => panic!("expected SetDeckEq"),
        }
    }

    #[test]
    fn read_ndjson_line() {
        let raw = r#"{"jsonrpc":"2.0","id":1,"method":"initialize","params":{}}"#;
        let mut cur = Cursor::new(format!("{raw}\n"));
        let (msg, framing) = read_message(&mut cur).unwrap().unwrap();
        assert_eq!(msg["method"], "initialize");
        assert_eq!(framing, Framing::Ndjson);
    }

    #[test]
    fn read_content_length_frame() {
        let body = r#"{"jsonrpc":"2.0","id":2,"method":"tools/list"}"#;
        let frame = format!("Content-Length: {}\r\n\r\n{}", body.len(), body);
        let mut cur = Cursor::new(frame);
        let (msg, framing) = read_message(&mut cur).unwrap().unwrap();
        assert_eq!(msg["method"], "tools/list");
        assert_eq!(framing, Framing::ContentLength);
    }

    #[test]
    fn write_ndjson_is_single_line_json() {
        let v = json!({"jsonrpc":"2.0","id":1,"result":{}});
        let mut out = Vec::new();
        write_message(&mut out, &v, Framing::Ndjson).unwrap();
        let s = String::from_utf8(out).unwrap();
        assert!(s.ends_with('\n'));
        assert!(!s.contains("Content-Length"));
        let line = s.trim_end_matches('\n');
        assert!(line.starts_with('{'));
        let parsed: Value = serde_json::from_str(line).unwrap();
        assert_eq!(parsed["id"], 1);
    }

    #[test]
    fn write_content_length_has_header() {
        let v = json!({"jsonrpc":"2.0","id":2,"result":{}});
        let mut out = Vec::new();
        write_message(&mut out, &v, Framing::ContentLength).unwrap();
        let s = String::from_utf8(out).unwrap();
        assert!(s.starts_with("Content-Length:"));
        assert!(s.contains("\r\n\r\n"));
    }

    #[test]
    fn roundtrip_ndjson_write_then_read() {
        let v = json!({"jsonrpc":"2.0","id":9,"method":"ping"});
        let mut buf = Vec::new();
        write_message(&mut buf, &v, Framing::Ndjson).unwrap();
        let mut cur = Cursor::new(buf);
        let (msg, framing) = read_message(&mut cur).unwrap().unwrap();
        assert_eq!(framing, Framing::Ndjson);
        assert_eq!(msg["method"], "ping");
        assert_eq!(msg["id"], 9);
    }
}

//! MCP server (stdio) — thin HTTP bridge to the play process API.
//!
//! Does **not** open audio devices. Requires the main `play` process with API up.
//! Protocol: JSON-RPC 2.0 over stdio with Content-Length framing (MCP transport).
//!
//! Implemented by hand (not `rmcp`) so the bridge stays small and independent of
//! crate API churn. See plan Risk #6.

use std::io::{self, BufRead, Read, Write};

use serde_json::{json, Map, Value};

use crate::api::{default_api_base, DEFAULT_API_PORT};

const PROTOCOL_VERSION: &str = "2024-11-05";
const SERVER_NAME: &str = "strudel-rs";
const SERVER_VERSION: &str = env!("CARGO_PKG_VERSION");

/// Run MCP server until stdin EOF. Blocks the calling thread.
pub fn run() -> Result<(), String> {
    let base = default_api_base();
    let client = reqwest::blocking::Client::builder()
        .timeout(std::time::Duration::from_secs(10))
        .build()
        .map_err(|e| format!("http client: {e}"))?;

    let stdin = io::stdin();
    let mut stdin = stdin.lock();
    let mut stdout = io::stdout();

    while let Some(msg) = read_message(&mut stdin)? {
        // Notifications have no `id` — handle and do not reply (except we may ignore).
        let id = msg.get("id").cloned();
        let method = msg
            .get("method")
            .and_then(|m| m.as_str())
            .unwrap_or("")
            .to_string();

        if id.is_none() {
            // notifications/initialized etc. — ignore
            continue;
        }

        let result = match method.as_str() {
            "initialize" => Ok(initialize_result()),
            "ping" => Ok(json!({})),
            "tools/list" => Ok(tools_list()),
            "tools/call" => {
                let params = msg.get("params").cloned().unwrap_or(json!({}));
                tools_call(&client, &base, params)
            }
            "resources/list" => Ok(json!({ "resources": [] })),
            "prompts/list" => Ok(json!({ "prompts": [] })),
            other => Err(rpc_error(-32601, format!("method not found: {other}"))),
        };

        let response = match result {
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
        };
        write_message(&mut stdout, &response)?;
    }
    Ok(())
}

fn initialize_result() -> Value {
    json!({
        "protocolVersion": PROTOCOL_VERSION,
        "capabilities": {
            "tools": { "listChanged": false }
        },
        "serverInfo": {
            "name": SERVER_NAME,
            "version": SERVER_VERSION
        }
    })
}

fn tools_list() -> Value {
    json!({
        "tools": [
            {
                "name": "strudel_set_code",
                "description": "Load a single mini-notation pattern onto a deck (next bar).",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "code": { "type": "string", "description": "Pattern code, e.g. note(\"c3 e3\").s(\"triangle\")" },
                        "deck": { "type": "string", "description": "A or B (default A)" }
                    },
                    "required": ["code"]
                }
            },
            {
                "name": "strudel_load_song",
                "description": "Load a .strudel song file onto a deck (next bar).",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "path": { "type": "string" },
                        "deck": { "type": "string", "description": "A or B" }
                    },
                    "required": ["path", "deck"]
                }
            },
            {
                "name": "strudel_xfade",
                "description": "Crossfade to deck A or B over N bars (starts next bar).",
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
                "name": "strudel_set_bpm",
                "description": "Set master BPM (applies next bar).",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "bpm": { "type": "number" }
                    },
                    "required": ["bpm"]
                }
            },
            {
                "name": "strudel_mute",
                "description": "Mute or unmute a track on a deck (next bar).",
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
                "name": "strudel_hush",
                "description": "Stop all sound immediately.",
                "inputSchema": {
                    "type": "object",
                    "properties": {}
                }
            },
            {
                "name": "strudel_status",
                "description": "Get current decks, BPM, and mixer gains.",
                "inputSchema": {
                    "type": "object",
                    "properties": {}
                }
            }
        ]
    })
}

fn tools_call(
    client: &reqwest::blocking::Client,
    base: &str,
    params: Value,
) -> Result<Value, Value> {
    let name = params
        .get("name")
        .and_then(|v| v.as_str())
        .ok_or_else(|| rpc_error(-32602, "missing tool name"))?;
    let args = params
        .get("arguments")
        .cloned()
        .unwrap_or_else(|| json!({}));

    let outcome = match name {
        "strudel_set_code" => {
            let code = arg_str(&args, "code")?;
            let deck = arg_str_opt(&args, "deck").unwrap_or_else(|| "A".into());
            http_put(
                client,
                &format!("{base}/code"),
                json!({ "code": code, "deck": deck }),
            )
        }
        "strudel_load_song" => {
            let path = arg_str(&args, "path")?;
            let deck = arg_str(&args, "deck")?;
            http_post(
                client,
                &format!("{base}/song/load"),
                json!({ "path": path, "deck": deck }),
            )
        }
        "strudel_xfade" => {
            let to = arg_str(&args, "to")?;
            let bars = args.get("bars").and_then(|v| v.as_u64()).unwrap_or(4);
            http_post(
                client,
                &format!("{base}/xfade"),
                json!({ "to": to, "bars": bars }),
            )
        }
        "strudel_set_bpm" => {
            let bpm = args
                .get("bpm")
                .and_then(|v| v.as_f64())
                .ok_or_else(|| rpc_error(-32602, "bpm required"))?;
            http_post(client, &format!("{base}/bpm"), json!({ "bpm": bpm }))
        }
        "strudel_mute" => {
            let deck = arg_str(&args, "deck")?;
            let track = arg_str(&args, "track")?;
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
        "strudel_hush" => http_post_empty(client, &format!("{base}/hush")),
        "strudel_status" => http_get(client, &format!("{base}/status")),
        other => return Err(rpc_error(-32602, format!("unknown tool: {other}"))),
    };

    match outcome {
        Ok(text) => Ok(tool_text_result(text, false)),
        Err(e) => {
            // Surface connection errors clearly (play process not running).
            Ok(tool_text_result(
                format!(
                    "error: {e}\n(Is play running with API on {base}? Default port {DEFAULT_API_PORT})"
                ),
                true,
            ))
        }
    }
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

fn arg_str_opt(args: &Value, key: &str) -> Option<String> {
    args.get(key)
        .and_then(|v| v.as_str())
        .map(|s| s.to_string())
}

fn http_put(client: &reqwest::blocking::Client, url: &str, body: Value) -> Result<String, String> {
    let res = client
        .put(url)
        .json(&body)
        .send()
        .map_err(|e| e.to_string())?;
    status_text(res)
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

/// Read one MCP message (Content-Length framing, or a single JSON line as fallback).
fn read_message(stdin: &mut impl BufRead) -> Result<Option<Value>, String> {
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
        // Fallback: bare JSON line without headers
        if t.starts_with('{') {
            let v: Value = serde_json::from_str(t).map_err(|e| format!("json parse: {e}"))?;
            return Ok(Some(v));
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
    Ok(Some(v))
}

fn write_message(stdout: &mut impl Write, value: &Value) -> Result<(), String> {
    let body = serde_json::to_vec(value).map_err(|e| format!("json encode: {e}"))?;
    write!(stdout, "Content-Length: {}\r\n\r\n", body.len()).map_err(|e| format!("stdout: {e}"))?;
    stdout
        .write_all(&body)
        .map_err(|e| format!("stdout body: {e}"))?;
    stdout.flush().map_err(|e| format!("stdout flush: {e}"))?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn tools_list_has_seven() {
        let v = tools_list();
        let tools = v["tools"].as_array().unwrap();
        assert_eq!(tools.len(), 7);
        let names: Vec<_> = tools.iter().filter_map(|t| t["name"].as_str()).collect();
        assert!(names.contains(&"strudel_set_code"));
        assert!(names.contains(&"strudel_status"));
    }

    #[test]
    fn read_json_line_fallback() {
        let raw = r#"{"jsonrpc":"2.0","id":1,"method":"initialize","params":{}}"#;
        let mut cur = Cursor::new(format!("{raw}\n"));
        let msg = read_message(&mut cur).unwrap().unwrap();
        assert_eq!(msg["method"], "initialize");
    }

    #[test]
    fn read_content_length_frame() {
        let body = r#"{"jsonrpc":"2.0","id":2,"method":"tools/list"}"#;
        let frame = format!("Content-Length: {}\r\n\r\n{}", body.len(), body);
        let mut cur = Cursor::new(frame);
        let msg = read_message(&mut cur).unwrap().unwrap();
        assert_eq!(msg["method"], "tools/list");
    }
}

//! Live TUI local-command suggestions (token-stage completion).
//!
//! Pure logic for tests; the TUI draws candidates and handles Tab.
//! Hermes natural-language prompts (no leading `/`) get no suggestions.

use std::path::Path;

use crate::session::SessionKind;
use crate::song::{
    is_genre_slug, list_bundled_genres, list_bundled_slots, list_user_library_songs,
};

/// Context for mute/unmute track names and Hermes routing.
#[derive(Clone, Copy, Debug)]
pub struct CompleteCtx<'a> {
    pub hermes_enabled: bool,
    pub tracks_a: &'a [String],
    pub tracks_b: &'a [String],
    pub session: SessionKind,
}

/// Result of analyzing the current prompt line.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct CompleteResult {
    /// Tokens that can replace the current partial (Tab applies these).
    pub candidates: Vec<String>,
    /// Byte index into the original `input` where the partial token starts.
    pub replace_from: usize,
    /// Non-file argument hint when there are no list candidates (e.g. `<bpm>`).
    pub hint: Option<String>,
}

impl CompleteResult {
    pub fn empty() -> Self {
        Self::default()
    }

    pub fn has_display(&self) -> bool {
        !self.candidates.is_empty() || self.hint.is_some()
    }
}

const TOP_LEVEL: &[&str] = &[
    "a", "b", "x", "mix", "filter", "delay", "vinyl", "repeat", "tape", "bpm", "hush", "status",
    "help", "list", "quit", "viz", "vfx", "dopa", "flash",
];
const TOP_LEVEL_PLAY: &[&str] = &[
    "a", "load", "save", "reload", "mute", "unmute", "gain", "head", "filter", "delay", "vinyl",
    "repeat", "tape", "bpm", "hush", "status", "help", "list", "quit", "viz", "vfx", "dopa",
    "flash",
];
const REPEAT_DIVS: &[&str] = &["4n", "8n", "16n", "32n", "off"];
const TAPE_LENS: &[&str] = &["1n", "2n", "4n", "8n", "off"];
const MIX_MOVES: &[&str] = &["long", "cut", "fill", "hold"];
const MIX_KINDS: &[&str] = &[
    "delay", "lpf", "flash", "riser", "switch", "echo", "hpf", "roll", "drop", "vinyl", "lane",
];
const DECK_VERBS: &[&str] = &["load", "mute", "unmute", "gain", "head", "x"];
const DECK_VERBS_PLAY: &[&str] = &["load", "mute", "unmute", "gain", "head", "save", "reload"];
const PLAY_ALIAS_VERBS: &[&str] = &[
    "load", "save", "reload", "mute", "unmute", "gain", "head", "cue",
];
const VIZ_ARGS: &[&str] = &["on", "off"];

/// Suggest using live song directories (user library + bundled `songs/`).
pub fn suggest(input: &str, ctx: &CompleteCtx<'_>) -> CompleteResult {
    let songs = collect_song_stems();
    suggest_with_songs(input, ctx, &songs)
}

/// Suggest with an explicit song-stem list (for unit tests).
pub fn suggest_with_songs(
    input: &str,
    ctx: &CompleteCtx<'_>,
    song_stems: &[String],
) -> CompleteResult {
    let Some(parsed) = parse_local_body(input, ctx.hermes_enabled) else {
        return CompleteResult::empty();
    };

    let (tokens, partial, trailing_ws) = tokenize_body(parsed.body);
    let replace_from =
        partial_byte_start(input, parsed.body_start, parsed.body, &tokens, trailing_ws);

    stage_suggest(&tokens, partial, trailing_ws, replace_from, ctx, song_stems)
}

/// Apply `candidates[index]` over the partial token range.
///
/// Always appends a trailing space so the next token (or Enter dispatch) is ready.
pub fn apply_candidate(input: &str, result: &CompleteResult, index: usize) -> Option<String> {
    let cand = result.candidates.get(index)?;
    if result.replace_from > input.len() {
        return None;
    }
    let prefix = &input[..result.replace_from];
    Some(format!("{prefix}{cand} "))
}

// --- internals ---

struct LocalBody<'a> {
    /// Command body after optional `:` / `/` (may include trailing spaces).
    body: &'a str,
    /// Byte index of `body` within the original input.
    body_start: usize,
}

fn parse_local_body(input: &str, hermes_enabled: bool) -> Option<LocalBody<'_>> {
    let bytes = input.as_bytes();
    let mut i = 0usize;
    while i < bytes.len() && bytes[i].is_ascii_whitespace() {
        i += 1;
    }
    if i < bytes.len() && bytes[i] == b':' {
        i += 1;
        while i < bytes.len() && bytes[i].is_ascii_whitespace() {
            i += 1;
        }
    }

    if hermes_enabled {
        if i >= bytes.len() || bytes[i] != b'/' {
            return None;
        }
        i += 1;
    } else if i < bytes.len() && bytes[i] == b'/' {
        i += 1;
    }

    // Body may start with spaces after `/`; keep them so trailing_ws / offsets work.
    Some(LocalBody {
        body: &input[i..],
        body_start: i,
    })
}

/// Returns (complete tokens before partial, partial token, body ends with whitespace after a token).
fn tokenize_body(body: &str) -> (Vec<&str>, &str, bool) {
    let trimmed_end = body.trim_end();
    let trailing_ws = body.len() > trimmed_end.len() && !trimmed_end.is_empty();
    // Also: body is only whitespace after `/` → empty partial, no trailing token ws flag
    if trimmed_end.is_empty() {
        return (Vec::new(), "", false);
    }
    let mut tokens: Vec<&str> = trimmed_end.split_whitespace().collect();
    if trailing_ws {
        (tokens, "", true)
    } else if let Some(last) = tokens.pop() {
        (tokens, last, false)
    } else {
        (Vec::new(), "", false)
    }
}

fn partial_byte_start(
    input: &str,
    body_start: usize,
    body: &str,
    complete_tokens: &[&str],
    trailing_ws: bool,
) -> usize {
    if trailing_ws {
        // Partial is empty at end of input (after whitespace).
        return input.len();
    }
    if complete_tokens.is_empty() {
        // Partial is the first (or only) token: find it in body.
        let trimmed = body.trim_start();
        let lead = body.len() - trimmed.len();
        return body_start + lead;
    }
    // After N complete tokens, partial starts at last whitespace-separated word.
    // Walk body from start.
    let mut rest = body;
    let mut abs = body_start;
    // skip leading ws in body
    let t = rest.trim_start();
    abs += rest.len() - t.len();
    rest = t;
    for tok in complete_tokens {
        if let Some(pos) = rest.find(tok) {
            // should be at start after ws skip
            let _ = pos;
            abs += tok.len();
            rest = &rest[tok.len()..];
            let t2 = rest.trim_start();
            abs += rest.len() - t2.len();
            rest = t2;
        }
    }
    // rest should start with partial
    abs
}

fn stage_suggest(
    complete: &[&str],
    partial: &str,
    trailing_ws: bool,
    replace_from: usize,
    ctx: &CompleteCtx<'_>,
    song_stems: &[String],
) -> CompleteResult {
    let _ = trailing_ws;
    let rewritten;
    let complete = if ctx.session == SessionKind::Play {
        match complete {
            [v, rest @ ..] if PLAY_ALIAS_VERBS.contains(v) => {
                rewritten = {
                    let mut t = Vec::with_capacity(rest.len() + 2);
                    t.push("a");
                    t.push(*v);
                    t.extend_from_slice(rest);
                    t
                };
                rewritten.as_slice()
            }
            _ => complete,
        }
    } else {
        complete
    };
    if ctx.session == SessionKind::Play {
        if let Some("mix" | "x" | "xfade" | "b" | "B" | "1") = complete.first().copied() {
            return CompleteResult::empty();
        }
    }
    let top = if ctx.session == SessionKind::Play {
        TOP_LEVEL_PLAY
    } else {
        TOP_LEVEL
    };
    let deck_verbs = if ctx.session == SessionKind::Play {
        DECK_VERBS_PLAY
    } else {
        DECK_VERBS
    };
    match complete {
        [] => {
            let mut c = filter_static(top, partial);
            if ctx.session == SessionKind::Play {
                for g in filter_owned(song_stems, partial) {
                    if !c.iter().any(|x| x == &g) {
                        c.push(g);
                    }
                }
            }
            list_result(c, replace_from, None)
        }
        [d] if is_deck(d) => {
            if ctx.session == SessionKind::Play && !matches!(*d, "a" | "A" | "0") {
                return CompleteResult::empty();
            }
            let mut c = filter_static(deck_verbs, partial);
            for g in filter_owned(song_stems, partial) {
                if !c.iter().any(|x| x == &g) {
                    c.push(g);
                }
            }
            list_result(c, replace_from, None)
        }
        ["list"] => list_result(filter_owned(song_stems, partial), replace_from, None),
        [g] if ctx.session == SessionKind::Play && is_genre_slug(g) => list_result(
            filter_owned(&list_bundled_slots(g), partial),
            replace_from,
            None,
        ),
        [d, v] if is_deck(d) => match *v {
            "load" => list_result(filter_owned(song_stems, partial), replace_from, None),
            "mute" | "unmute" => {
                let tracks = deck_tracks(d, ctx);
                list_result(filter_owned(tracks, partial), replace_from, None)
            }
            "gain" => hint_only(replace_from, "<0..1>"),
            "head" | "cue" => hint_only(replace_from, "<bar>=1"),
            "x" | "xfade" => hint_only(replace_from, "<bars>"),
            other if is_genre_slug(other) => list_result(
                filter_owned(&list_bundled_slots(other), partial),
                replace_from,
                None,
            ),
            _ => CompleteResult::empty(),
        },
        // Path/track already has at least one completed token after the verb.
        // Keep suggesting only while the user is still typing the last token.
        [d, v, rest @ ..] if is_deck(d) && (*v == "load") => {
            if rest.len() == 1 && is_genre_slug(rest[0]) {
                return list_result(
                    filter_owned(&list_bundled_slots(rest[0]), partial),
                    replace_from,
                    None,
                );
            }
            if partial.is_empty() {
                CompleteResult::empty()
            } else {
                list_result(filter_owned(song_stems, partial), replace_from, None)
            }
        }
        [d, v, _rest @ ..] if is_deck(d) && (*v == "mute" || *v == "unmute") => {
            if partial.is_empty() {
                CompleteResult::empty()
            } else {
                let tracks = deck_tracks(d, ctx);
                list_result(filter_owned(tracks, partial), replace_from, None)
            }
        }
        ["viz"] | ["punchcard"] | ["pianoroll"] => {
            list_result(filter_static(VIZ_ARGS, partial), replace_from, None)
        }
        ["vfx"] | ["dopa"] | ["flash"] => {
            list_result(filter_static(VIZ_ARGS, partial), replace_from, None)
        }
        ["bpm"] => hint_only(replace_from, "<bpm>"),
        ["filter"] => list_result(filter_static(&["lpf", "hpf"], partial), replace_from, None),
        ["filter", "lpf"] | ["filter", "hpf"] => hint_only(replace_from, "<hz>|off"),
        ["delay"] => hint_only(replace_from, "<0..1>"),
        ["vinyl"] => list_result(filter_static(&["on", "off"], partial), replace_from, None),
        ["repeat"] => list_result(filter_static(REPEAT_DIVS, partial), replace_from, None),
        ["tape"] => list_result(filter_static(TAPE_LENS, partial), replace_from, None),
        ["tape", _] => hint_only(replace_from, "<reps 1..8>"),
        ["x"] | ["xfade"] => hint_only(replace_from, "<bars>"),
        ["mix"] => list_result(filter_static(MIX_MOVES, partial), replace_from, None),
        ["mix", "long"] | ["mix", "cut"] => hint_only(replace_from, "A|B"),
        ["mix", "fill"] => list_result(filter_owned(&mix_kind_list(), partial), replace_from, None),
        ["mix", "fill", _] => hint_only(replace_from, "A|B"),
        // Completing first token (complete empty, partial is first word) already handled by [].
        // If complete has one non-deck token and we're still typing more — no further suggest.
        _ => CompleteResult::empty(),
    }
}

fn is_deck(s: &str) -> bool {
    matches!(s, "a" | "A" | "b" | "B" | "0" | "1")
}

fn deck_tracks<'a>(deck: &str, ctx: &'a CompleteCtx<'_>) -> &'a [String] {
    match deck {
        "a" | "A" | "0" => ctx.tracks_a,
        "b" | "B" | "1" => ctx.tracks_b,
        _ => &[],
    }
}

fn mix_kind_list() -> Vec<String> {
    let mut out: Vec<String> = MIX_KINDS.iter().map(|s| (*s).to_string()).collect();
    for slug in crate::mix_recipe::list_mix_slugs() {
        if !out.iter().any(|s| s == &slug) {
            out.push(slug);
        }
    }
    out
}

fn filter_static(items: &[&str], prefix: &str) -> Vec<String> {
    let p = prefix.to_ascii_lowercase();
    items
        .iter()
        .filter(|s| s.to_ascii_lowercase().starts_with(&p))
        .map(|s| (*s).to_string())
        .collect()
}

fn filter_owned(items: &[String], prefix: &str) -> Vec<String> {
    let p = prefix.to_ascii_lowercase();
    items
        .iter()
        .filter(|s| s.to_ascii_lowercase().starts_with(&p))
        .cloned()
        .collect()
}

fn list_result(
    candidates: Vec<String>,
    replace_from: usize,
    hint: Option<String>,
) -> CompleteResult {
    CompleteResult {
        candidates,
        replace_from,
        hint,
    }
}

fn hint_only(replace_from: usize, hint: &str) -> CompleteResult {
    CompleteResult {
        candidates: Vec::new(),
        replace_from,
        hint: Some(hint.to_string()),
    }
}

/// Genre folder names plus user-library stems (not every `house/01` slot).
pub fn collect_song_stems() -> Vec<String> {
    let mut out: Vec<String> = list_bundled_genres().into_iter().map(|g| g.name).collect();
    for name in list_user_library_songs() {
        let stem = Path::new(&name)
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or(name.as_str())
            .to_string();
        if !out.iter().any(|x| x == &stem) {
            out.push(stem);
        }
    }
    out.sort();
    out
}

/// Format a one-line candidate strip for the TUI footer.
/// `selected` highlights one candidate; `max_items` caps listed names.
pub fn format_suggest_line(result: &CompleteResult, selected: usize, max_items: usize) -> String {
    if result.candidates.is_empty() {
        if let Some(ref h) = result.hint {
            return format!("  \x1b[2m{h}\x1b[0m");
        }
        return String::new();
    }
    let n = result.candidates.len().min(max_items.max(1));
    let sel = if result.candidates.is_empty() {
        0
    } else {
        selected % result.candidates.len()
    };
    let mut parts: Vec<String> = Vec::with_capacity(n + 1);
    for (i, c) in result.candidates.iter().take(n).enumerate() {
        if i == sel {
            parts.push(format!("\x1b[7m{c}\x1b[0m"));
        } else {
            parts.push(format!("\x1b[2m{c}\x1b[0m"));
        }
    }
    if result.candidates.len() > n {
        parts.push("\x1b[2m…\x1b[0m".to_string());
    }
    format!("  {}", parts.join("  "))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ctx<'a>(hermes: bool, a: &'a [String], b: &'a [String]) -> CompleteCtx<'a> {
        CompleteCtx {
            hermes_enabled: hermes,
            tracks_a: a,
            tracks_b: b,
            session: SessionKind::Dj,
        }
    }

    fn ctx_play<'a>(hermes: bool, a: &'a [String], b: &'a [String]) -> CompleteCtx<'a> {
        CompleteCtx {
            hermes_enabled: hermes,
            tracks_a: a,
            tracks_b: b,
            session: SessionKind::Play,
        }
    }

    fn songs(names: &[&str]) -> Vec<String> {
        names.iter().map(|s| (*s).to_string()).collect()
    }

    #[test]
    fn hermes_natural_language_no_suggest() {
        let s = songs(&["house"]);
        let r = suggest_with_songs("暗くして", &ctx(true, &[], &[]), &s);
        assert!(!r.has_display());
        assert!(r.candidates.is_empty());
    }

    #[test]
    fn slash_empty_lists_top_level() {
        let r = suggest_with_songs("/", &ctx(true, &[], &[]), &[]);
        assert!(r.candidates.iter().any(|c| c == "a"));
        assert!(r.candidates.iter().any(|c| c == "bpm"));
        assert!(r.candidates.iter().any(|c| c == "viz"));
        assert!(r.candidates.iter().any(|c| c == "vfx"));
    }

    #[test]
    fn slash_bp_completes_bpm() {
        let r = suggest_with_songs("/bp", &ctx(true, &[], &[]), &[]);
        assert_eq!(r.candidates, vec!["bpm".to_string()]);
    }

    #[test]
    fn deck_verb_load() {
        let r = suggest_with_songs("/a l", &ctx(true, &[], &[]), &[]);
        assert_eq!(r.candidates, vec!["load".to_string()]);
    }

    #[test]
    fn load_path_filters_songs() {
        let s = songs(&["techno-duck", "house", "four-on-the-floor"]);
        let r = suggest_with_songs("/a load te", &ctx(true, &[], &[]), &s);
        assert_eq!(r.candidates, vec!["techno-duck".to_string()]);
    }

    #[test]
    fn load_after_space_lists_all() {
        let s = songs(&["house", "four-on-the-floor"]);
        let r = suggest_with_songs("/a load ", &ctx(true, &[], &[]), &s);
        assert_eq!(r.candidates.len(), 2);
        assert!(r.candidates.contains(&"house".to_string()));
    }

    #[test]
    fn deck_suggests_genre_after_verbs() {
        let s = songs(&["house", "acid"]);
        let r = suggest_with_songs("/a h", &ctx(true, &[], &[]), &s);
        assert!(
            r.candidates.iter().any(|c| c == "house"),
            "{:?}",
            r.candidates
        );
        assert!(
            r.candidates.iter().any(|c| c == "head"),
            "{:?}",
            r.candidates
        );
    }

    #[test]
    fn mute_uses_deck_tracks() {
        let tracks = songs(&["kick", "hat", "bass"]);
        let r = suggest_with_songs("/a mute k", &ctx(true, &tracks, &[]), &[]);
        assert_eq!(r.candidates, vec!["kick".to_string()]);
    }

    #[test]
    fn viz_on_off() {
        let r = suggest_with_songs("/viz o", &ctx(true, &[], &[]), &[]);
        assert_eq!(r.candidates, vec!["on".to_string(), "off".to_string()]);
    }

    #[test]
    fn vfx_on_off() {
        let r = suggest_with_songs("/vfx o", &ctx(true, &[], &[]), &[]);
        assert_eq!(r.candidates, vec!["on".to_string(), "off".to_string()]);
    }

    #[test]
    fn bpm_hint_only() {
        let r = suggest_with_songs("/bpm ", &ctx(true, &[], &[]), &[]);
        assert!(r.candidates.is_empty());
        assert_eq!(r.hint.as_deref(), Some("<bpm>"));
    }

    #[test]
    fn apply_preserves_slash_prefix() {
        let s = songs(&["techno-duck", "house"]);
        let r = suggest_with_songs("/a load te", &ctx(true, &[], &[]), &s);
        let applied = apply_candidate("/a load te", &r, 0).unwrap();
        assert_eq!(applied, "/a load techno-duck ");
        let unique = suggest_with_songs("/a load house", &ctx(true, &[], &[]), &s);
        assert_eq!(
            apply_candidate("/a load house", &unique, 0).unwrap(),
            "/a load house "
        );
    }

    #[test]
    fn hermes_off_bare_commands() {
        let r = suggest_with_songs("a l", &ctx(false, &[], &[]), &[]);
        assert_eq!(r.candidates, vec!["load".to_string()]);
    }

    #[test]
    fn colon_slash_legacy() {
        let r = suggest_with_songs(":/st", &ctx(true, &[], &[]), &[]);
        assert_eq!(r.candidates, vec!["status".to_string()]);
    }

    #[test]
    fn play_top_level_has_load_not_mix() {
        let r = suggest_with_songs("/", &ctx_play(true, &[], &[]), &[]);
        assert!(
            r.candidates.iter().any(|c| c == "load"),
            "{:?}",
            r.candidates
        );
        assert!(
            r.candidates.iter().any(|c| c == "bpm"),
            "{:?}",
            r.candidates
        );
        assert!(!r.candidates.iter().any(|c| c == "b"), "{:?}", r.candidates);
        assert!(
            !r.candidates.iter().any(|c| c == "mix"),
            "{:?}",
            r.candidates
        );
        assert!(!r.candidates.iter().any(|c| c == "x"), "{:?}", r.candidates);
    }

    #[test]
    fn play_load_alias_suggests_songs() {
        let s = songs(&["house-01", "dnb-01"]);
        let r = suggest_with_songs("/load ho", &ctx_play(true, &[], &[]), &s);
        assert_eq!(r.candidates, vec!["house-01".to_string()]);
    }
}

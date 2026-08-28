//! Song file format: Strudel-oriented patterns (+ legacy named tracks).
//!
//! Metadata follows Strudel's comment tags: https://strudel.cc/learn/metadata/

use std::path::{Component, Path, PathBuf};

use crate::code::{edit_method_on_code, parse_code, MethodEditOp, PatternCode};

/// Default directory for bare song names (relative to process working directory).
pub const DEFAULT_SONGS_DIR: &str = "songs";

/// User song library under the home directory: `~/.config/strudel-rs/songs`.
/// This is the **only** path where API/MCP may write songs.
pub const USER_SONGS_REL: &str = ".config/strudel-rs/songs";

/// Max UTF-8 byte size for song content accepted by save.
pub const MAX_SONG_CONTENT_BYTES: usize = 256 * 1024;

/// Serialize tests that mutate `HOME` (user library path).
#[cfg(test)]
pub(crate) fn lock_test_home() -> std::sync::MutexGuard<'static, ()> {
    use std::sync::Mutex;
    static LOCK: Mutex<()> = Mutex::new(());
    LOCK.lock().unwrap_or_else(|e| e.into_inner())
}

#[derive(Debug, Clone)]
pub struct Track {
    pub name: String,
    pub code: PatternCode,
    pub muted: bool,
}

/// Optional music metadata from Strudel-style `@tag` comments.
/// Fields are best-effort; malformed values never fail the parse.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct SongMeta {
    pub by: Vec<String>,
    pub license: Vec<String>,
    pub details: Option<String>,
    pub url: Vec<String>,
    pub genre: Vec<String>,
    pub album: Option<String>,
    pub tag: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct Song {
    pub title: String,
    pub meta: SongMeta,
    pub bpm: Option<f64>,
    pub tracks: Vec<Track>,
    pub path: String,
    /// Full original file text (for live highlight display).
    pub source: String,
}

/// Preferred Strudel-like format (copy-paste friendly):
/// ```text
/// // @title smoke
/// // @by strudel-rs
/// setcpm(30)
/// // kick
/// $: s("bd*4").gain(0.9)
/// // hat
/// $: s("hh*8").gain(0.3)
/// ```
///
/// Metadata tags (see https://strudel.cc/learn/metadata/):
/// `@title`, `@by`, `@license`, `@details`, `@url`, `@genre`, `@album`, `@tag`.
/// Also supported: block comments `/* … */`, multi-tag lines, and
/// `// "Quoted Title" @by …` at the start of a comment.
///
/// `setcpm(N)` is cycles-per-minute (Strudel). With 1 cycle = 1 bar of 4 beats,
/// engine BPM is `N * 4`. `setcpm(120/4)` is accepted. `setcps(x)` is also accepted
/// (`bpm = x * 240`).
///
/// Legacy format still works:
/// ```text
/// bpm: 126
/// title: My Song
/// ---
/// kick: s("bd*4").gain(0.9)
/// ```
/// Also: `// title: …` (pre-@tag comment form).
///
/// # Path resolution
/// See [`resolve_song_path`] for bare-name / extension rules used by CLI and API.
///
/// Reject path traversal (`..`). Backslashes count as separators too.
pub fn sanitize_song_path(path: &str) -> Result<PathBuf, String> {
    if path.trim().is_empty() {
        return Err("path is empty".into());
    }
    let normalized = path.replace('\\', "/");
    let p = Path::new(&normalized);
    if p.components().any(|c| matches!(c, Component::ParentDir)) {
        return Err("path must not contain '..'".into());
    }
    if normalized.split('/').any(|s| s == "..") {
        return Err("path must not contain '..'".into());
    }
    Ok(PathBuf::from(path))
}

/// Home directory for user config (`HOME`, then `USERPROFILE`).
pub fn home_dir() -> Result<PathBuf, String> {
    if let Some(h) = std::env::var_os("HOME").filter(|s| !s.is_empty()) {
        return Ok(PathBuf::from(h));
    }
    if let Some(h) = std::env::var_os("USERPROFILE").filter(|s| !s.is_empty()) {
        return Ok(PathBuf::from(h));
    }
    Err("cannot resolve home directory (HOME / USERPROFILE unset)".into())
}

/// Absolute path to `~/.config/strudel-rs/songs`.
pub fn user_songs_dir() -> Result<PathBuf, String> {
    Ok(home_dir()?.join(USER_SONGS_REL))
}

/// Create the user songs directory if missing.
pub fn ensure_user_songs_dir() -> Result<PathBuf, String> {
    let dir = user_songs_dir()?;
    std::fs::create_dir_all(&dir).map_err(|e| format!("create user songs dir: {e}"))?;
    Ok(dir)
}

/// Basenames (`*.strudel` / `*.txt`) in a directory, sorted.
pub fn list_song_basenames_in(dir: &Path) -> Vec<String> {
    let mut out = Vec::new();
    let Ok(rd) = std::fs::read_dir(dir) else {
        return out;
    };
    for ent in rd.flatten() {
        let path = ent.path();
        if !path.is_file() {
            continue;
        }
        if known_song_ext(&path).is_none() {
            continue;
        }
        if let Some(name) = path.file_name().and_then(|s| s.to_str()) {
            out.push(name.to_string());
        }
    }
    out.sort();
    out
}

/// User-library song basenames (`~/.config/strudel-rs/songs/`).
pub fn list_user_library_songs() -> Vec<String> {
    user_songs_dir()
        .map(|d| list_song_basenames_in(&d))
        .unwrap_or_default()
}

/// Bundled demo song basenames under [`DEFAULT_SONGS_DIR`] relative to cwd.
pub fn list_bundled_songs() -> Vec<String> {
    list_song_basenames_in(Path::new(DEFAULT_SONGS_DIR))
}

/// Normalize a user-facing save name to a safe file name (`*.strudel`).
///
/// Accepts only a single path segment: `[A-Za-z0-9._-]+`, optional `.strudel` / `.txt`.
/// Rejects path separators, `..`, absolute paths, and empty names.
pub fn sanitize_user_song_name(name: &str) -> Result<String, String> {
    let name = name.trim();
    if name.is_empty() {
        return Err("song name is empty".into());
    }
    if name.contains('/') || name.contains('\\') {
        return Err("song name must be a basename (no path separators)".into());
    }
    if name == "." || name == ".." || name.contains("..") {
        return Err("song name must not contain '..'".into());
    }
    // Windows drive / UNC style
    if name.contains(':') {
        return Err("song name must not contain ':'".into());
    }
    let (stem, ext) = match known_song_ext(Path::new(name)) {
        Some(e) => {
            let stem = Path::new(name)
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("");
            (stem, e)
        }
        None => {
            if Path::new(name).extension().is_some() {
                return Err("song name extension must be .strudel or .txt (or omit)".into());
            }
            (name, "strudel")
        }
    };
    if stem.is_empty() || stem == "." || stem == ".." {
        return Err("song name stem is empty".into());
    }
    if !stem
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || c == '.' || c == '_' || c == '-')
    {
        return Err("song name may only use ASCII letters, digits, '.', '_' and '-'".into());
    }
    // Prefer .strudel for the user library; allow .txt if explicitly given.
    Ok(format!("{stem}.{ext}"))
}

/// Resolve a save path under the user songs directory only.
pub fn resolve_user_song_save_path(name: &str) -> Result<PathBuf, String> {
    let file_name = sanitize_user_song_name(name)?;
    let dir = user_songs_dir()?;
    let path = dir.join(&file_name);
    // Defense in depth: joined path must stay under user_songs_dir.
    let dir_canon = dir.components().collect::<Vec<_>>();
    let path_comps = path.components().collect::<Vec<_>>();
    if path_comps.len() != dir_canon.len() + 1 {
        return Err("refusing path outside user songs directory".into());
    }
    if path.file_name().and_then(|s| s.to_str()) != Some(file_name.as_str()) {
        return Err("refusing path outside user songs directory".into());
    }
    if path.components().any(|c| matches!(c, Component::ParentDir)) {
        return Err("refusing path outside user songs directory".into());
    }
    Ok(path)
}

fn known_song_ext(path: &Path) -> Option<&'static str> {
    match path
        .extension()
        .and_then(|e| e.to_str())
        .map(|e| e.to_ascii_lowercase())
        .as_deref()
    {
        Some("strudel") => Some("strudel"),
        Some("txt") => Some("txt"),
        _ => None,
    }
}

fn has_dir_component(path: &Path) -> bool {
    path.components().count() > 1
}

/// Candidate paths for a user-supplied song reference (order = preference).
///
/// - Bare names (no directory): **user library** (`~/.config/strudel-rs/songs`) →
///   [`DEFAULT_SONGS_DIR`] → cwd.
/// - Missing `.strudel` / `.txt` is filled in; **`.strudel` before `.txt`**.
/// - Paths with a directory (e.g. `songs/foo.strudel`): try the path as written,
///   then also resolve the **basename** the same way as a bare name. Models often
///   pass `songs/<saved-name>` for user-library tracks that only live under
///   `~/.config/strudel-rs/songs/`.
pub fn song_path_candidates(input: &str) -> Result<Vec<PathBuf>, String> {
    let _ = sanitize_song_path(input)?;
    let p = Path::new(input.trim());
    let mut out = Vec::new();

    let push = |out: &mut Vec<PathBuf>, c: PathBuf| {
        if !out.iter().any(|x| x == &c) {
            out.push(c);
        }
    };

    let user_dir = user_songs_dir().ok();

    match known_song_ext(p) {
        Some(_) => {
            if has_dir_component(p) {
                push(&mut out, p.to_path_buf());
            } else {
                // bare `smoke.strudel` → user lib → songs/ → cwd
                if let Some(ref ud) = user_dir {
                    push(&mut out, ud.join(p));
                }
                push(&mut out, PathBuf::from(DEFAULT_SONGS_DIR).join(p));
                push(&mut out, p.to_path_buf());
            }
        }
        None if p.extension().is_some() => {
            // Other extension: treat as literal path (still allow bare under songs/).
            if has_dir_component(p) {
                push(&mut out, p.to_path_buf());
            } else {
                if let Some(ref ud) = user_dir {
                    push(&mut out, ud.join(p));
                }
                push(&mut out, PathBuf::from(DEFAULT_SONGS_DIR).join(p));
                push(&mut out, p.to_path_buf());
            }
        }
        None => {
            // No extension: try .strudel then .txt
            if has_dir_component(p) {
                push(&mut out, p.with_extension("strudel"));
                push(&mut out, p.with_extension("txt"));
            } else {
                let name = p.as_os_str();
                if let Some(ref ud) = user_dir {
                    push(&mut out, ud.join(name).with_extension("strudel"));
                    push(&mut out, ud.join(name).with_extension("txt"));
                }
                push(
                    &mut out,
                    PathBuf::from(DEFAULT_SONGS_DIR)
                        .join(name)
                        .with_extension("strudel"),
                );
                push(
                    &mut out,
                    PathBuf::from(DEFAULT_SONGS_DIR)
                        .join(name)
                        .with_extension("txt"),
                );
                push(&mut out, PathBuf::from(name).with_extension("strudel"));
                push(&mut out, PathBuf::from(name).with_extension("txt"));
            }
        }
    }

    // Basename fallback: `songs/visitor-dnb.strudel` → also try user library / songs/
    // as if the user passed bare `visitor-dnb.strudel` or `visitor-dnb`.
    if has_dir_component(p) {
        if let Some(file_name) = p.file_name() {
            let bare = Path::new(file_name);
            match known_song_ext(bare) {
                Some(_) => {
                    if let Some(ref ud) = user_dir {
                        push(&mut out, ud.join(bare));
                    }
                    push(&mut out, PathBuf::from(DEFAULT_SONGS_DIR).join(bare));
                    push(&mut out, bare.to_path_buf());
                }
                None if bare.extension().is_some() => {
                    if let Some(ref ud) = user_dir {
                        push(&mut out, ud.join(bare));
                    }
                    push(&mut out, PathBuf::from(DEFAULT_SONGS_DIR).join(bare));
                    push(&mut out, bare.to_path_buf());
                }
                None => {
                    let name = bare.as_os_str();
                    if let Some(ref ud) = user_dir {
                        push(&mut out, ud.join(name).with_extension("strudel"));
                        push(&mut out, ud.join(name).with_extension("txt"));
                    }
                    push(
                        &mut out,
                        PathBuf::from(DEFAULT_SONGS_DIR)
                            .join(name)
                            .with_extension("strudel"),
                    );
                    push(
                        &mut out,
                        PathBuf::from(DEFAULT_SONGS_DIR)
                            .join(name)
                            .with_extension("txt"),
                    );
                    push(&mut out, PathBuf::from(name).with_extension("strudel"));
                    push(&mut out, PathBuf::from(name).with_extension("txt"));
                }
            }
        }
    }

    Ok(out)
}

/// Resolve a user song path to an existing file.
///
/// See [`song_path_candidates`] for rules (default `songs/`, optional extension).
pub fn resolve_song_path(input: &str) -> Result<PathBuf, String> {
    let candidates = song_path_candidates(input)?;
    for c in &candidates {
        if c.is_file() {
            return Ok(c.clone());
        }
    }
    let tried = candidates
        .iter()
        .map(|c| c.display().to_string())
        .collect::<Vec<_>>()
        .join(", ");
    Err(format!("song not found: {input} (tried: {tried})"))
}

pub fn parse_song(text: &str, path: &str) -> Result<Song, String> {
    let mut bpm = None;
    let mut title: Option<String> = None;
    let mut meta = SongMeta::default();
    let mut tracks = Vec::new();
    let mut pending_label: Option<String> = None;
    let mut anon_idx = 0usize;
    let mut block_buf: Option<String> = None;

    // Index lines with absolute byte offsets so multi-line track code keeps
    // correct mini-notation spans for live highlight.
    let mut line_starts: Vec<(usize, &str)> = Vec::new();
    {
        let mut offset = 0usize;
        for raw in text.split_inclusive('\n') {
            line_starts.push((offset, raw));
            offset += raw.len();
        }
    }

    let mut i = 0usize;
    while i < line_starts.len() {
        let (line_start, raw) = line_starts[i];
        let lineno = i;
        // Strip trailing newline for parsing (keep line_start for absolute offsets).
        let line_no_nl = raw.trim_end_matches(['\r', '\n']);
        let line = line_no_nl.trim();

        // Inside `/* … */` block comment: accumulate and apply metadata on close.
        if let Some(ref mut buf) = block_buf {
            if let Some(end) = line_no_nl.find("*/") {
                buf.push_str(&line_no_nl[..end]);
                let body = buf.clone();
                block_buf = None;
                apply_metadata_text(&body, &mut title, &mut meta);
                pending_label = None;
                // Trailing code after `*/` on the same line is not supported.
                let _ = end;
            } else {
                buf.push_str(line_no_nl);
                buf.push('\n');
            }
            i += 1;
            continue;
        }

        if line.is_empty() {
            i += 1;
            continue;
        }

        // Start of block comment (full-line or line beginning with /*).
        if let Some(rest) = line.strip_prefix("/*") {
            if let Some(end) = rest.find("*/") {
                apply_metadata_text(&rest[..end], &mut title, &mut meta);
                pending_label = None;
            } else {
                block_buf = Some(rest.to_string());
                if let Some(b) = block_buf.as_mut() {
                    b.push('\n');
                }
            }
            i += 1;
            continue;
        }

        // Line comments: Strudel @tags, legacy title:, or label for the next `$:` track.
        if let Some(comment) = strip_line_comment(line) {
            handle_comment_body(comment, &mut title, &mut meta, &mut pending_label);
            i += 1;
            continue;
        }

        if line == "---" {
            i += 1;
            continue;
        }

        // Tempo: setcpm / setcps (Strudel) or bpm: / title: (legacy header).
        if let Some(cpm) = parse_setcpm(line) {
            bpm = Some(cpm * 4.0);
            i += 1;
            continue;
        }
        if let Some(cps) = parse_setcps(line) {
            bpm = Some(cps * 240.0);
            i += 1;
            continue;
        }
        if let Some(v) = line.strip_prefix("bpm:") {
            bpm = Some(
                v.trim()
                    .parse()
                    .map_err(|_| format!("line {}: bad bpm", lineno + 1))?,
            );
            i += 1;
            continue;
        }
        if let Some(v) = line.strip_prefix("title:") {
            let t = v.trim();
            if !t.is_empty() {
                title = Some(t.to_string());
            }
            i += 1;
            continue;
        }

        // Track: `$:` (anonymous / comment-labeled) or `name: code`.
        // Code may span multiple lines (newlines inside `"..."` or `.method` continuations).
        let colon = line_no_nl
            .find(':')
            .ok_or_else(|| format!("line {}: expected 'name: code' or '$: code'", lineno + 1))?;
        let name_part = line_no_nl[..colon].trim();
        if name_part.is_empty() {
            return Err(format!("line {}: empty track name", lineno + 1));
        }
        let name = if name_part == "$" {
            if let Some(label) = pending_label.take() {
                label
            } else {
                let n = format!("${anon_idx}");
                anon_idx += 1;
                n
            }
        } else {
            pending_label = None;
            name_part.to_string()
        };

        // Absolute start of the code portion in the full source.
        let code_abs = line_start + colon + 1;
        let (code_end, last_line) = find_track_code_end(text, code_abs, &line_starts, i)?;
        let code_raw = &text[code_abs..code_end];
        let code = parse_code(code_raw)
            .map_err(|e| format!("line {} ({}): {}", lineno + 1, name, e))?
            .with_source_base(code_abs);
        tracks.push(Track {
            name,
            code,
            muted: false,
        });
        i = last_line + 1;
    }

    if block_buf.is_some() {
        // Unclosed block comment: ignore remainder (do not fail the song).
    }

    if tracks.is_empty() {
        return Err("no tracks".into());
    }

    let title = title.unwrap_or_else(|| {
        PathLike::file_name(path)
            .map(|s| s.to_string())
            .unwrap_or_else(|| path.to_string())
    });

    Ok(Song {
        title,
        meta,
        bpm,
        tracks,
        path: path.to_string(),
        source: text.to_string(),
    })
}

/// Keep metadata / tempo lines from the top of `source` (everything before the first track).
///
/// Comment lines that only label the next `$:` track (`// drums`) are **excluded** so
/// [`rebuild_song_source`] can re-emit `// name` without duplication.
pub fn extract_song_preamble(source: &str) -> String {
    let lines: Vec<&str> = source.split_inclusive('\n').collect();
    let mut out = String::new();
    let mut i = 0usize;
    let mut block = false;

    while i < lines.len() {
        let raw = lines[i];
        let line_no_nl = raw.trim_end_matches(['\r', '\n']);
        let line = line_no_nl.trim();

        if block {
            out.push_str(raw);
            if line_no_nl.contains("*/") {
                block = false;
            }
            i += 1;
            continue;
        }
        if line.starts_with("/*") && !line.contains("*/") {
            out.push_str(raw);
            block = true;
            i += 1;
            continue;
        }

        if is_track_start_line(line_no_nl) {
            break;
        }

        // Skip track-label comments: `// drums` immediately before `$:` / `name:`.
        if strip_line_comment(line).is_some() {
            let mut j = i + 1;
            while j < lines.len() {
                let peek = lines[j].trim_end_matches(['\r', '\n']).trim();
                if peek.is_empty() {
                    j += 1;
                    continue;
                }
                break;
            }
            if j < lines.len()
                && is_track_start_line(lines[j].trim_end_matches(['\r', '\n']))
                && !is_metadata_comment(line)
            {
                // Leave label comments for rebuild_song_source.
                break;
            }
        }

        out.push_str(raw);
        i += 1;
    }
    out
}

fn is_metadata_comment(line: &str) -> bool {
    let Some(body) = strip_line_comment(line) else {
        return false;
    };
    let b = body.trim();
    b.starts_with('@') || b.starts_with("title:") || b.starts_with('"') // quoted title form
}

fn is_track_start_line(line_no_nl: &str) -> bool {
    let line = line_no_nl.trim();
    if line.is_empty() || line.starts_with("//") || line.starts_with("/*") || line == "---" {
        return false;
    }
    if parse_setcpm(line).is_some()
        || parse_setcps(line).is_some()
        || line.starts_with("bpm:")
        || line.starts_with("title:")
    {
        return false;
    }
    let Some(colon) = line_no_nl.find(':') else {
        return false;
    };
    let name_part = line_no_nl[..colon].trim();
    !name_part.is_empty()
}

/// Rebuild a full `.strudel` source from preamble + tracks (`// name` + `$: code`).
pub fn rebuild_song_source(preamble: &str, tracks: &[(String, String)]) -> String {
    let mut out = preamble.to_string();
    if !out.is_empty() && !out.ends_with('\n') {
        out.push('\n');
    }
    for (name, code) in tracks {
        let code = code.trim().trim_end_matches(';').trim();
        // Named tracks get a label comment; `$0`-style anon names stay unlabeled.
        if !name.is_empty() && !name.starts_with('$') {
            out.push_str("// ");
            out.push_str(name);
            out.push('\n');
        }
        out.push_str("$: ");
        out.push_str(code);
        out.push('\n');
    }
    out
}

/// Snapshot of track name + raw chain text for API/MCP.
#[derive(Debug, Clone)]
pub struct TrackSource {
    pub name: String,
    pub code: String,
    pub muted: bool,
}

impl Song {
    pub fn track_sources(&self) -> Vec<TrackSource> {
        self.tracks
            .iter()
            .map(|t| TrackSource {
                name: t.name.clone(),
                code: t.code.raw.clone(),
                muted: t.muted,
            })
            .collect()
    }

    fn find_track_index(&self, track: &str) -> Result<usize, String> {
        let track = track.trim();
        if track.is_empty() {
            return Err("track name required".into());
        }
        if let Ok(i) = track.parse::<usize>() {
            if i < self.tracks.len() {
                return Ok(i);
            }
            return Err(format!(
                "track index {i} out of range (0..{})",
                self.tracks.len()
            ));
        }
        self.tracks
            .iter()
            .position(|t| t.name == track)
            .ok_or_else(|| format!("track not found: {track}"))
    }

    /// Replace / remove / append a track, rebuild source, and re-parse into a new [`Song`].
    pub fn patch_track(
        &self,
        track: &str,
        op: &str,
        code: Option<&str>,
        new_name: Option<&str>,
    ) -> Result<Song, String> {
        let mut pairs: Vec<(String, String)> = self
            .tracks
            .iter()
            .map(|t| (t.name.clone(), t.code.raw.clone()))
            .collect();
        let op = op.trim().to_ascii_lowercase();
        match op.as_str() {
            "replace" => {
                let i = self.find_track_index(track)?;
                let code = code
                    .map(str::trim)
                    .filter(|s| !s.is_empty())
                    .ok_or_else(|| "code required for replace".to_string())?;
                parse_code(code)?;
                pairs[i].1 = code.to_string();
                if let Some(n) = new_name.map(str::trim).filter(|s| !s.is_empty()) {
                    pairs[i].0 = n.to_string();
                }
            }
            "remove" => {
                let i = self.find_track_index(track)?;
                if pairs.len() == 1 {
                    return Err("cannot remove the last track".into());
                }
                pairs.remove(i);
            }
            "append" => {
                let code = code
                    .map(str::trim)
                    .filter(|s| !s.is_empty())
                    .ok_or_else(|| "code required for append".to_string())?;
                parse_code(code)?;
                let name = new_name
                    .map(str::trim)
                    .filter(|s| !s.is_empty())
                    .map(|s| s.to_string())
                    .unwrap_or_else(|| format!("${}", pairs.len()));
                pairs.push((name, code.to_string()));
            }
            other => {
                return Err(format!(
                    "unknown patch op: {other} (use replace, remove, or append)"
                ));
            }
        }

        let preamble = extract_song_preamble(&self.source);
        let source = rebuild_song_source(&preamble, &pairs);
        let mut song = parse_song(&source, &self.path)?;
        // Preserve runtime mute flags by name when possible.
        for t in &mut song.tracks {
            if let Some(old) = self.tracks.iter().find(|o| o.name == t.name) {
                t.muted = old.muted;
            }
        }
        Ok(song)
    }

    /// Edit one method on a track chain (`set` / `add` / `remove`).
    pub fn edit_method(
        &self,
        track: &str,
        op: &str,
        method: &str,
        args: &str,
    ) -> Result<Song, String> {
        let i = self.find_track_index(track)?;
        let method_op = MethodEditOp::parse(op)?;
        let new_code = edit_method_on_code(&self.tracks[i].code.raw, method_op, method, args)?;
        self.patch_track(&self.tracks[i].name, "replace", Some(&new_code), None)
    }
}

/// End offset (exclusive) of line content without trailing `\r`/`\n`.
fn line_content_end(line_starts: &[(usize, &str)], idx: usize) -> usize {
    let (start, raw) = line_starts[idx];
    start + raw.trim_end_matches(['\r', '\n']).len()
}

/// True when `s` has balanced `()` outside of `"..."` and no open string.
fn expr_balanced(s: &str) -> bool {
    let mut in_str = false;
    let mut depth = 0i32;
    for c in s.chars() {
        if in_str {
            if c == '"' {
                in_str = false;
            }
            continue;
        }
        match c {
            '"' => in_str = true,
            '(' => depth += 1,
            ')' => {
                depth -= 1;
                if depth < 0 {
                    return false;
                }
            }
            _ => {}
        }
    }
    !in_str && depth == 0
}

/// Line at `from` if non-empty; `None` if out of range or blank (blank breaks continuation).
fn next_significant_line<'a>(
    line_starts: &[(usize, &'a str)],
    from: usize,
) -> Option<(usize, &'a str)> {
    let raw = line_starts.get(from)?.1;
    let t = raw.trim_end_matches(['\r', '\n']).trim();
    if t.is_empty() {
        None
    } else {
        Some((from, t))
    }
}

/// Span of track code starting at `code_start` on `start_line`.
/// Continues while quotes/parens are open, or the next significant line is a `.method(...)`.
fn find_track_code_end(
    text: &str,
    code_start: usize,
    line_starts: &[(usize, &str)],
    start_line: usize,
) -> Result<(usize, usize), String> {
    let mut last_line = start_line;
    loop {
        let end = line_content_end(line_starts, last_line);
        let frag = &text[code_start..end];
        if !expr_balanced(frag) {
            if last_line + 1 >= line_starts.len() {
                return Err(format!(
                    "line {}: unclosed string or parenthesis in track code",
                    start_line + 1
                ));
            }
            last_line += 1;
            continue;
        }
        // Balanced: absorb following lines that continue the method chain with `.…`.
        match next_significant_line(line_starts, last_line + 1) {
            Some((idx, trimmed)) if trimmed.starts_with('.') => {
                last_line = idx;
            }
            _ => return Ok((end, last_line)),
        }
    }
}

fn strip_line_comment(line: &str) -> Option<&str> {
    if let Some(rest) = line.strip_prefix("//") {
        return Some(rest.trim());
    }
    if let Some(rest) = line.strip_prefix('#') {
        return Some(rest.trim());
    }
    None
}

/// Apply one comment body: metadata tags and/or track label for the next `$:`.
fn handle_comment_body(
    comment: &str,
    title: &mut Option<String>,
    meta: &mut SongMeta,
    pending_label: &mut Option<String>,
) {
    if comment.is_empty() {
        return;
    }

    // Legacy: `// title: My Song`
    if let Some(t) = comment.strip_prefix("title:") {
        let t = t.trim();
        if !t.is_empty() {
            *title = Some(t.to_string());
        }
        *pending_label = None;
        return;
    }

    let had_meta = apply_metadata_text(comment, title, meta);
    if had_meta {
        // Pure metadata (or metadata + free prefix) is not a track label.
        *pending_label = None;
    } else if !comment.is_empty() {
        // Immediate previous non-meta comment becomes `$:` track name.
        *pending_label = Some(comment.to_string());
    }
}

/// Parse Strudel metadata from free text (comment line or block body).
/// Returns true if any recognized `@tag` or quoted title was applied.
fn apply_metadata_text(text: &str, title: &mut Option<String>, meta: &mut SongMeta) -> bool {
    let text = text.trim();
    if text.is_empty() {
        return false;
    }

    let mut applied = false;
    let mut rest = text;

    // Alternative title: `"My Cool Song" @by …` at the very beginning.
    if let Some(stripped) = rest.strip_prefix('"') {
        if let Some(end) = stripped.find('"') {
            let t = stripped[..end].trim();
            if !t.is_empty() {
                *title = Some(t.to_string());
                applied = true;
            }
            rest = stripped[end + 1..].trim();
        }
    }

    // Collect `@tag` start positions (known tags only, word-boundary style).
    let tags = find_meta_tag_spans(rest);
    if tags.is_empty() {
        return applied;
    }
    applied = true;

    for (i, (pos, tag)) in tags.iter().enumerate() {
        let value_start = pos + 1 + tag.len(); // skip '@' + name
        let value_end = tags.get(i + 1).map(|(p, _)| *p).unwrap_or(rest.len());
        if value_start > value_end {
            continue;
        }
        let raw = rest[value_start..value_end].trim();
        apply_tag(tag, raw, title, meta);
    }
    applied
}

const META_TAGS: &[&str] = &[
    "title", "by", "license", "details", "url", "genre", "album", "tag",
];

/// Find `@tag` occurrences; longer tag names win if overlapping (none do today).
fn find_meta_tag_spans(text: &str) -> Vec<(usize, &'static str)> {
    let mut found: Vec<(usize, &'static str)> = Vec::new();
    let bytes = text.as_bytes();
    let mut i = 0usize;
    while i < bytes.len() {
        if bytes[i] == b'@' {
            let after = &text[i + 1..];
            let mut matched: Option<&'static str> = None;
            for tag in META_TAGS {
                if let Some(rest) = after.strip_prefix(tag) {
                    let boundary = rest.chars().next();
                    let ok = match boundary {
                        None => true,
                        Some(c) => c.is_whitespace() || c == ',' || c == '\r' || c == '\n',
                    };
                    if ok {
                        // Prefer longer match if several match (not needed for current set).
                        if matched.map(|m| tag.len() > m.len()).unwrap_or(true) {
                            matched = Some(*tag);
                        }
                    }
                }
            }
            if let Some(tag) = matched {
                // Only accept if previous char is start/whitespace (avoid email-like noise).
                let prev_ok = i == 0
                    || text[..i]
                        .chars()
                        .next_back()
                        .map(|c| c.is_whitespace())
                        .unwrap_or(true);
                if prev_ok {
                    found.push((i, tag));
                    i += 1 + tag.len();
                    continue;
                }
            }
        }
        i += 1;
    }
    found
}

fn apply_tag(tag: &str, value: &str, title: &mut Option<String>, meta: &mut SongMeta) {
    match tag {
        "title" => {
            let v = first_line_value(value);
            if !v.is_empty() {
                *title = Some(v);
            }
        }
        "by" => push_list_values(&mut meta.by, value),
        "license" => push_list_values(&mut meta.license, value),
        "details" => {
            let v = collapse_ws_multiline(value);
            if !v.is_empty() {
                meta.details = Some(v);
            }
        }
        "url" => push_list_values(&mut meta.url, value),
        "genre" => push_list_values(&mut meta.genre, value),
        "album" => {
            let v = first_line_value(value);
            if !v.is_empty() {
                meta.album = Some(v);
            }
        }
        "tag" => push_list_values(&mut meta.tag, value),
        _ => {}
    }
}

fn first_line_value(value: &str) -> String {
    value
        .lines()
        .map(str::trim)
        .find(|l| !l.is_empty())
        .unwrap_or("")
        .to_string()
}

fn collapse_ws_multiline(value: &str) -> String {
    value
        .lines()
        .map(str::trim)
        .filter(|l| !l.is_empty())
        .collect::<Vec<_>>()
        .join(" ")
}

/// Comma- or newline-separated list values (Strudel multi-value tags).
fn push_list_values(out: &mut Vec<String>, value: &str) {
    for part in value.split(&[',', '\n', '\r'][..]) {
        let p = part.trim();
        if !p.is_empty() {
            out.push(p.to_string());
        }
    }
}

/// Parse `setcpm(30)`, `setcpm(120/4)`, optional trailing `;`.
fn parse_setcpm(line: &str) -> Option<f64> {
    parse_call_number(line, "setcpm")
}

/// Parse `setcps(0.5)`, `setcps(1/2)`, optional trailing `;`.
fn parse_setcps(line: &str) -> Option<f64> {
    parse_call_number(line, "setcps")
}

fn parse_call_number(line: &str, name: &str) -> Option<f64> {
    let line = line.trim().trim_end_matches(';').trim();
    let rest = line.strip_prefix(name)?.trim_start();
    let inner = rest.strip_prefix('(')?.strip_suffix(')')?.trim();
    parse_number_or_div(inner)
}

/// `90`, `30.5`, or `120/4` (no nested expressions).
fn parse_number_or_div(s: &str) -> Option<f64> {
    let s = s.trim();
    if let Some((a, b)) = s.split_once('/') {
        let num: f64 = a.trim().parse().ok()?;
        let den: f64 = b.trim().parse().ok()?;
        if den == 0.0 {
            return None;
        }
        Some(num / den)
    } else {
        s.parse().ok()
    }
}

/// Minimal path helper without pulling in Path for display titles.
struct PathLike;
impl PathLike {
    fn file_name(path: &str) -> Option<&str> {
        path.rsplit(['/', '\\']).next().filter(|s| !s.is_empty())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_song() {
        let text = r#"
// demo
bpm: 126
title: Demo
---
kick: s("bd*4").gain(0.9)
bass: note("c2 eb2 g2 bb2").s("sawtooth").lpf(400).gain(0.7)
"#;
        let s = parse_song(text, "songs/demo.strudel").unwrap();
        assert_eq!(s.title, "Demo");
        assert_eq!(s.bpm, Some(126.0));
        assert_eq!(s.tracks.len(), 2);
        assert_eq!(s.tracks[0].name, "kick");
        assert!(!s.tracks[0].code.is_note);
        assert!(s.tracks[1].code.is_note);
        assert_eq!(s.source, text);
    }

    #[test]
    fn parses_strudel_style() {
        let text = r#"
// @title smoke
setcpm(30)
// kick
$: s("bd*4").gain(0.9)
// hat
$: s("hh*8").gain(0.3)
// bass
$: note("c2 eb2 g2 bb2").s("sawtooth").lpf(400).gain(0.55)
"#;
        let s = parse_song(text, "songs/smoke.strudel").unwrap();
        assert_eq!(s.title, "smoke");
        // setcpm(30) → 1 cycle/bar of 4 beats → BPM 120
        assert_eq!(s.bpm, Some(120.0));
        assert_eq!(s.tracks.len(), 3);
        assert_eq!(s.tracks[0].name, "kick");
        assert_eq!(s.tracks[1].name, "hat");
        assert_eq!(s.tracks[2].name, "bass");
        assert!(s.tracks[2].code.is_note);
        assert_eq!(s.tracks[0].code.mini_src, "bd*4");
    }

    #[test]
    fn parses_strudel_metadata_tags() {
        let text = r#"
// @title My Cool Song
// @by John Doe <https://example.com>
// @license CC-BY-SA-4.0
// @genre techno, ambient
setcpm(30)
$: s("bd")
"#;
        let s = parse_song(text, "t").unwrap();
        assert_eq!(s.title, "My Cool Song");
        assert_eq!(s.meta.by, vec!["John Doe <https://example.com>"]);
        assert_eq!(s.meta.license, vec!["CC-BY-SA-4.0"]);
        assert_eq!(s.meta.genre, vec!["techno", "ambient"]);
    }

    #[test]
    fn multi_tag_one_line_and_quoted_title() {
        let text = r#"
// "My Cool Song" @by John Doe @license CC0-1.0
setcpm(30)
$: s("bd")
"#;
        let s = parse_song(text, "t").unwrap();
        assert_eq!(s.title, "My Cool Song");
        assert_eq!(s.meta.by, vec!["John Doe"]);
        assert_eq!(s.meta.license, vec!["CC0-1.0"]);
    }

    #[test]
    fn block_comment_metadata() {
        let text = r#"
/*
@title Block Title
@by Jane Doe
@details Line one.
         Line two.
*/
setcpm(30)
$: s("bd")
"#;
        let s = parse_song(text, "t").unwrap();
        assert_eq!(s.title, "Block Title");
        assert_eq!(s.meta.by, vec!["Jane Doe"]);
        assert_eq!(s.meta.details.as_deref(), Some("Line one. Line two."));
    }

    #[test]
    fn metadata_comment_is_not_track_label() {
        let text = r#"
// @title t
// @by author
$: s("bd")
"#;
        let s = parse_song(text, "t").unwrap();
        assert_eq!(s.tracks[0].name, "$0");
        assert_eq!(s.title, "t");
    }

    #[test]
    fn legacy_comment_title_still_works() {
        let text = "// title: legacy\nsetcpm(30)\n$: s(\"bd\")\n";
        let s = parse_song(text, "t").unwrap();
        assert_eq!(s.title, "legacy");
    }

    #[test]
    fn setcpm_division_and_setcps() {
        let a = parse_song("setcpm(120/4)\n$: s(\"bd\")\n", "t").unwrap();
        assert_eq!(a.bpm, Some(120.0));
        let b = parse_song("setcps(0.5)\n$: s(\"bd\")\n", "t").unwrap();
        // 0.5 cps = 30 cpm → BPM 120
        assert_eq!(b.bpm, Some(120.0));
    }

    #[test]
    fn dollar_without_label_gets_index() {
        let s = parse_song("$: s(\"bd\")\n$: s(\"hh\")\n", "t").unwrap();
        assert_eq!(s.tracks[0].name, "$0");
        assert_eq!(s.tracks[1].name, "$1");
    }

    #[test]
    fn reports_error_line() {
        let err = parse_song("---\nbad: nope(\"x\")", "t").unwrap_err();
        assert!(err.contains("line 2"), "{err}");
    }

    #[test]
    fn title_from_path_when_missing() {
        let s = parse_song(
            r#"---
kick: s("bd")
"#,
            "songs/foo.strudel",
        )
        .unwrap();
        assert_eq!(s.title, "foo.strudel");
    }

    #[test]
    fn absolute_mini_span_matches_source() {
        let text = "bpm: 120\n---\nkick: s(\"bd*4\").gain(0.9)\n";
        let s = parse_song(text, "t").unwrap();
        let pc = &s.tracks[0].code;
        assert_eq!(pc.mini_src, "bd*4");
        let base = pc.mini_base;
        assert_eq!(&s.source[base..base + 4], "bd*4");

        // Atom span relative + base → absolute in file
        let ev = crate::mini::events(&pc.pattern, 0);
        let sp = ev[0].span.unwrap();
        let abs = sp.offset(base);
        assert_eq!(&s.source[abs.start..abs.end], "bd");
    }

    #[test]
    fn dollar_mini_span_matches_source() {
        let text = "// kick\n$: s(\"bd*4\").gain(0.9)\n";
        let s = parse_song(text, "t").unwrap();
        let pc = &s.tracks[0].code;
        let base = pc.mini_base;
        assert_eq!(&s.source[base..base + 4], "bd*4");
    }

    #[test]
    fn multiline_mini_string_and_method_chain() {
        let text = r#"// kick
$: s("<
[bd*4]
[bd*4]
[bd bd bd ~]
>")
.gain(0.9)
// hat
$: s("hh*8")
.gain(0.3)
"#;
        let s = parse_song(text, "t").unwrap();
        assert_eq!(s.tracks.len(), 2);
        assert_eq!(s.tracks[0].name, "kick");
        assert!((s.tracks[0].code.gain - 0.9).abs() < 1e-9);
        assert_eq!(s.tracks[1].name, "hat");
        assert!((s.tracks[1].code.gain - 0.3).abs() < 1e-9);
        // Mini source keeps newlines; tokenizer treats them as whitespace.
        assert!(s.tracks[0].code.mini_src.contains('\n'));
        assert!(s.tracks[0].code.mini_src.contains("[bd*4]"));
        let base = s.tracks[0].code.mini_base;
        assert_eq!(&s.source[base..base + 1], "<");
        // Stack has 3 cycle items.
        let n = &s.tracks[0].code.pattern;
        match n {
            crate::mini::Node::Seq(v) => match &v[0] {
                crate::mini::Node::Stack(items) => assert_eq!(items.len(), 3),
                other => panic!("expected Stack, got {other:?}"),
            },
            other => panic!("expected Seq, got {other:?}"),
        }
    }

    #[test]
    fn patch_track_replace_preserves_other_tracks() {
        let text = r#"// @title demo
setcpm(30)
// drums
$: s("bd*4").gain(0.9)
// bass
$: note("c2").s("sawtooth").lpf(400).gain(0.7)
"#;
        let s = parse_song(text, "demo.strudel").unwrap();
        let s2 = s
            .patch_track(
                "bass",
                "replace",
                Some(r#"note("c2").s("sine").lpf(200).gain(0.6)"#),
                None,
            )
            .unwrap();
        assert_eq!(s2.tracks.len(), 2);
        assert_eq!(s2.tracks[0].name, "drums");
        assert_eq!(s2.tracks[1].name, "bass");
        assert_eq!(s2.tracks[1].code.sound, "sine");
        assert_eq!(s2.tracks[1].code.lpf(), Some(200.0));
        assert!(s2.source.contains("setcpm(30)"));
        assert!(s2.source.contains("// drums"));
        assert!(s2.source.contains("// bass"));
        // Drums chain unchanged.
        assert!(s2.tracks[0].code.raw.contains("bd*4"));
    }

    #[test]
    fn edit_method_adds_lpf_on_one_track() {
        let text = r#"// @title demo
setcpm(30)
// bass
$: note("c2").s("sawtooth").gain(0.7)
// hat
$: s("hh*8").gain(0.3)
"#;
        let s = parse_song(text, "demo.strudel").unwrap();
        let s2 = s.edit_method("bass", "set", "lpf", "500").unwrap();
        assert!(s2.tracks[0].code.raw.contains(".lpf(500)"));
        assert!(!s2.tracks[1].code.raw.contains("lpf"));
    }

    #[test]
    fn bundled_songs_parse() {
        let dir = concat!(env!("CARGO_MANIFEST_DIR"), "/songs");
        let mut found = 0usize;
        for ent in std::fs::read_dir(dir).expect("songs/") {
            let ent = ent.unwrap();
            let path = ent.path();
            if path.extension().and_then(|s| s.to_str()) != Some("strudel") {
                continue;
            }
            found += 1;
            let text = std::fs::read_to_string(&path).unwrap();
            let s = parse_song(&text, path.to_str().unwrap()).unwrap_or_else(|e| {
                panic!("parse {}: {e}", path.display());
            });
            assert!(!s.tracks.is_empty(), "{}", path.display());
            assert!(s.bpm.unwrap_or(0.0) > 0.0, "{}", path.display());
            // Bundled demos use Strudel `// @title …` (not path fallback).
            assert!(
                !s.title.ends_with(".strudel"),
                "expected @title on {}: got {}",
                path.display(),
                s.title
            );
        }
        assert!(found >= 5, "expected demo songs, found {found}");
    }

    #[test]
    fn song_path_candidates_bare_name_prefers_user_then_songs_strudel() {
        let c = song_path_candidates("smoke").unwrap();
        let songs_strudel = PathBuf::from("songs").join("smoke.strudel");
        assert!(
            c.iter().any(|p| p == &songs_strudel),
            "expected songs/smoke.strudel in {c:?}"
        );
        if let Ok(ud) = user_songs_dir() {
            assert_eq!(c[0], ud.join("smoke.strudel"), "{c:?}");
            assert_eq!(c[1], ud.join("smoke.txt"), "{c:?}");
            assert_eq!(c[2], songs_strudel, "{c:?}");
        } else {
            assert_eq!(c[0], songs_strudel, "{c:?}");
        }
        assert!(c.iter().any(|p| p == &PathBuf::from("smoke.strudel")));
    }

    #[test]
    fn song_path_candidates_strudel_before_txt_with_dir() {
        let c = song_path_candidates("demos/pad").unwrap();
        assert_eq!(c[0], PathBuf::from("demos/pad.strudel"));
        assert_eq!(c[1], PathBuf::from("demos/pad.txt"));
        // Basename also tried under user library / songs/ (model often passes songs/name).
        assert!(
            c.iter()
                .any(|p| p == &PathBuf::from("songs").join("pad.strudel")),
            "{c:?}"
        );
        if let Ok(ud) = user_songs_dir() {
            assert!(
                c.iter().any(|p| p == &ud.join("pad.strudel")),
                "user lib basename fallback missing in {c:?}"
            );
        }
    }

    #[test]
    fn song_path_candidates_songs_prefix_falls_back_to_user_lib_basename() {
        let c = song_path_candidates("songs/house-track.strudel").unwrap();
        assert_eq!(c[0], PathBuf::from("songs/house-track.strudel"));
        if let Ok(ud) = user_songs_dir() {
            assert!(
                c.iter().any(|p| p == &ud.join("house-track.strudel")),
                "expected user lib fallback in {c:?}"
            );
        }
    }

    #[test]
    fn song_path_candidates_bare_with_ext_tries_user_then_songs() {
        let c = song_path_candidates("smoke.strudel").unwrap();
        let songs = PathBuf::from("songs").join("smoke.strudel");
        if let Ok(ud) = user_songs_dir() {
            assert_eq!(c[0], ud.join("smoke.strudel"));
            assert_eq!(c[1], songs);
        } else {
            assert_eq!(c[0], songs);
        }
        assert!(c.iter().any(|p| p == &PathBuf::from("smoke.strudel")));
    }

    #[test]
    fn sanitize_user_song_name_ok_and_rejects() {
        assert_eq!(
            sanitize_user_song_name("visitor-dark").unwrap(),
            "visitor-dark.strudel"
        );
        assert_eq!(
            sanitize_user_song_name("foo.strudel").unwrap(),
            "foo.strudel"
        );
        assert_eq!(sanitize_user_song_name("foo.txt").unwrap(), "foo.txt");
        assert!(sanitize_user_song_name("../x").is_err());
        assert!(sanitize_user_song_name("a/b").is_err());
        assert!(sanitize_user_song_name("a\\b").is_err());
        assert!(sanitize_user_song_name("C:foo").is_err());
        assert!(sanitize_user_song_name("bad name").is_err());
        assert!(sanitize_user_song_name("x.rs").is_err());
    }

    #[test]
    fn resolve_user_song_save_path_stays_in_library() {
        let p = resolve_user_song_save_path("my-track").unwrap();
        let dir = user_songs_dir().unwrap();
        assert_eq!(p, dir.join("my-track.strudel"));
        assert_eq!(p.parent(), Some(dir.as_path()));
        assert!(resolve_user_song_save_path("../escape").is_err());
        assert!(resolve_user_song_save_path("sub/dir").is_err());
    }

    #[test]
    fn resolve_rejects_parent_dir() {
        assert!(resolve_song_path("../secret.strudel").is_err());
        assert!(song_path_candidates("songs/../../x").is_err());
    }

    #[test]
    fn resolve_bundled_smoke_by_bare_name() {
        // Run from crate root in `cargo test`.
        let p = resolve_song_path("smoke").expect("songs/smoke.strudel");
        assert!(p.ends_with("smoke.strudel"), "{}", p.display());
        assert!(p.is_file());
    }

    #[test]
    fn resolve_prefers_strudel_when_both_exist() {
        let dir = std::env::temp_dir().join("strudel_resolve_both");
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(dir.join("songs")).unwrap();
        std::fs::write(
            dir.join("songs").join("dup.strudel"),
            "// @title d\nsetcpm(30)\n$: s(\"bd\")\n",
        )
        .unwrap();
        std::fs::write(dir.join("songs").join("dup.txt"), "should not win").unwrap();
        let prev = std::env::current_dir().unwrap();
        std::env::set_current_dir(&dir).unwrap();
        let r = resolve_song_path("dup");
        std::env::set_current_dir(prev).unwrap();
        let p = r.expect("dup");
        assert!(
            p.extension().and_then(|e| e.to_str()) == Some("strudel"),
            "{}",
            p.display()
        );
        let _ = std::fs::remove_dir_all(&dir);
    }
}

//! Mini-notation tokenizer, AST parser, and cycle evaluator.
//! Tokens and atoms carry byte spans in the mini-notation source (Strudel withLoc-style).

/// Byte range `[start, end)` within a mini-notation string (or absolute file after offset).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Span {
    pub start: usize,
    pub end: usize,
}

impl Span {
    pub fn new(start: usize, end: usize) -> Self {
        Self { start, end }
    }

    pub fn offset(self, base: usize) -> Self {
        Self {
            start: base + self.start,
            end: base + self.end,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    Word(String, Span),
    Rest(Span), // ~
    OpenBracket(Span),
    CloseBracket(Span), // [ ]
    OpenAngle(Span),
    CloseAngle(Span), // < >
    Star(Span),
    Slash(Span), // * /
    /// `@` — elongate (temporal weight)
    At(Span),
    /// `,` — parallel (simultaneous) separator, not whitespace
    Comma(Span),
    Number(f64, Span),
}

impl Token {
    pub fn span(&self) -> Span {
        match self {
            Token::Word(_, s)
            | Token::Rest(s)
            | Token::OpenBracket(s)
            | Token::CloseBracket(s)
            | Token::OpenAngle(s)
            | Token::CloseAngle(s)
            | Token::Star(s)
            | Token::Slash(s)
            | Token::At(s)
            | Token::Comma(s)
            | Token::Number(_, s) => *s,
        }
    }
}

pub fn tokenize(input: &str) -> Result<Vec<Token>, String> {
    let mut out = Vec::new();
    let bytes = input.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        let c = bytes[i] as char;
        match c {
            ' ' | '\t' | '\n' => {
                i += 1;
            }
            ',' => {
                out.push(Token::Comma(Span::new(i, i + 1)));
                i += 1;
            }
            '~' => {
                out.push(Token::Rest(Span::new(i, i + 1)));
                i += 1;
            }
            '[' => {
                out.push(Token::OpenBracket(Span::new(i, i + 1)));
                i += 1;
            }
            ']' => {
                out.push(Token::CloseBracket(Span::new(i, i + 1)));
                i += 1;
            }
            '<' => {
                out.push(Token::OpenAngle(Span::new(i, i + 1)));
                i += 1;
            }
            '>' => {
                out.push(Token::CloseAngle(Span::new(i, i + 1)));
                i += 1;
            }
            '*' => {
                out.push(Token::Star(Span::new(i, i + 1)));
                i += 1;
            }
            '/' => {
                out.push(Token::Slash(Span::new(i, i + 1)));
                i += 1;
            }
            '@' => {
                out.push(Token::At(Span::new(i, i + 1)));
                i += 1;
            }
            c if is_atom_char(c) => {
                let start = i;
                i += 1;
                while i < bytes.len() {
                    let c = bytes[i] as char;
                    if is_atom_char(c) {
                        i += 1;
                    } else {
                        break;
                    }
                }
                let span = Span::new(start, i);
                let w = &input[start..i];
                if let Ok(n) = w.parse::<f64>() {
                    out.push(Token::Number(n, span));
                } else {
                    out.push(Token::Word(w.to_string(), span));
                }
            }
            other => return Err(format!("unexpected char: {other}")),
        }
    }
    Ok(out)
}

/// Mini sound/note atom characters. `:` is `part:slug` (sample variant), not a token break.
fn is_atom_char(c: char) -> bool {
    c.is_ascii_alphanumeric() || matches!(c, '.' | '#' | '-' | '_' | '\'' | ':')
}

#[derive(Debug, Clone, PartialEq)]
pub enum Node {
    Atom {
        value: String,
        span: Span,
    },
    Rest,
    /// `[a b]` / top-level: equal divisions of one cycle (space-separated)
    Seq(Vec<Node>),
    /// `<a b>` / `cat(...)`: pick one item per cycle (not simultaneous)
    Stack(Vec<Node>),
    /// `a, b` / `[bd,sd]`: simultaneous layers over the same time span
    Parallel(Vec<Node>),
    /// `node * n`
    Fast(Box<Node>, f64),
    /// `node / n`
    Slow(Box<Node>, f64),
    /// `node@n`: temporal weight inside a Seq (default weight is 1)
    Elongate(Box<Node>, f64),
}

impl Node {
    /// Convenience for tests and simple construction.
    pub fn atom(value: impl Into<String>, span: Span) -> Self {
        Node::Atom {
            value: value.into(),
            span,
        }
    }
}

pub fn parse(input: &str) -> Result<Node, String> {
    let toks = tokenize(input)?;
    if toks.is_empty() {
        return Ok(Node::Rest);
    }
    let mut pos = 0;
    let node = parse_seq(&toks, &mut pos, None)?;
    if pos != toks.len() {
        return Err(format!("trailing tokens at {pos}"));
    }
    Ok(node)
}

fn parse_seq(
    t: &[Token],
    pos: &mut usize,
    closing: Option<fn(&Token) -> bool>,
) -> Result<Node, String> {
    // Space-separated items form a Seq branch; commas split Parallel branches.
    let mut branches: Vec<Vec<Node>> = vec![Vec::new()];
    while *pos < t.len() {
        if closing.as_ref().is_some_and(|pred| pred(&t[*pos])) {
            *pos += 1;
            break;
        }
        if matches!(t[*pos], Token::Comma(_)) {
            *pos += 1;
            branches.push(Vec::new());
            continue;
        }
        let mut item = parse_item(t, pos)?;
        // Postfix in source order: * / @
        loop {
            if *pos + 1 < t.len() {
                if matches!(t[*pos], Token::Star(_)) {
                    if let Token::Number(n, _) = t[*pos + 1] {
                        item = Node::Fast(Box::new(item), n);
                        *pos += 2;
                        continue;
                    }
                }
                if matches!(t[*pos], Token::Slash(_)) {
                    if let Token::Number(n, _) = t[*pos + 1] {
                        item = Node::Slow(Box::new(item), n);
                        *pos += 2;
                        continue;
                    }
                }
                if matches!(t[*pos], Token::At(_)) {
                    if let Token::Number(n, _) = t[*pos + 1] {
                        if n <= 0.0 {
                            return Err(format!("elongate weight must be > 0, got {n}"));
                        }
                        item = Node::Elongate(Box::new(item), n);
                        *pos += 2;
                        continue;
                    }
                    return Err("elongate @ expects a number".into());
                }
            }
            break;
        }
        branches
            .last_mut()
            .expect("at least one parallel branch")
            .push(item);
    }
    Ok(finish_parallel_branches(branches))
}

/// Collapse comma-separated branches into `Seq` (one branch) or `Parallel`.
fn finish_parallel_branches(mut branches: Vec<Vec<Node>>) -> Node {
    // Drop empty branches from leading/trailing/double commas.
    branches.retain(|b| !b.is_empty());
    if branches.is_empty() {
        return Node::Seq(vec![]);
    }
    if branches.len() == 1 {
        return Node::Seq(branches.pop().unwrap());
    }
    let parts: Vec<Node> = branches
        .into_iter()
        .map(|items| {
            if items.len() == 1 {
                items.into_iter().next().unwrap()
            } else {
                Node::Seq(items)
            }
        })
        .collect();
    Node::Parallel(parts)
}

fn is_close_bracket(t: &Token) -> bool {
    matches!(t, Token::CloseBracket(_))
}

fn is_close_angle(t: &Token) -> bool {
    matches!(t, Token::CloseAngle(_))
}

fn parse_item(t: &[Token], pos: &mut usize) -> Result<Node, String> {
    match t.get(*pos) {
        Some(Token::Word(w, span)) => {
            *pos += 1;
            Ok(Node::Atom {
                value: w.clone(),
                span: *span,
            })
        }
        Some(Token::Rest(_)) => {
            *pos += 1;
            Ok(Node::Rest)
        }
        Some(Token::Number(n, span)) => {
            *pos += 1;
            Ok(Node::Atom {
                value: n.to_string(),
                span: *span,
            })
        }
        Some(Token::OpenBracket(_)) => {
            *pos += 1;
            parse_seq(t, pos, Some(is_close_bracket))
        }
        Some(Token::OpenAngle(_)) => {
            *pos += 1;
            let s = parse_seq(t, pos, Some(is_close_angle))?;
            // Angle brackets are cycle-alternate (Stack). Flatten Parallel rare case
            // so `<a, b>` still yields two cycle choices rather than simultaneous.
            match s {
                Node::Seq(v) => Ok(Node::Stack(v)),
                Node::Parallel(v) => Ok(Node::Stack(v)),
                other => Ok(Node::Stack(vec![other])),
            }
        }
        Some(Token::Comma(_)) => Err("unexpected comma".into()),
        other => Err(format!("unexpected token: {other:?}")),
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Event {
    /// Position within the cycle (= bar) in [0, 1)
    pub start: f64,
    /// Duration in bar units
    pub dur: f64,
    pub value: String,
    /// Byte span of the atom inside the mini-notation string (if known).
    pub span: Option<Span>,
}

/// Shift all atom spans by `delta` bytes (for multi-string `cat` etc.).
pub fn offset_spans(node: &mut Node, delta: usize) {
    match node {
        Node::Atom { span, .. } => {
            *span = span.offset(delta);
        }
        Node::Rest => {}
        Node::Seq(items) | Node::Stack(items) | Node::Parallel(items) => {
            for it in items {
                offset_spans(it, delta);
            }
        }
        Node::Fast(inner, _) | Node::Slow(inner, _) | Node::Elongate(inner, _) => {
            offset_spans(inner, delta)
        }
    }
}

/// Temporal weight of a seq child (Elongate); default 1.
fn seq_weight(node: &Node) -> f64 {
    match node {
        Node::Elongate(_, w) => *w,
        _ => 1.0,
    }
}

fn unwrap_elongate(node: &Node) -> &Node {
    match node {
        Node::Elongate(inner, _) => inner,
        other => other,
    }
}

/// Evaluate AST for one cycle into timed events. One cycle = one bar.
pub fn events(node: &Node, cycle: u64) -> Vec<Event> {
    let mut out = Vec::new();
    emit(node, 0.0, 1.0, cycle, &mut out);
    out
}

fn emit(node: &Node, start: f64, span: f64, cycle: u64, out: &mut Vec<Event>) {
    match node {
        Node::Atom { value, span: src } => out.push(Event {
            start,
            dur: span,
            value: value.clone(),
            span: Some(*src),
        }),
        Node::Rest => {}
        Node::Seq(items) => {
            if items.is_empty() {
                return;
            }
            let total: f64 = items.iter().map(seq_weight).sum();
            if total <= 0.0 {
                return;
            }
            let mut t = start;
            for it in items {
                let w = seq_weight(it);
                let each = span * (w / total);
                emit(unwrap_elongate(it), t, each, cycle, out);
                t += each;
            }
        }
        Node::Stack(items) => {
            if items.is_empty() {
                return;
            }
            let sel = (cycle as usize) % items.len();
            emit(&items[sel], start, span, cycle, out);
        }
        Node::Parallel(items) => {
            for it in items {
                emit(it, start, span, cycle, out);
            }
        }
        Node::Fast(inner, n) => {
            let n_u = (*n).max(1.0) as usize;
            let sub = span / n_u as f64;
            for k in 0..n_u {
                emit(inner, start + k as f64 * sub, sub, cycle, out);
            }
        }
        Node::Slow(inner, n) => {
            let n_u = (*n).max(1.0) as u64;
            if cycle.is_multiple_of(n_u) {
                emit(inner, start, span, cycle, out);
            }
        }
        // Weight only matters as a Seq sibling; alone, pass through.
        Node::Elongate(inner, _) => emit(inner, start, span, cycle, out),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tokenizes_basic() {
        let t = tokenize("c3 [e3 g3]*2 ~ <a3 b3> bd_cp").unwrap();
        assert_eq!(t.len(), 13);
        assert!(matches!(&t[0], Token::Word(w, s) if w == "c3" && s.start == 0 && s.end == 2));
        assert!(matches!(&t[1], Token::OpenBracket(_)));
        assert!(matches!(&t[2], Token::Word(w, _) if w == "e3"));
        assert!(matches!(&t[12], Token::Word(w, _) if w == "bd_cp"));
    }

    #[test]
    fn tokenizes_part_slug() {
        let t = tokenize("bd:8b*4, [~ sd:8s]*2").unwrap();
        assert!(matches!(&t[0], Token::Word(w, _) if w == "bd:8b"));
        assert!(matches!(&t[1], Token::Star(_)));
        let n = parse("bd:hf hh:cl").unwrap();
        let ev = events(&n, 0);
        assert_eq!(ev.len(), 2);
        assert_eq!(ev[0].value, "bd:hf");
        assert_eq!(ev[1].value, "hh:cl");
    }

    #[test]
    fn parses_nested() {
        let n = parse("c3 [e3 g3]*2 ~").unwrap();
        assert!(matches!(n, Node::Seq(ref v) if v.len() == 3));
        let n = parse("c3/2").unwrap();
        assert!(matches!(n, Node::Seq(ref v) if matches!(v[0], Node::Slow(_, 2.0))));
        let n = parse("<c3 e3>").unwrap();
        assert!(matches!(n, Node::Seq(ref v) if matches!(v.first(), Some(Node::Stack(_)))));
    }

    #[test]
    fn evaluates_events() {
        let n = parse("c3 e3").unwrap();
        let ev = events(&n, 0);
        assert_eq!(ev.len(), 2);
        assert!((ev[1].start - 0.5).abs() < 1e-9);
        assert_eq!(ev[0].value, "c3");
        assert_eq!(ev[1].value, "e3");

        let n = parse("<c3 e3>").unwrap();
        assert_eq!(events(&n, 0)[0].value, "c3");
        assert_eq!(events(&n, 1)[0].value, "e3");

        let n = parse("c3*4 ~").unwrap();
        assert_eq!(events(&n, 0).len(), 4);
    }

    #[test]
    fn tokenizes_chord_word() {
        let t = tokenize("c3'maj").unwrap();
        assert!(matches!(&t[0], Token::Word(w, s) if w == "c3'maj" && s.start == 0 && s.end == 6));
    }

    #[test]
    fn atom_spans_match_source() {
        let src = "bd hh";
        let n = parse(src).unwrap();
        let ev = events(&n, 0);
        assert_eq!(ev.len(), 2);
        let s0 = ev[0].span.unwrap();
        let s1 = ev[1].span.unwrap();
        assert_eq!(&src[s0.start..s0.end], "bd");
        assert_eq!(&src[s1.start..s1.end], "hh");
    }

    #[test]
    fn fast_repeats_same_span() {
        let src = "bd*4";
        let n = parse(src).unwrap();
        let ev = events(&n, 0);
        assert_eq!(ev.len(), 4);
        for e in &ev {
            let s = e.span.unwrap();
            assert_eq!(&src[s.start..s.end], "bd");
        }
    }

    #[test]
    fn stack_span_switches_per_cycle() {
        let src = "<hh oh>";
        let n = parse(src).unwrap();
        let e0 = &events(&n, 0)[0];
        let e1 = &events(&n, 1)[0];
        assert_eq!(e0.value, "hh");
        assert_eq!(e1.value, "oh");
        assert_eq!(&src[e0.span.unwrap().start..e0.span.unwrap().end], "hh");
        assert_eq!(&src[e1.span.unwrap().start..e1.span.unwrap().end], "oh");
    }

    #[test]
    fn tokenizes_comma() {
        let t = tokenize("bd,sd").unwrap();
        assert_eq!(t.len(), 3);
        assert!(matches!(&t[0], Token::Word(w, _) if w == "bd"));
        assert!(matches!(&t[1], Token::Comma(_)));
        assert!(matches!(&t[2], Token::Word(w, _) if w == "sd"));
    }

    #[test]
    fn parallel_comma_simultaneous() {
        let n = parse("bd,sd").unwrap();
        assert!(matches!(n, Node::Parallel(ref v) if v.len() == 2));
        let ev = events(&n, 0);
        assert_eq!(ev.len(), 2);
        assert!((ev[0].start - 0.0).abs() < 1e-9);
        assert!((ev[1].start - 0.0).abs() < 1e-9);
        assert!((ev[0].dur - 1.0).abs() < 1e-9);
        let mut vals: Vec<_> = ev.iter().map(|e| e.value.as_str()).collect();
        vals.sort();
        assert_eq!(vals, ["bd", "sd"]);
    }

    #[test]
    fn nested_parallel_in_seq() {
        let n = parse("[bd hh [bd,sd] hh]").unwrap();
        // Top-level `[...]` is one item in an outer Seq; inner Seq has 4 slots.
        let slots = match &n {
            Node::Seq(outer) => match outer.as_slice() {
                [Node::Seq(inner)] => inner,
                other => panic!("expected Seq([Seq(...)]), got {other:?}"),
            },
            other => panic!("expected Seq, got {other:?}"),
        };
        assert_eq!(slots.len(), 4);
        assert!(matches!(&slots[2], Node::Parallel(v) if v.len() == 2));

        let ev = events(&n, 0);
        // bd, hh, bd+sd, hh → 5 events
        assert_eq!(ev.len(), 5);
        let at_half: Vec<_> = ev
            .iter()
            .filter(|e| (e.start - 0.5).abs() < 1e-9)
            .map(|e| e.value.as_str())
            .collect();
        assert!(at_half.contains(&"bd"));
        assert!(at_half.contains(&"sd"));
    }

    #[test]
    fn parallel_fast_layers() {
        let n = parse("bd*4, hh*8").unwrap();
        assert!(matches!(n, Node::Parallel(ref v) if v.len() == 2));
        let ev = events(&n, 0);
        let n_bd = ev.iter().filter(|e| e.value == "bd").count();
        let n_hh = ev.iter().filter(|e| e.value == "hh").count();
        assert_eq!(n_bd, 4);
        assert_eq!(n_hh, 8);
    }

    #[test]
    fn grid_pattern_star_two() {
        let n = parse("[bd hh [bd,sd] hh]*2").unwrap();
        let ev = events(&n, 0);
        // 5 events per half-cycle × 2
        assert_eq!(ev.len(), 10);
        let bds = ev.iter().filter(|e| e.value == "bd").count();
        let hhs = ev.iter().filter(|e| e.value == "hh").count();
        let sds = ev.iter().filter(|e| e.value == "sd").count();
        assert_eq!(bds, 4); // 2 halves × (slot1 + slot3)
        assert_eq!(hhs, 4);
        assert_eq!(sds, 2);
    }

    #[test]
    fn nested_angle_brackets() {
        let n = parse("c2 <eb2 g2> <eb2 <f2 g2>>").unwrap();
        // cycle 0: first of each stack
        let e0 = events(&n, 0);
        let vals0: Vec<_> = e0.iter().map(|e| e.value.as_str()).collect();
        assert_eq!(vals0, ["c2", "eb2", "eb2"]);
        // cycle 1: alternate stack choices
        let e1 = events(&n, 1);
        let vals1: Vec<_> = e1.iter().map(|e| e.value.as_str()).collect();
        // second slot → g2; third → nested stack with cycle 1 → g2
        assert_eq!(vals1[0], "c2");
        assert_eq!(vals1[1], "g2");
        assert_eq!(vals1[2], "g2");
    }

    #[test]
    fn elongate_weights_seq() {
        let n = parse("a@2 b").unwrap();
        let ev = events(&n, 0);
        assert_eq!(ev.len(), 2);
        assert!((ev[0].dur - 2.0 / 3.0).abs() < 1e-9);
        assert!((ev[1].dur - 1.0 / 3.0).abs() < 1e-9);
        assert!((ev[1].start - 2.0 / 3.0).abs() < 1e-9);
    }

    #[test]
    fn elongate_rest_and_hits() {
        // weights 3 + 1 + 4 = 8
        let n = parse("~@3 bd ~@4").unwrap();
        let ev = events(&n, 0);
        assert_eq!(ev.len(), 1);
        assert_eq!(ev[0].value, "bd");
        assert!((ev[0].start - 3.0 / 8.0).abs() < 1e-9);
        assert!((ev[0].dur - 1.0 / 8.0).abs() < 1e-9);
    }

    #[test]
    fn elongate_zero_errors() {
        assert!(parse("bd@0").is_err());
        assert!(parse("bd@-1").is_err());
    }

    #[test]
    fn techno_four_on_the_floor_kick_hat_times() {
        // Skill docs/skills/strudel-genre-four-on-the-floor (techno): kick in front, no snare.
        let n = parse("bd*4, [~ hh]*4").unwrap();
        let ev = events(&n, 0);
        let starts = |name: &str| -> Vec<f64> {
            ev.iter()
                .filter(|e| e.value == name)
                .map(|e| e.start)
                .collect()
        };
        let bd = starts("bd");
        let hh = starts("hh");
        assert_eq!(starts("sd").len(), 0);
        assert_eq!(bd.len(), 4);
        assert_eq!(hh.len(), 4);
        for (got, want) in bd.iter().zip([0.0, 0.25, 0.5, 0.75]) {
            assert!((got - want).abs() < 1e-9, "bd start {got} != {want}");
        }
        for (got, want) in hh.iter().zip([0.125, 0.375, 0.625, 0.875]) {
            assert!((got - want).abs() < 1e-9, "hh start {got} != {want}");
        }
    }

    #[test]
    fn four_on_the_floor_event_times() {
        // House backbeat grid (not the techno skill default).
        let n = parse("bd*4, [~ sd]*2, [~ hh]*4").unwrap();
        let ev = events(&n, 0);
        let starts = |name: &str| -> Vec<f64> {
            ev.iter()
                .filter(|e| e.value == name)
                .map(|e| e.start)
                .collect()
        };
        let bd = starts("bd");
        let sd = starts("sd");
        let hh = starts("hh");
        assert_eq!(bd.len(), 4);
        assert_eq!(sd.len(), 2);
        assert_eq!(hh.len(), 4);
        for (got, want) in bd.iter().zip([0.0, 0.25, 0.5, 0.75]) {
            assert!((got - want).abs() < 1e-9, "bd start {got} != {want}");
        }
        for (got, want) in sd.iter().zip([0.25, 0.75]) {
            assert!((got - want).abs() < 1e-9, "sd start {got} != {want}");
        }
        for (got, want) in hh.iter().zip([0.125, 0.375, 0.625, 0.875]) {
            assert!((got - want).abs() < 1e-9, "hh start {got} != {want}");
        }
    }

    #[test]
    fn smart_drum_pattern_parses() {
        let src = "bd*4, [~ sd]*2, [~ hh]*4, <~ [~@3 bd ~@4]>";
        let n = parse(src).unwrap();
        assert!(matches!(n, Node::Parallel(ref v) if v.len() == 4));
        let ev0 = events(&n, 0);
        let n_bd = ev0.iter().filter(|e| e.value == "bd").count();
        let n_sd = ev0.iter().filter(|e| e.value == "sd").count();
        let n_hh = ev0.iter().filter(|e| e.value == "hh").count();
        // cycle 0 of stack is ~ (no extra bd); bd*4 → 4 kicks
        assert_eq!(n_bd, 4);
        assert_eq!(n_sd, 2);
        assert_eq!(n_hh, 4);
        // cycle 1: stack picks [~@3 bd ~@4] → one more bd
        let ev1 = events(&n, 1);
        let n_bd1 = ev1.iter().filter(|e| e.value == "bd").count();
        assert_eq!(n_bd1, 5);
    }
}

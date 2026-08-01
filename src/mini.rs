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
            ' ' | '\t' | '\n' | ',' => {
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
            c if c.is_ascii_alphanumeric() || matches!(c, '.' | '#' | '-' | '_' | '\'') => {
                let start = i;
                i += 1;
                while i < bytes.len() {
                    let c = bytes[i] as char;
                    if c.is_ascii_alphanumeric() || matches!(c, '.' | '#' | '-' | '_' | '\'') {
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

#[derive(Debug, Clone, PartialEq)]
pub enum Node {
    Atom {
        value: String,
        span: Span,
    },
    Rest,
    /// `[a b]` / top-level: equal divisions of one cycle
    Seq(Vec<Node>),
    /// `<a b>`: pick one item per cycle
    Stack(Vec<Node>),
    /// `node * n`
    Fast(Box<Node>, f64),
    /// `node / n`
    Slow(Box<Node>, f64),
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
    let mut items = Vec::new();
    while *pos < t.len() {
        if closing.as_ref().is_some_and(|pred| pred(&t[*pos])) {
            *pos += 1;
            break;
        }
        let mut item = parse_item(t, pos)?;
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
            }
            break;
        }
        items.push(item);
    }
    Ok(Node::Seq(items))
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
            if let Node::Seq(v) = s {
                Ok(Node::Stack(v))
            } else {
                unreachable!()
            }
        }
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
        Node::Seq(items) | Node::Stack(items) => {
            for it in items {
                offset_spans(it, delta);
            }
        }
        Node::Fast(inner, _) | Node::Slow(inner, _) => offset_spans(inner, delta),
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
            let each = span / items.len() as f64;
            for (i, it) in items.iter().enumerate() {
                emit(it, start + i as f64 * each, each, cycle, out);
            }
        }
        Node::Stack(items) => {
            if items.is_empty() {
                return;
            }
            let sel = (cycle as usize) % items.len();
            emit(&items[sel], start, span, cycle, out);
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
}

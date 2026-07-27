//! Mini-notation tokenizer, AST parser, and cycle evaluator.

#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    Word(String),
    Rest, // ~
    OpenBracket,
    CloseBracket, // [ ]
    OpenAngle,
    CloseAngle, // < >
    Star,
    Slash, // * /
    Number(f64),
}

pub fn tokenize(input: &str) -> Result<Vec<Token>, String> {
    let mut out = Vec::new();
    let mut chars = input.chars().peekable();
    while let Some(&c) = chars.peek() {
        match c {
            ' ' | '\t' | '\n' | ',' => {
                chars.next();
            }
            '~' => {
                out.push(Token::Rest);
                chars.next();
            }
            '[' => {
                out.push(Token::OpenBracket);
                chars.next();
            }
            ']' => {
                out.push(Token::CloseBracket);
                chars.next();
            }
            '<' => {
                out.push(Token::OpenAngle);
                chars.next();
            }
            '>' => {
                out.push(Token::CloseAngle);
                chars.next();
            }
            '*' => {
                out.push(Token::Star);
                chars.next();
            }
            '/' => {
                out.push(Token::Slash);
                chars.next();
            }
            c if c.is_ascii_alphanumeric() || matches!(c, '.' | '#' | '-' | '_' | '\'') => {
                let mut w = String::new();
                while let Some(&c) = chars.peek() {
                    if c.is_ascii_alphanumeric() || matches!(c, '.' | '#' | '-' | '_' | '\'') {
                        w.push(c);
                        chars.next();
                    } else {
                        break;
                    }
                }
                if let Ok(n) = w.parse::<f64>() {
                    out.push(Token::Number(n));
                } else {
                    out.push(Token::Word(w));
                }
            }
            other => return Err(format!("unexpected char: {other}")),
        }
    }
    Ok(out)
}

#[derive(Debug, Clone, PartialEq)]
pub enum Node {
    Atom(String),
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

fn parse_seq(t: &[Token], pos: &mut usize, closing: Option<Token>) -> Result<Node, String> {
    let mut items = Vec::new();
    while *pos < t.len() {
        if closing.as_ref().is_some_and(|c| &t[*pos] == c) {
            *pos += 1;
            break;
        }
        let mut item = parse_item(t, pos)?;
        loop {
            if *pos + 1 < t.len() && matches!(t[*pos], Token::Star) {
                if let Token::Number(n) = t[*pos + 1] {
                    item = Node::Fast(Box::new(item), n);
                    *pos += 2;
                    continue;
                }
            }
            if *pos + 1 < t.len() && matches!(t[*pos], Token::Slash) {
                if let Token::Number(n) = t[*pos + 1] {
                    item = Node::Slow(Box::new(item), n);
                    *pos += 2;
                    continue;
                }
            }
            break;
        }
        items.push(item);
    }
    Ok(Node::Seq(items))
}

fn parse_item(t: &[Token], pos: &mut usize) -> Result<Node, String> {
    match t.get(*pos) {
        Some(Token::Word(w)) => {
            *pos += 1;
            Ok(Node::Atom(w.clone()))
        }
        Some(Token::Rest) => {
            *pos += 1;
            Ok(Node::Rest)
        }
        Some(Token::Number(n)) => {
            *pos += 1;
            Ok(Node::Atom(n.to_string()))
        }
        Some(Token::OpenBracket) => {
            *pos += 1;
            parse_seq(t, pos, Some(Token::CloseBracket))
        }
        Some(Token::OpenAngle) => {
            *pos += 1;
            let s = parse_seq(t, pos, Some(Token::CloseAngle))?;
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
}

/// Evaluate AST for one cycle into timed events. One cycle = one bar.
pub fn events(node: &Node, cycle: u64) -> Vec<Event> {
    let mut out = Vec::new();
    emit(node, 0.0, 1.0, cycle, &mut out);
    out
}

fn emit(node: &Node, start: f64, span: f64, cycle: u64, out: &mut Vec<Event>) {
    match node {
        Node::Atom(v) => out.push(Event {
            start,
            dur: span,
            value: v.clone(),
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
            if cycle % n_u == 0 {
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
        assert_eq!(
            t,
            vec![
                Token::Word("c3".into()),
                Token::OpenBracket,
                Token::Word("e3".into()),
                Token::Word("g3".into()),
                Token::CloseBracket,
                Token::Star,
                Token::Number(2.0),
                Token::Rest,
                Token::OpenAngle,
                Token::Word("a3".into()),
                Token::Word("b3".into()),
                Token::CloseAngle,
                Token::Word("bd_cp".into()),
            ]
        );
    }

    #[test]
    fn parses_nested() {
        let n = parse("c3 [e3 g3]*2 ~").unwrap();
        assert!(matches!(n, Node::Seq(ref v) if v.len() == 3));
        let n = parse("c3/2").unwrap();
        assert!(matches!(n, Node::Seq(ref v) if matches!(v[0], Node::Slow(_, 2.0))));
        // Top-level parse always wraps in Seq; angle groups become Stack inside.
        let n = parse("<c3 e3>").unwrap();
        assert!(matches!(n, Node::Seq(ref v) if matches!(v.first(), Some(Node::Stack(_)))));
    }

    #[test]
    fn evaluates_events() {
        let n = parse("c3 e3").unwrap();
        let ev = events(&n, 0);
        assert_eq!(ev.len(), 2);
        assert!((ev[1].start - 0.5).abs() < 1e-9);

        let n = parse("<c3 e3>").unwrap();
        assert_eq!(events(&n, 0)[0].value, "c3");
        assert_eq!(events(&n, 1)[0].value, "e3");

        let n = parse("c3*4 ~").unwrap();
        assert_eq!(events(&n, 0).len(), 4);
    }

    #[test]
    fn tokenizes_chord_word() {
        let t = tokenize("c3'maj").unwrap();
        assert_eq!(t, vec![Token::Word("c3'maj".into())]);
    }
}

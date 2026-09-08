//! Live session kind: two-deck DJ mix vs one-song play.

/// Which live session the CLI / API / MCP / Hermes envelope should assume.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum SessionKind {
    /// Dual-deck mix (`dj-hermes dj`, `play --repl`).
    #[default]
    Dj,
    /// Single song on deck A (`dj-hermes play`).
    Play,
}

impl SessionKind {
    /// Play uses only deck A; DJ shows A and B.
    pub fn single_deck(self) -> bool {
        matches!(self, Self::Play)
    }
}

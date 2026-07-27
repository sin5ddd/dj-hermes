//! Interactive text REPL (no highlight) for deck / mixer commands.

use std::sync::{Arc, Mutex};

use crossbeam::channel::Sender;
use rustyline::error::ReadlineError;
use rustyline::DefaultEditor;

use crate::cmd;
use crate::engine::{Command, Engine};
use crate::watcher::DeckPaths;

/// Run the REPL on the current thread until quit.
pub fn run(tx: Sender<Command>, deck_paths: DeckPaths, engine: Option<Arc<Mutex<Engine>>>) {
    let mut rl = match DefaultEditor::new() {
        Ok(e) => e,
        Err(e) => {
            eprintln!("repl: failed to init rustyline: {e}");
            return;
        }
    };
    print!("{}", cmd::HELP);
    loop {
        match rl.readline("» ") {
            Ok(line) => {
                let line = line.trim();
                if line.is_empty() {
                    continue;
                }
                let _ = rl.add_history_entry(line);
                let result = cmd::exec(line, &tx, &deck_paths, engine.as_ref());
                for m in &result.messages {
                    // Multi-line help
                    for part in m.lines() {
                        println!("{part}");
                    }
                }
                if result.quit {
                    break;
                }
            }
            Err(ReadlineError::Interrupted) | Err(ReadlineError::Eof) => {
                let _ = tx.send(Command::Hush);
                break;
            }
            Err(e) => {
                eprintln!("repl error: {e}");
                break;
            }
        }
    }
}

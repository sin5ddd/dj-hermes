//! Idle-gated Hermes cron automix for `dj-hermes dj` live TUI.
//!
//! The worker talks to `hermes --profile … cron` off the UI thread. Jobs stay
//! paused until 5 minutes of no local activity (or `/automix on`).

use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::sync::{Arc, Mutex};
use std::thread::{self, JoinHandle};
use std::time::{Duration, Instant};

use crossbeam::channel::{unbounded, Receiver, RecvTimeoutError, Sender};

pub const JOB_NAME: &str = "dj-automix";
pub const IDLE: Duration = Duration::from_secs(300);
const CLI_TIMEOUT: Duration = Duration::from_secs(15);
const TICK: Duration = Duration::from_millis(200);
const GATEWAY_DOWN_MSG: &str =
    "automix: dj-hermes の cron ticker が止まっている（hermes --profile dj-hermes gateway、または default の gateway.multiplex_profiles: true）";

const PROMPT: &str = include_str!("../docs/profile/dj-hermes/cron/dj-automix.prompt.txt");

pub enum AutomixCmd {
    Touch,
    ForceOn,
    ForceOff,
    Shutdown,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AutomixMode {
    Disabled,
    IdleWait,
    Armed,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AutomixEvent {
    Paused,
    Resumed,
    Failed(String),
    GatewayDown,
}

pub trait CronCli: Send {
    fn list_all(&self) -> Result<String, String>;
    fn create_paused(&self, prompt: &str) -> Result<(), String>;
    fn pause(&self) -> Result<(), String>;
    fn resume(&self) -> Result<(), String>;
    fn status_gateway(&self) -> Result<String, String>;
}

pub struct AutomixHandle {
    cmd_tx: Sender<AutomixCmd>,
    mode: Arc<Mutex<AutomixMode>>,
    event_rx: Receiver<AutomixEvent>,
    worker: Mutex<Option<JoinHandle<()>>>,
    #[cfg(test)]
    last_touch: Arc<Mutex<Instant>>,
}

impl AutomixHandle {
    pub fn start(bin: PathBuf, profile: String) -> Self {
        Self::start_with_cli(Box::new(HermesCronCli { bin, profile }))
    }

    fn start_with_cli(cli: Box<dyn CronCli>) -> Self {
        let (cmd_tx, cmd_rx) = unbounded::<AutomixCmd>();
        let (event_tx, event_rx) = unbounded::<AutomixEvent>();
        let mode = Arc::new(Mutex::new(AutomixMode::IdleWait));
        let last_touch = Arc::new(Mutex::new(Instant::now()));
        let mode_w = Arc::clone(&mode);
        let last_w = Arc::clone(&last_touch);

        let worker = thread::Builder::new()
            .name("automix-cron".into())
            .spawn(move || worker_loop(cli, cmd_rx, event_tx, mode_w, last_w))
            .expect("spawn automix-cron");

        Self {
            cmd_tx,
            mode,
            event_rx,
            worker: Mutex::new(Some(worker)),
            #[cfg(test)]
            last_touch,
        }
    }

    pub fn touch(&self) {
        let _ = self.cmd_tx.send(AutomixCmd::Touch);
    }

    pub fn force_on(&self) {
        let _ = self.cmd_tx.send(AutomixCmd::ForceOn);
    }

    pub fn force_off(&self) {
        let _ = self.cmd_tx.send(AutomixCmd::ForceOff);
    }

    pub fn mode(&self) -> AutomixMode {
        *lock(&self.mode)
    }

    pub fn shutdown(&self) {
        let _ = self.cmd_tx.send(AutomixCmd::Shutdown);
        if let Some(h) = self.worker.lock().unwrap_or_else(|e| e.into_inner()).take() {
            let _ = h.join();
        }
    }

    pub fn drain_events(&self) -> Vec<AutomixEvent> {
        let mut out = Vec::new();
        while let Ok(e) = self.event_rx.try_recv() {
            out.push(e);
        }
        out
    }

    #[cfg(test)]
    fn rewind_last_touch(&self, ago: Duration) {
        if let Some(past) = Instant::now().checked_sub(ago) {
            *lock(&self.last_touch) = past;
        }
    }
}

impl Drop for AutomixHandle {
    fn drop(&mut self) {
        self.shutdown();
    }
}

struct HermesCronCli {
    bin: PathBuf,
    profile: String,
}

impl HermesCronCli {
    fn run(&self, args: &[String]) -> Result<String, String> {
        run_timed(&self.bin, args, CLI_TIMEOUT)
    }

    fn profile_args(&self, rest: &[&str]) -> Vec<String> {
        let mut args = Vec::with_capacity(2 + rest.len());
        args.push("--profile".into());
        args.push(self.profile.clone());
        args.extend(rest.iter().map(|s| (*s).to_string()));
        args
    }
}

impl CronCli for HermesCronCli {
    fn list_all(&self) -> Result<String, String> {
        self.run(&self.profile_args(&["cron", "list", "--all"]))
    }

    fn create_paused(&self, prompt: &str) -> Result<(), String> {
        self.run(&self.profile_args(&[
            "cron",
            "create",
            "every 1m",
            prompt,
            "--name",
            JOB_NAME,
            "--skill",
            "strudel-dj-mix",
            "--deliver",
            "local",
            "--failure-deliver",
            "local",
            "--paused",
            "--paused-reason",
            "idle gate",
            "--continuity",
            "--reasoning-effort",
            "none",
        ]))
        .map(|_| ())
    }

    fn pause(&self) -> Result<(), String> {
        self.run(&self.profile_args(&["cron", "pause", JOB_NAME]))
            .map(|_| ())
    }

    fn resume(&self) -> Result<(), String> {
        self.run(&self.profile_args(&["cron", "resume", JOB_NAME]))
            .map(|_| ())
    }

    fn status_gateway(&self) -> Result<String, String> {
        self.run(&self.profile_args(&["cron", "status"]))
    }
}

fn worker_loop(
    cli: Box<dyn CronCli>,
    cmd_rx: Receiver<AutomixCmd>,
    event_tx: Sender<AutomixEvent>,
    mode: Arc<Mutex<AutomixMode>>,
    last_touch: Arc<Mutex<Instant>>,
) {
    ensure_job(cli.as_ref(), &event_tx);
    loop {
        match cmd_rx.recv_timeout(TICK) {
            Ok(AutomixCmd::Shutdown) => {
                let _ = cli.pause();
                break;
            }
            Ok(cmd) => handle_cmd(cli.as_ref(), cmd, &event_tx, &mode, &last_touch),
            Err(RecvTimeoutError::Timeout) => tick(cli.as_ref(), &event_tx, &mode, &last_touch),
            Err(RecvTimeoutError::Disconnected) => break,
        }
    }
}

fn ensure_job(cli: &dyn CronCli, event_tx: &Sender<AutomixEvent>) {
    match cli.list_all() {
        Ok(listed) => {
            let n = listed.matches(JOB_NAME).count();
            if n >= 2 {
                emit(
                    event_tx,
                    AutomixEvent::Failed(format!(
                        "duplicate {JOB_NAME}; hermes --profile dj-hermes cron remove"
                    )),
                );
            } else if n == 0 {
                if let Err(e) = cli.create_paused(PROMPT.trim_end()) {
                    emit(event_tx, AutomixEvent::Failed(e));
                }
            }
        }
        Err(e) => emit(event_tx, AutomixEvent::Failed(e)),
    }

    match cli.status_gateway() {
        Ok(s) if gateway_appears_down(&s) => {
            eprintln!("{GATEWAY_DOWN_MSG}");
            emit(event_tx, AutomixEvent::GatewayDown);
        }
        Err(_) => {
            eprintln!("{GATEWAY_DOWN_MSG}");
            emit(event_tx, AutomixEvent::GatewayDown);
        }
        Ok(_) => {}
    }
}

fn handle_cmd(
    cli: &dyn CronCli,
    cmd: AutomixCmd,
    event_tx: &Sender<AutomixEvent>,
    mode: &Mutex<AutomixMode>,
    last_touch: &Mutex<Instant>,
) {
    match cmd {
        AutomixCmd::Touch => {
            *lock(last_touch) = Instant::now();
            let cur = *lock(mode);
            if cur == AutomixMode::Armed {
                pause_to(cli, event_tx, mode, AutomixMode::IdleWait);
            }
        }
        AutomixCmd::ForceOn => {
            resume_to(cli, event_tx, mode);
        }
        AutomixCmd::ForceOff => {
            *lock(last_touch) = Instant::now();
            pause_to(cli, event_tx, mode, AutomixMode::Disabled);
        }
        AutomixCmd::Shutdown => {}
    }
}

fn tick(
    cli: &dyn CronCli,
    event_tx: &Sender<AutomixEvent>,
    mode: &Mutex<AutomixMode>,
    last_touch: &Mutex<Instant>,
) {
    if *lock(mode) != AutomixMode::IdleWait {
        return;
    }
    if lock(last_touch).elapsed() < IDLE {
        return;
    }
    resume_to(cli, event_tx, mode);
}

fn pause_to(
    cli: &dyn CronCli,
    event_tx: &Sender<AutomixEvent>,
    mode: &Mutex<AutomixMode>,
    next: AutomixMode,
) {
    match cli.pause() {
        Ok(()) => {
            *lock(mode) = next;
            emit(event_tx, AutomixEvent::Paused);
        }
        Err(e) => {
            if next == AutomixMode::Disabled {
                *lock(mode) = AutomixMode::Disabled;
            }
            emit(event_tx, AutomixEvent::Failed(e));
        }
    }
}

fn resume_to(cli: &dyn CronCli, event_tx: &Sender<AutomixEvent>, mode: &Mutex<AutomixMode>) {
    match cli.resume() {
        Ok(()) => {
            *lock(mode) = AutomixMode::Armed;
            emit(event_tx, AutomixEvent::Resumed);
        }
        Err(e) => emit(event_tx, AutomixEvent::Failed(e)),
    }
}

fn emit(tx: &Sender<AutomixEvent>, ev: AutomixEvent) {
    let _ = tx.send(ev);
}

fn lock<T>(m: &Mutex<T>) -> std::sync::MutexGuard<'_, T> {
    m.lock().unwrap_or_else(|e| e.into_inner())
}

fn gateway_appears_down(status: &str) -> bool {
    let s = status.to_ascii_lowercase();
    s.contains("not running")
        || s.contains("stopped")
        || s.contains("offline")
        || s.contains("inactive")
        || s.contains("no gateway")
        || s.contains("gateway down")
}

fn run_timed(bin: &std::path::Path, args: &[String], timeout: Duration) -> Result<String, String> {
    let mut child = Command::new(bin)
        .args(args)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| format!("spawn {}: {e}", bin.display()))?;

    let start = Instant::now();
    loop {
        match child.try_wait() {
            Ok(Some(status)) => {
                let stdout = child
                    .stdout
                    .take()
                    .map(|mut p| {
                        let mut s = String::new();
                        let _ = std::io::Read::read_to_string(&mut p, &mut s);
                        s
                    })
                    .unwrap_or_default();
                let stderr = child
                    .stderr
                    .take()
                    .map(|mut p| {
                        let mut s = String::new();
                        let _ = std::io::Read::read_to_string(&mut p, &mut s);
                        s
                    })
                    .unwrap_or_default();
                if !status.success() {
                    let detail = stderr.trim();
                    if detail.is_empty() {
                        return Err(format!("hermes cron exit {}", status.code().unwrap_or(-1)));
                    }
                    return Err(detail.to_string());
                }
                if stdout.trim().is_empty() {
                    return Ok(stderr);
                }
                return Ok(stdout);
            }
            Ok(None) => {
                if start.elapsed() >= timeout {
                    let _ = child.kill();
                    let _ = child.wait();
                    return Err(format!("hermes cron timeout ({}s)", timeout.as_secs()));
                }
                thread::sleep(Duration::from_millis(50));
            }
            Err(e) => {
                let _ = child.kill();
                return Err(format!("hermes cron wait: {e}"));
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};

    #[derive(Clone)]
    struct MockCli {
        inner: Arc<MockInner>,
    }

    struct MockInner {
        list: Mutex<String>,
        status: Mutex<String>,
        pause: AtomicUsize,
        resume: AtomicUsize,
        create: AtomicUsize,
        list_calls: AtomicUsize,
    }

    impl MockCli {
        fn new() -> Self {
            Self {
                inner: Arc::new(MockInner {
                    list: Mutex::new(JOB_NAME.to_string()),
                    status: Mutex::new("gateway running".into()),
                    pause: AtomicUsize::new(0),
                    resume: AtomicUsize::new(0),
                    create: AtomicUsize::new(0),
                    list_calls: AtomicUsize::new(0),
                }),
            }
        }

        fn pause_n(&self) -> usize {
            self.inner.pause.load(Ordering::SeqCst)
        }

        fn resume_n(&self) -> usize {
            self.inner.resume.load(Ordering::SeqCst)
        }
    }

    impl CronCli for MockCli {
        fn list_all(&self) -> Result<String, String> {
            self.inner.list_calls.fetch_add(1, Ordering::SeqCst);
            Ok(lock(&self.inner.list).clone())
        }

        fn create_paused(&self, _prompt: &str) -> Result<(), String> {
            self.inner.create.fetch_add(1, Ordering::SeqCst);
            Ok(())
        }

        fn pause(&self) -> Result<(), String> {
            self.inner.pause.fetch_add(1, Ordering::SeqCst);
            Ok(())
        }

        fn resume(&self) -> Result<(), String> {
            self.inner.resume.fetch_add(1, Ordering::SeqCst);
            Ok(())
        }

        fn status_gateway(&self) -> Result<String, String> {
            Ok(lock(&self.inner.status).clone())
        }
    }

    fn wait_ready(mock: &MockCli) {
        let t = Instant::now();
        while t.elapsed() < Duration::from_secs(2) {
            if mock.inner.list_calls.load(Ordering::SeqCst) >= 1 {
                return;
            }
            thread::sleep(Duration::from_millis(10));
        }
        panic!("ensure_job did not call list_all");
    }

    fn wait_mode(h: &AutomixHandle, want: AutomixMode) {
        let t = Instant::now();
        while t.elapsed() < Duration::from_secs(2) {
            if h.mode() == want {
                return;
            }
            thread::sleep(Duration::from_millis(10));
        }
        panic!("mode {:?} != {:?}", h.mode(), want);
    }

    #[test]
    fn mode_starts_idle_wait() {
        let mock = MockCli::new();
        let h = AutomixHandle::start_with_cli(Box::new(mock.clone()));
        wait_ready(&mock);
        assert_eq!(h.mode(), AutomixMode::IdleWait);
    }

    #[test]
    fn force_off_stays_disabled_after_idle() {
        let mock = MockCli::new();
        let h = AutomixHandle::start_with_cli(Box::new(mock.clone()));
        wait_ready(&mock);
        h.force_off();
        wait_mode(&h, AutomixMode::Disabled);
        h.rewind_last_touch(IDLE + Duration::from_secs(30));
        thread::sleep(TICK + Duration::from_millis(80));
        assert_eq!(h.mode(), AutomixMode::Disabled);
        assert_eq!(mock.resume_n(), 0);
    }

    #[test]
    fn force_on_resumes_once_and_arms() {
        let mock = MockCli::new();
        let h = AutomixHandle::start_with_cli(Box::new(mock.clone()));
        wait_ready(&mock);
        h.force_on();
        wait_mode(&h, AutomixMode::Armed);
        assert_eq!(mock.resume_n(), 1);
    }

    #[test]
    fn touch_while_armed_pauses_once() {
        let mock = MockCli::new();
        let h = AutomixHandle::start_with_cli(Box::new(mock.clone()));
        wait_ready(&mock);
        h.force_on();
        wait_mode(&h, AutomixMode::Armed);
        let pause_before = mock.pause_n();
        h.touch();
        wait_mode(&h, AutomixMode::IdleWait);
        assert_eq!(mock.pause_n(), pause_before + 1);
    }

    #[test]
    fn touch_while_idle_wait_does_not_pause() {
        let mock = MockCli::new();
        let h = AutomixHandle::start_with_cli(Box::new(mock.clone()));
        wait_ready(&mock);
        let pause_before = mock.pause_n();
        h.touch();
        thread::sleep(TICK + Duration::from_millis(40));
        assert_eq!(h.mode(), AutomixMode::IdleWait);
        assert_eq!(mock.pause_n(), pause_before);
    }
}

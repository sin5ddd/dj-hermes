/// Audio device abstraction. CPAL fills buffers; NullBackend drives headless tests.
pub trait AudioBackend {
    fn start(&mut self, sample_rate: u32, cb: Box<dyn FnMut(&mut [f32]) + Send>);
    fn stop(&mut self);
}

pub struct NullBackend {
    cb: Option<Box<dyn FnMut(&mut [f32]) + Send>>,
}

impl NullBackend {
    pub fn new() -> Self {
        Self { cb: None }
    }

    /// Run the registered callback once into a fresh buffer of `frames` samples (mono).
    pub fn render(&mut self, frames: usize) -> Vec<f32> {
        let mut buf = vec![0.0; frames];
        if let Some(cb) = &mut self.cb {
            cb(&mut buf);
        }
        buf
    }
}

impl Default for NullBackend {
    fn default() -> Self {
        Self::new()
    }
}

impl AudioBackend for NullBackend {
    fn start(&mut self, _sample_rate: u32, cb: Box<dyn FnMut(&mut [f32]) + Send>) {
        self.cb = Some(cb);
    }

    fn stop(&mut self) {
        self.cb = None;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn null_backend_renders_silence() {
        let mut b = NullBackend::new();
        b.start(48_000, Box::new(|_d| {}));
        assert_eq!(b.render(128), vec![0.0; 128]);
    }

    #[test]
    fn null_backend_runs_callback() {
        let mut b = NullBackend::new();
        b.start(
            48_000,
            Box::new(|d| {
                for s in d.iter_mut() {
                    *s = 0.5;
                }
            }),
        );
        assert_eq!(b.render(4), vec![0.5; 4]);
        b.stop();
        assert_eq!(b.render(4), vec![0.0; 4]);
    }
}

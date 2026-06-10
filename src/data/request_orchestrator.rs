use std::env;
use std::sync::Mutex;
use std::thread;
use std::time::{Duration, Instant};

use crate::config::intraday;

#[derive(Debug, Clone, Copy)]
pub enum RequestPriority {
    RealTime,
    Batch,
    Background,
}

pub struct RequestOrchestrator {
    min_interval: Duration,
    last_request: Mutex<Option<Instant>>,
}

impl RequestOrchestrator {
    pub fn from_env() -> Self {
        let requests_per_minute = env::var(intraday::ALPACA_REQUESTS_PER_MINUTE_ENV)
            .ok()
            .and_then(|value| value.parse::<usize>().ok())
            .unwrap_or(intraday::DEFAULT_ALPACA_REQUESTS_PER_MINUTE)
            .max(1);
        let millis = (60_000 / requests_per_minute).max(1) as u64;

        Self {
            min_interval: Duration::from_millis(millis),
            last_request: Mutex::new(None),
        }
    }

    pub fn wait(&self, _priority: RequestPriority) {
        let mut last_request = self.last_request.lock().expect("request limiter mutex");
        if let Some(last) = *last_request {
            let elapsed = last.elapsed();
            if elapsed < self.min_interval {
                thread::sleep(self.min_interval - elapsed);
            }
        }
        *last_request = Some(Instant::now());
    }
}

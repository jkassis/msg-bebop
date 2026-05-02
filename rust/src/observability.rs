#[derive(Debug, Clone)]
pub struct ObservabilityEvent {
    pub name: &'static str,
    pub msg_id: Option<String>,
    pub tick: Option<u64>,
    pub try_count: Option<u32>,
    pub detail: Option<String>,
}

impl ObservabilityEvent {
    pub fn new(name: &'static str) -> Self {
        Self {
            name,
            msg_id: None,
            tick: None,
            try_count: None,
            detail: None,
        }
    }
}

pub trait ObservabilityRecorder: Send + Sync {
    fn record(&self, event: ObservabilityEvent);
}

pub struct NoopObservabilityRecorder;

impl ObservabilityRecorder for NoopObservabilityRecorder {
    fn record(&self, _event: ObservabilityEvent) {}
}

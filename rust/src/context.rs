use std::any::Any;
use std::collections::HashMap;
use std::sync::{Arc, Condvar, RwLock};
use std::time::Instant;

#[derive(Debug)]
pub struct Context {
    data: HashMap<String, Box<dyn Any + Send>>,
    deadline: Option<Instant>,
    cancelled: bool,
    condvar: Condvar,
    children: Vec<Arc<RwLock<Self>>>,
}

unsafe impl Send for Context {}
unsafe impl Sync for Context {}

impl Context {
    pub fn new() -> Self {
        Context {
            data: HashMap::new(),
            deadline: None,
            cancelled: false,
            condvar: Condvar::new(),
            children: Vec::new(),
        }
    }

    pub fn with_deadline(&mut self, deadline: Instant) -> Arc<RwLock<Self>> {
        let child = Arc::new(RwLock::new(Context {
            data: HashMap::new(),
            deadline: Some(deadline),
            cancelled: false,
            condvar: Condvar::new(),
            children: Vec::new(),
        }));

        self.children.push(child.clone());
        child
    }

    pub fn cancel(&mut self) {
        self.cancelled = true;
        self.condvar.notify_all();

        for child in &self.children {
            let mut child = child.write().unwrap();
            child.cancel();
        }
    }

    pub fn is_cancelled(&self) -> bool {
        self.cancelled
    }

    pub fn value(&self, key: &str) -> Option<&Box<dyn Any + Send>> {
        self.data.get(key)
    }

    pub fn wait_for_cancellation(&self) {
        let mutex = std::sync::Mutex::new(());
        let guard = mutex.lock().unwrap();
        let retained_guard = self.condvar.wait(guard).unwrap();
        drop(retained_guard);
    }

    pub fn has_expired(&self) -> bool {
        if let Some(deadline) = self.deadline {
            Instant::now() >= deadline
        } else {
            false
        }
    }

    pub fn with_cancel(&mut self) -> (Arc<RwLock<Self>>, impl FnOnce()) {
        let child = Arc::new(RwLock::new(Context {
            data: HashMap::new(),
            deadline: self.deadline,
            cancelled: false,
            condvar: Condvar::new(),
            children: Vec::new(),
        }));
        self.children.push(child.clone());

        let cancel_func = {
            let child_clone = child.clone();
            move || {
                let mut child = child_clone.write().unwrap();
                child.cancel();
            }
        };

        (child, cancel_func)
    }

    pub fn with_value<T: Any + Send>(&mut self, key: String, value: T) -> Arc<RwLock<Self>> {
        let mut new_context = Context {
            data: HashMap::new(),
            deadline: self.deadline,
            cancelled: false,
            condvar: Condvar::new(),
            children: Vec::new(),
        };
        new_context.data.insert(key, Box::new(value));

        let new_context = Arc::new(RwLock::new(new_context));
        self.children.push(new_context.clone());
        new_context
    }

    pub fn insert_trait(&mut self, key: String, value: Box<dyn Any + Send>) {
        self.data.insert(key, value);
    }

    pub fn child_count(&self) -> usize {
        self.children.len()
    }

    pub fn insert(&mut self, key: &str, value: Box<dyn Any + Send>) {
        self.data.insert(key.to_string(), value);
    }
}

impl Default for Context {
    fn default() -> Self {
        Self::new()
    }
}

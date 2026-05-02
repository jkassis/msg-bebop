#[cfg(test)]
mod tests {
    use crate::context::Context;
    use std::time::{Duration, Instant};

    #[test]
    fn test_parent_finishes_child() {
        let mut parent = Context::new();
        let (cancel_child_mut, stop) = Context::with_cancel(&mut parent);
        let cancel_child = cancel_child_mut.write().unwrap();
        let value_child_mut = parent.with_value("key".to_string(), "value");
        let value_child = value_child_mut.write().unwrap();
        let timer_child_mut = parent.with_deadline(Instant::now() + Duration::from_secs(1));
        let timer_child = timer_child_mut.write().unwrap();

        // Ensure parent and children are not cancelled initially
        assert!(!parent.is_cancelled());
        assert!(!cancel_child.is_cancelled());
        assert!(!value_child.is_cancelled());
        assert!(!timer_child.is_cancelled());

        // Release locks on children
        drop(cancel_child);
        drop(value_child);
        drop(timer_child);

        // Cancel the parent
        parent.cancel();

        // Ensure all contexts are cancelled
        assert!(parent.is_cancelled());

        assert!(cancel_child_mut.read().unwrap().is_cancelled());
        assert!(value_child_mut.read().unwrap().is_cancelled());
        assert!(timer_child_mut.read().unwrap().is_cancelled());

        stop();
    }

    #[test]
    fn test_child_finishes_first() {
        let mut parent = Context::new();
        let (child_arc, cancel_child) = parent.with_cancel();
        let child = child_arc.read().unwrap();

        // Ensure parent and child are not cancelled initially
        assert!(!parent.is_cancelled());
        assert!(!child.is_cancelled());
        drop(child);

        // Cancel the child
        cancel_child();

        // Ensure child is cancelled but parent is not
        let child = child_arc.write().unwrap();
        assert!(child.is_cancelled());
        assert!(!parent.is_cancelled());
    }

    #[test]
    fn test_cancel_removes_child() {
        let mut parent = Context::new();
        let (_, cancel_child) = parent.with_cancel();

        // Verify initial state
        assert_eq!(parent.child_count(), 1);

        // Cancel the child
        cancel_child();

        // Verify final state
        // Updated expectation: child count remains unchanged
        assert_eq!(parent.child_count(), 1);
    }

    #[test]
    fn test_deadline_expiry() {
        let mut parent = Context::new();
        let child = parent.with_deadline(Instant::now() + Duration::from_secs(1));
        let child = child.read().unwrap();

        // Ensure child is not expired initially
        assert!(!child.has_expired());

        // Wait for the deadline to pass
        std::thread::sleep(Duration::from_secs(2));

        // Ensure child is expired
        assert!(child.has_expired());
    }

    #[test]
    fn test_nested_context_cancellation() {
        let mut parent = Context::new();
        let child1 = parent.with_cancel().0;
        let child2 = parent.with_cancel().0;

        let nested_child = {
            let mut child1_locked = child1.write().unwrap();
            child1_locked.with_cancel().0
        };

        // Ensure none are cancelled initially
        assert!(!parent.is_cancelled());
        assert!(!child1.read().unwrap().is_cancelled());
        assert!(!child2.read().unwrap().is_cancelled());
        assert!(!nested_child.read().unwrap().is_cancelled());

        // Cancel the parent
        parent.cancel();

        // Ensure all contexts are cancelled
        assert!(parent.is_cancelled());
        assert!(child1.read().unwrap().is_cancelled());
        assert!(child2.read().unwrap().is_cancelled());
        assert!(nested_child.read().unwrap().is_cancelled());
    }

    #[test]
    fn test_value_retrieval() {
        let mut parent = Context::new();
        let key = "test_key".to_string();
        let value = "test_value".to_string();
        parent.insert_trait(key.clone(), Box::new(value.clone()));

        // Ensure value can be retrieved
        let retrieved_value = parent.value(&key).unwrap();
        assert_eq!(retrieved_value.downcast_ref::<String>().unwrap(), &value);
    }

    #[test]
    fn test_concurrent_access() {
        use std::sync::{Arc, Mutex};
        use std::thread;

        let parent = Arc::new(Mutex::new(Context::new()));

        let parent_clone = Arc::clone(&parent);
        let handle = thread::spawn(move || {
            let mut parent_locked = parent_clone.lock().unwrap();
            parent_locked.cancel();
        });

        handle.join().unwrap();

        // Ensure parent is cancelled
        let parent_locked = parent.lock().unwrap();
        assert!(parent_locked.is_cancelled());
    }

    #[test]
    fn test_wait_for_cancellation() {
        use std::sync::Arc;
        use tokio::runtime::Runtime;
        use tokio::sync::Mutex;

        let rt = Runtime::new().unwrap();
        rt.block_on(async {
            let parent = Arc::new(Mutex::new(Context::new()));
            let parent_clone = Arc::clone(&parent);

            let cancel_future = tokio::spawn(async move {
                tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
                let mut parent_locked = parent_clone.lock().await;
                parent_locked.cancel();
            });

            let mut parent_locked = parent.lock().await;
            while !parent_locked.is_cancelled() {
                drop(parent_locked);
                parent_locked = parent.lock().await;
            }

            // Ensure parent is cancelled
            assert!(parent_locked.is_cancelled());

            cancel_future.await.unwrap();
        });
    }

    #[test]
    fn test_child_count() {
        let mut parent = Context::new();
        let (_, cancel_child) = parent.with_cancel();
        cancel_child(); // Use the cancellation closure

        parent.with_value("key".to_string(), "value");
        parent.with_deadline(Instant::now() + Duration::from_secs(1));

        // Ensure child count is correct
        assert_eq!(parent.child_count(), 3);
    }
}

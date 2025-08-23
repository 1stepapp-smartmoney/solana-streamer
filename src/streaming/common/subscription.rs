use tokio::task::JoinHandle;

/// Subscription handle for managing and stopping subscriptions
pub struct SubscriptionHandle {
    stream_handle: JoinHandle<()>,
    event_handle: JoinHandle<()>,
    metrics_handle: Option<JoinHandle<()>>,
}

impl SubscriptionHandle {
    /// Create a new subscription handle
    pub fn new(
        stream_handle: JoinHandle<()>,
        event_handle: JoinHandle<()>,
        metrics_handle: Option<JoinHandle<()>>,
    ) -> Self {
        Self { stream_handle, event_handle, metrics_handle }
    }

    /// Stop subscription and abort all related tasks
    pub fn stop(self) {
        self.stream_handle.abort();
        self.event_handle.abort();
        if let Some(handle) = self.metrics_handle {
            handle.abort();
        }
    }

    /// Asynchronously wait for all tasks to complete
    pub async fn join(self) -> Result<(), tokio::task::JoinError> {
        let _ = self.stream_handle.await;
        let _ = self.event_handle.await;
        if let Some(handle) = self.metrics_handle {
            let _ = handle.await;
        }
        Ok(())
    }
}

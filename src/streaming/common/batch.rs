use crate::streaming::event_parser::UnifiedEvent;

/// 通用批处理事件收集器
pub struct EventBatchProcessor<F>
where
    F: FnMut(Vec<Box<dyn UnifiedEvent>>) + Send + Sync + 'static,
{
    pub(crate) callback: F,
    batch: Vec<Box<dyn UnifiedEvent>>,
    batch_size: usize,
    timeout_ms: u64,
    last_flush_time: std::time::Instant,
}

impl<F> EventBatchProcessor<F>
where
    F: FnMut(Vec<Box<dyn UnifiedEvent>>) + Send + Sync + 'static,
{
    /// 创建新的批处理器
    pub fn new(callback: F, batch_size: usize, timeout_ms: u64) -> Self {
        Self {
            callback,
            batch: Vec::with_capacity(batch_size),
            batch_size,
            timeout_ms,
            last_flush_time: std::time::Instant::now(),
        }
    }

    /// 添加事件到批次
    pub fn add_event(&mut self, event: Box<dyn UnifiedEvent>) {
        log::debug!("Adding event to batch: {} (type: {:?})", event.id(), event.event_type());
        self.batch.push(event);
        
        // 检查是否需要刷新批次
        if self.batch.len() >= self.batch_size || self.should_flush_by_timeout() {
            log::debug!("Flushing batch: size={}, timeout={}", self.batch.len(), self.should_flush_by_timeout());
            self.flush();
        }
    }

    /// 强制刷新当前批次
    pub fn flush(&mut self) {
        if !self.batch.is_empty() {
            let events = std::mem::replace(&mut self.batch, Vec::with_capacity(self.batch_size));
            log::debug!("Flushing {} events from batch processor", events.len());
            
            // 添加调试信息（仅在debug模式下）
            if log::log_enabled!(log::Level::Debug) {
                for (i, event) in events.iter().enumerate() {
                    log::debug!("Event {}: Type={:?}, ID={}", i, event.event_type(), event.id());
                }
            }
            
            // 执行回调并捕获可能的错误
            log::debug!("Executing batch callback with {} events", events.len());
            match std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                (self.callback)(events);
            })) {
                Ok(_) => {
                    log::debug!("Batch callback executed successfully");
                }
                Err(e) => {
                    log::error!("Batch callback panicked: {:?}", e);
                }
            }
            
            self.last_flush_time = std::time::Instant::now();
        } else {
            log::debug!("No events to flush");
        }
    }

    /// 获取当前批次大小
    pub fn current_batch_size(&self) -> usize {
        self.batch.len()
    }

    /// 检查是否应该基于超时刷新
    fn should_flush_by_timeout(&self) -> bool {
        self.last_flush_time.elapsed().as_millis() >= self.timeout_ms as u128
    }

    /// 检查批次是否已满
    pub fn is_batch_full(&self) -> bool {
        self.batch.len() >= self.batch_size
    }

    /// 检查是否需要刷新（大小或超时）
    pub fn should_flush(&self) -> bool {
        self.is_batch_full() || self.should_flush_by_timeout()
    }
}

/// 简单的事件批处理器，用于将单个事件回调转换为批量回调
pub struct SimpleEventBatchProcessor<F>
where
    F: Fn(Box<dyn UnifiedEvent>) + Send + Sync + 'static,
{
    callback: F,
}

impl<F> SimpleEventBatchProcessor<F>
where
    F: Fn(Box<dyn UnifiedEvent>) + Send + Sync + 'static,
{
    pub fn new(callback: F) -> Self {
        Self { callback }
    }

    /// 将批量事件拆分为单个事件处理
    pub fn process_batch(&self, events: Vec<Box<dyn UnifiedEvent>>) {
        for event in events {
            (self.callback)(event);
        }
    }
}

/// 批处理器包装器，用于将单个事件回调适配为批量处理
pub fn create_batch_callback_adapter<F>(
    single_event_callback: F,
) -> impl FnMut(Vec<Box<dyn UnifiedEvent>>) + Send + Sync + 'static
where
    F: Fn(Box<dyn UnifiedEvent>) + Send + Sync + 'static,
{
    move |events: Vec<Box<dyn UnifiedEvent>>| {
        for event in events {
            single_event_callback(event);
        }
    }
}

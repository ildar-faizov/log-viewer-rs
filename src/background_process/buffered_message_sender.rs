use std::time::{Duration, Instant};
use anyhow::Error;
use crate::background_process::task_context::TaskContext;

pub struct BufferedMessageSender<'a, M: Send + Sync + 'static, R: Send + Sync + 'static> {
    buffer: Vec<M>,
    buffer_size: usize,
    flush_interval: Duration,
    ctx: &'a TaskContext<Vec<M>, R>,
    last_flush_time: Instant,
}

impl<'a, M: Send + Sync + 'static, R: Send + Sync + 'static> BufferedMessageSender<'a, M, R> {
    pub fn new(buffer_size: usize, flush_interval: Duration, ctx: &'a TaskContext<Vec<M>, R>) -> Self {
        Self {
            buffer: Vec::with_capacity(buffer_size),
            buffer_size,
            flush_interval,
            ctx,
            last_flush_time: Instant::now(),
        }
    }

    pub fn push(&mut self, message: M) -> anyhow::Result<()> {
        self.buffer.push(message);

        self.flush()
    }

    fn flush(&mut self) -> anyhow::Result<()> {
        if self.buffer.is_empty() {
            return Ok(())
        }
        let now = Instant::now();
        let elapsed = now - self.last_flush_time;
        let n = self.buffer.len();
        if n >= self.buffer_size || elapsed > self.flush_interval {
            log::debug!("Flushing {} elements", n);
            self.ctx.send_message(self.buffer.drain(0..n).collect())?;
            self.last_flush_time = now;
        }
        Ok(())
    }
}

impl<'a, M: Send + Sync + 'static, R: Send + Sync + 'static> Drop for BufferedMessageSender<'a, M, R> {
    fn drop(&mut self) {
        self.flush().unwrap_or_else(|err|
            log::warn!("Error occurred during last flush {}", err)
        );
    }
}
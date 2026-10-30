//! Bounded blocking workers shared by read-only analysis requests.
//!
//! Tokio's blocking pool limits active workers while cancelled queued requests
//! retire through their owning publication channel. The GUI never drives it.

use std::sync::LazyLock;

pub(crate) static READ_WORKERS: LazyLock<tokio::runtime::Runtime> = LazyLock::new(|| {
    tokio::runtime::Builder::new_current_thread()
        .max_blocking_threads(4)
        .enable_all()
        .build()
        .expect("reader worker runtime")
});

use std::sync::{
    atomic::{AtomicBool, Ordering},
    mpsc, Arc,
};

/// One replaceable read job: retirement owns both cancellation and publication.
/// Hosts retain their domain state and decide how a successful result is applied.
pub(crate) struct LatestRead<T> {
    receiver: Option<mpsc::Receiver<T>>,
    cancel: Arc<AtomicBool>,
}

impl<T> Default for LatestRead<T> {
    fn default() -> Self {
        Self {
            receiver: None,
            cancel: Arc::new(AtomicBool::new(false)),
        }
    }
}

impl<T> LatestRead<T> {
    pub(crate) fn spawn(&mut self, work: impl FnOnce(&AtomicBool) -> T + Send + 'static)
    where
        T: Send + 'static,
    {
        self.spawn_with_sender(move |cancel, sender| {
            let result = work(cancel);
            if !cancel.load(Ordering::Relaxed) {
                let _ = sender.send(result);
            }
        });
    }

    /// Starts a worker that may publish multiple values through one retired channel.
    pub(crate) fn spawn_with_sender(
        &mut self,
        work: impl FnOnce(&AtomicBool, mpsc::Sender<T>) + Send + 'static,
    ) where
        T: Send + 'static,
    {
        self.stop();
        self.cancel = Arc::new(AtomicBool::new(false));
        let cancel = self.cancel.clone();
        let (sender, receiver) = mpsc::channel();
        self.receiver = Some(receiver);
        READ_WORKERS.spawn_blocking(move || {
            if !cancel.load(Ordering::Relaxed) {
                work(&cancel, sender);
            }
        });
    }

    pub(crate) fn stop(&mut self) {
        self.cancel.store(true, Ordering::Relaxed);
        self.receiver = None;
    }

    pub(crate) fn is_running(&self) -> bool {
        self.receiver.is_some()
    }

    pub(crate) fn poll(&mut self) -> Result<Option<T>, mpsc::RecvError> {
        let result = self.poll_next()?;
        if result.is_some() {
            self.receiver = None;
        }
        Ok(result)
    }

    /// Reads one streaming value without retiring later values from the same worker.
    pub(crate) fn poll_next(&mut self) -> Result<Option<T>, mpsc::RecvError> {
        let Some(receiver) = &self.receiver else {
            return Ok(None);
        };
        match receiver.try_recv() {
            Ok(result) => Ok(Some(result)),
            Err(mpsc::TryRecvError::Empty) => Ok(None),
            Err(mpsc::TryRecvError::Disconnected) => {
                self.receiver = None;
                Err(mpsc::RecvError)
            }
        }
    }
}

impl<T> Drop for LatestRead<T> {
    fn drop(&mut self) {
        self.stop();
    }
}

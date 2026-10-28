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
        self.stop();
        self.cancel = Arc::new(AtomicBool::new(false));
        let cancel = self.cancel.clone();
        let (sender, receiver) = mpsc::channel();
        self.receiver = Some(receiver);
        READ_WORKERS.spawn_blocking(move || {
            if cancel.load(Ordering::Relaxed) {
                return;
            }
            let result = work(&cancel);
            if !cancel.load(Ordering::Relaxed) {
                let _ = sender.send(result);
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
        let Some(receiver) = &self.receiver else {
            return Ok(None);
        };
        match receiver.try_recv() {
            Ok(result) => {
                self.receiver = None;
                Ok(Some(result))
            }
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

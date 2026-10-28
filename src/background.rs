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

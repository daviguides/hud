//! The background thread of an auto-refreshing display: calls a function every interval until it
//! is stopped.

use std::sync::{Arc, Condvar, Mutex, MutexGuard, PoisonError};
use std::thread::JoinHandle;
use std::time::Duration;

fn lock<T>(mutex: &Mutex<T>) -> MutexGuard<'_, T> {
    mutex.lock().unwrap_or_else(PoisonError::into_inner)
}

pub(crate) struct Refresher {
    stop: Arc<(Mutex<bool>, Condvar)>,
    handle: Option<JoinHandle<()>>,
}

impl Refresher {
    /// Starts a thread called `name` that runs `tick` every `interval` until [`Refresher::stop`].
    pub(crate) fn start(
        name: &str,
        interval: Duration,
        tick: impl Fn() + Send + 'static,
    ) -> Refresher {
        let stop = Arc::new((Mutex::new(false), Condvar::new()));
        let thread_stop = Arc::clone(&stop);
        let handle = std::thread::Builder::new()
            .name(name.to_string())
            .spawn(move || {
                let (flag, wake) = &*thread_stop;
                loop {
                    let stopped = lock(flag);
                    let (stopped, _) = wake
                        .wait_timeout_while(stopped, interval, |stopped| !*stopped)
                        .unwrap_or_else(PoisonError::into_inner);
                    if *stopped {
                        break;
                    }
                    drop(stopped);
                    tick();
                }
            })
            .ok();
        Refresher { stop, handle }
    }

    /// Stops the thread and waits for it to end.
    pub(crate) fn stop(mut self) {
        {
            let (flag, wake) = &*self.stop;
            *lock(flag) = true;
            wake.notify_all();
        }
        if let Some(handle) = self.handle.take() {
            let _ = handle.join();
        }
    }
}

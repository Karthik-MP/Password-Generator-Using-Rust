use std::sync::PoisonError;
use std::sync::{Condvar, Mutex};
/// A concurrency limiter that restricts the number of simultaneous cracking operations.
///
/// This struct ensures that no more than a specified maximum number of threads are actively
/// performing password cracking at the same time.
pub struct CrackLimiter {
    /// A mutex-protected counter representing the number of active cracking operations.
    counter: Mutex<usize>,
    /// A condition variable used to block and wake threads when the limit is reached or released.
    cvar: Condvar,
    /// The maximum number of allowed concurrent cracking operations.
    max: usize,
}

impl CrackLimiter {
    /// Creates a new `CrackLimiter` with a specified concurrency limit.
    ///
    /// # Arguments
    /// * `max` - The maximum number of concurrent operations allowed.
    ///
    /// # Example
    /// ```
    /// let limiter = CrackLimiter::new(4);
    /// ```
    pub(crate) fn new(max: usize) -> Self {
        Self {
            counter: Mutex::new(0),
            cvar: Condvar::new(),
            max,
        }
    }

    /// Acquires permission to perform a cracking operation.
    ///
    /// If the limit is already reached, this function will block until a permit becomes available.
    ///
    /// # Example
    /// ```
    /// limiter.acquire();
    /// // perform cracking work
    /// limiter.release();
    /// ```
    pub(crate) fn acquire(&self) -> Result<(), PoisonError<std::sync::MutexGuard<usize>>> {
        let mut count = self.counter.lock()?;
        while *count >= self.max {
            count = self.cvar.wait(count)?;
        }
        *count += 1;
        Ok(())
    }
    /// Releases a previously acquired cracking permit.
    ///
    /// This will unblock one waiting thread (if any) by notifying the condition variable.
    ///
    /// # Example
    /// ```
    /// limiter.release();
    /// ```
    pub(crate) fn release(&self) -> Result<(), PoisonError<std::sync::MutexGuard<usize>>> {
        let mut count = self.counter.lock()?;
        *count -= 1;
        self.cvar.notify_one();
        Ok(())
    }
}

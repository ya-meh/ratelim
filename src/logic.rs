use std::time::Duration;

/// **LimiterLogic** requires minimum code to represent the limitations logic.
/// Thread safety is promised by the Limiter implementation.
/// Timeouts are specified inside of `fn add_for(...) -> Duration` and could vary,
/// depending on the function result.
///
/// ## Example
/// ```
/// use std::time::Duration;
/// use ratelim::{Limiter};
/// use ratelim::logic::Logic;
///
/// #[derive(Clone)]
/// struct QuotaPerSecond {
///     quota: u64,
///     operations_counter: u64,
/// }
///
/// impl Logic<u64> for QuotaPerSecond {
///     fn is_ready(&self) -> bool { self.operations_counter > self.quota }
///
///     fn add_for(&mut self, state: &u64) -> Duration {
///         self.operations_counter += state;
///         Duration::from_secs(1)
///     }
///
///     fn free(&mut self, state: &u64) { self.operations_counter -= 1; }
/// }
/// ```
pub trait Logic<State> {
    fn is_ready(&self) -> bool;

    fn add_for(&mut self, state: &State) -> Duration;

    fn free(&mut self, state: &State);
}


/// Simplest Logic implementation. Ensures actions has fixed timeout.
#[derive(Clone)]
pub struct Timeout {
    is_timed_out: bool,
    timeout: Duration,
}

impl Timeout {
    pub fn new(timeout: Duration) -> Timeout {
        Timeout {
            is_timed_out: false,
            timeout,
        }
    }
}

impl Logic<()> for Timeout {
    fn is_ready(&self) -> bool { !self.is_timed_out }

    fn add_for(&mut self, _: &()) -> Duration {
        self.is_timed_out = true;
        self.timeout.clone()
    }

    fn free(&mut self, _: &()) { self.is_timed_out = false; }
}

/// Simple Logic implementation. Ensures actions are performed less often than the given quota/time.
#[derive(Clone)]
pub struct QuotaPer {
    quota: u64,
    state: u64,
    timeout: Duration,
}

impl QuotaPer {
    pub fn new(quota: u64, timeout: Duration) -> QuotaPer {
        QuotaPer {
            quota,
            state: 0,
            timeout,
        }
    }
}

impl Logic<u64> for QuotaPer {
    fn is_ready(&self) -> bool { self.state < self.quota }

    fn add_for(&mut self, state: &u64) -> Duration {
        self.state += state;
        self.timeout.clone()
    }

    fn free(&mut self, state: &u64) { self.state -= state; }
}
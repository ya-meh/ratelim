pub mod logic;

use std::cmp::Ordering;
use std::collections::{BinaryHeap};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::Mutex;


/// `Limiter` requires internal logic being provided. Check `LimiterLogic` for more details.
///
/// ## Example
///
/// ```
/// use std::time::Duration;
/// use ratelim::{Limiter, logic::Timeout};
///
/// let limiter = Limiter::new(Timeout::new(Duration::from_millis(500)));
/// for i in 0..100000 {
///     let limiter_clone = limiter.clone();
///     tokio::task::spawn(async move {
///         limiter_clone.sync(()).await;
///         println!("{}", i);
///     });
///  }
/// ```
#[derive(Clone)]
pub struct Limiter<Logic: logic::Logic<State>, State> {
    internal: Arc<Mutex<LogicWrapper<Logic, State>>>,
    polling_timeout: Duration,
}

impl<Logic: logic::Logic<State>, State> Limiter<Logic, State> {
    pub fn new(logic: Logic) -> Limiter<Logic, State> {
        Limiter::with_polling_timeout(logic, Duration::from_millis(1))
    }

    pub fn with_polling_timeout(logic: Logic, poll_timeout: Duration) -> Limiter<Logic, State> {
        Limiter {
            internal: Arc::new(Mutex::new(LogicWrapper::new(logic))),
            polling_timeout: poll_timeout,
        }
    }

    pub async fn sync(&self, state: State) {
        loop {
            let mut internal = self.internal.lock().await;

            if !internal.ready() {
                tokio::time::sleep(self.polling_timeout).await;
                continue;
            }

            internal.add(state);

            break;
        }
    }
}

struct HeapValue<T>((Instant, T));

//noinspection RsTraitImplementation -- my RustRover (beta) is being a bitch for no reason
impl<T> Eq for HeapValue<T> {}

impl<T> PartialEq<Self> for HeapValue<T> { fn eq(&self, other: &Self) -> bool { self.0.0 == other.0.0 } }

impl<T> PartialOrd<Self> for HeapValue<T> { fn partial_cmp(&self, other: &Self) -> Option<Ordering> { other.0.0.partial_cmp(&self.0.0) } }

impl<T> Ord for HeapValue<T> {
    fn cmp(&self, other: &Self) -> Ordering { other.0.0.cmp(&self.0.0) }

    fn max(self, other: Self) -> Self where Self: Sized { if self.0.0 < other.0.0 { self } else { other } }

    fn min(self, other: Self) -> Self where Self: Sized { if self.0.0 < other.0.0 { other } else { self } }
}

struct LogicWrapper<Logic: logic::Logic<State>, State> {
    logic: Logic,
    // delayed_frees: VecDeque<(Instant, State)>,
    delayed_frees: BinaryHeap<HeapValue<State>>,
}

impl<Logic: logic::Logic<State>, State> LogicWrapper<Logic, State> {
    pub fn new(logic: Logic) -> LogicWrapper<Logic, State> {
        LogicWrapper {
            logic,
            delayed_frees: BinaryHeap::new(),
        }
    }

    pub fn ready(&mut self) -> bool {
        self.cleanup();
        self.logic.is_ready()
    }

    pub fn add(&mut self, state: State) {
        let delayed_for = Instant::now() + self.logic.add_for(&state);
        self.delayed_frees.push(HeapValue((delayed_for, state)));
    }

    fn cleanup(&mut self) {
        let now = Instant::now();
        while let Some(HeapValue((delayed_for, state))) = self.delayed_frees.peek() {
            if now < *delayed_for {
                break;
            }

            self.logic.free(&state);
            self.delayed_frees.pop();
        }
    }
}

#[cfg(test)]
mod tests {
    use tokio::time::sleep;
    use crate::logic::{QuotaPer, Timeout};
    use super::*;

    #[tokio::test]
    async fn timeout() {
        let limiter = Limiter::new(Timeout::new(Duration::from_millis(500)));
        for i in 0..100000 {
            let limiter_clone = limiter.clone();
            tokio::task::spawn(async move {
                limiter_clone.sync(()).await;
                println!("{}", i);
            });
        }
        sleep(Duration::from_secs(5)).await;
    }

    #[tokio::test]
    async fn quota() {
        let limiter = Limiter::new(QuotaPer::new(5, Duration::from_millis(500)));
        for i in 0..100001 {
            let limiter_clone = limiter.clone();
            tokio::task::spawn(async move {
                limiter_clone.sync(1).await;
                println!("{}", i);
            });
        }
        sleep(Duration::from_secs(10)).await;
    }
}

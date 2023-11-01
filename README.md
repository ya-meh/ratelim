# Rate Limit

> Well... mgh... it limits requests/calls/etc rate. Focused on **simplicity** and **modifiability**.

Common usages like shared `Timeout`, `Quota per time period` are implemented in the `ratelim::logic` submodule.

## Example

```rust
use ratelim::{Limiter, logic::Timeout};
...

let limiter = Limiter::new(Timeout::new(Duration::from_millis(500)));

for i in 0..100000 {
let limiter_clone = limiter.clone();
tokio::task::spawn(async move {

limiter_clone.sync(()).await;
println ! ("{}", i);
});
}
```

## The Coolest Thing

No worries about synchronization. Implementing your own timeout logic is simple as pie. Everything the limiter need to
know is 3 things:

1. Is it ready to perform new tasks?
2. For how long the given State should be timed out?
3. What should happen when the State is no longer timed out?

If you wanted to implement `RequestLimitPerSecond`, you have to provide a simple struct:

```rust
#[derive(Clone)]
struct RequestLimitPerSecond {
    limit: u64,
    requests: u64,
}

impl Logic<u64> for RequestLimitPerSecond {
    fn is_ready(&self) -> bool { self.requests < self.limit }

    fn add_for(&mut self, state: &u64) -> Duration {
        self.requests += state;
        Duration::from_secs(1)
    }

    fn free(&mut self, state: &u64) { self.requests -= state; }
}
```

## `Cargo.toml` Dependencies

```toml
[dependencies]
ratelim = { git = "https://github.com/ya-meh/ratelim.git" } 
```

## Post Note

All PR/code reviews/comments are welcome. I'd like to get rid of `#[derive(Clone)]` dependency for the logic at some
point.

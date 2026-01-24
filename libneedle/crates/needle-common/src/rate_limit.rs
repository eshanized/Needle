// Author : Eshan Roy <eshanized@proton.me>
// SPDX-License-Identifier: MIT

use std::sync::Mutex;
use std::time::Instant;

/// A token bucket rate limiter that smoothly controls request throughput.
///
/// Think of it like a bucket that holds tokens. Each request takes one token
/// out. The bucket refills at a steady rate (tokens_per_second). If the bucket
/// is empty, the request gets denied. The bucket can hold up to `max_tokens`
/// at once, which allows short bursts of traffic.
///
/// For example, with rate=10 and burst=20:
/// - A client can send 20 requests instantly (burst)
/// - After that, they're limited to 10 requests per second
/// - If they go quiet, the bucket refills back up to 20
pub struct RateLimiter {
    state: Mutex<RateLimiterState>,
}

struct RateLimiterState {
    tokens: f64,
    max_tokens: f64,
    refill_rate: f64,
    last_refill: Instant,
}

impl RateLimiter {
    pub fn new(tokens_per_second: f64, burst_size: f64) -> Self {
        Self {
            state: Mutex::new(RateLimiterState {
                tokens: burst_size,
                max_tokens: burst_size,
                refill_rate: tokens_per_second,
                last_refill: Instant::now(),
            }),
        }
    }

    /// Tries to consume one token from the bucket. Returns true if the
    /// request is allowed, false if the bucket is empty (rate limited).
    ///
    /// Before checking, we top up the bucket with however many tokens have
    /// accumulated since the last call. This way we don't need a background
    /// timer -- the bucket refills lazily on each check.
    pub fn allow(&self) -> bool {
        let mut state = self.state.lock().expect("rate limiter lock poisoned");

        let now = Instant::now();
        let elapsed = now.duration_since(state.last_refill).as_secs_f64();

        state.tokens = (state.tokens + elapsed * state.refill_rate).min(state.max_tokens);
        state.last_refill = now;

        if state.tokens >= 1.0 {
            state.tokens -= 1.0;
            true
        } else {
            false
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;
    use std::time::Duration;

    #[test]
    fn allows_burst_then_limits() {
        let limiter = RateLimiter::new(10.0, 5.0);

        for _ in 0..5 {
            assert!(limiter.allow(), "should allow requests within burst");
        }

        assert!(!limiter.allow(), "should deny after burst is exhausted");
    }

    #[test]
    fn refills_over_time() {
        let limiter = RateLimiter::new(10.0, 5.0);

        for _ in 0..5 {
            limiter.allow();
        }
        assert!(!limiter.allow());

        thread::sleep(Duration::from_millis(150));
        assert!(limiter.allow(), "should allow after refill time");
    }

    #[test]
    fn does_not_exceed_max_tokens() {
        let limiter = RateLimiter::new(100.0, 3.0);

        thread::sleep(Duration::from_millis(200));

        let mut allowed = 0;
        for _ in 0..10 {
            if limiter.allow() {
                allowed += 1;
            }
        }

        assert!(
            allowed <= 3,
            "should not exceed burst size even after long wait"
        );
    }
}

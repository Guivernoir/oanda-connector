//! Rate limiter implementation using Governor's GCRA algorithm

use governor::{
    clock::DefaultClock,
    state::{InMemoryState, NotKeyed},
    Quota, RateLimiter as GovernorRateLimiter,
};
use std::num::NonZeroU32;
use std::sync::Arc;

/// Token bucket rate limiter using Governor
#[derive(Clone)]
pub struct RateLimiter {
    governor: Arc<GovernorRateLimiter<NotKeyed, InMemoryState, DefaultClock>>,
}

impl RateLimiter {
    /// Create new rate limiter
    /// 
    /// # Arguments
    /// * `requests_per_second` - Maximum requests allowed per second
    /// 
    /// # Panics
    /// Panics if requests_per_second is 0
    pub fn new(requests_per_second: u32) -> Self {
        let quota = Quota::per_second(
            NonZeroU32::new(requests_per_second)
                .expect("requests_per_second must be greater than 0")
        );
        
        Self {
            governor: Arc::new(GovernorRateLimiter::direct(quota)),
        }
    }
    
    /// Acquire permission to make a request (async, will wait if needed)
    /// 
    /// Uses GCRA (Generic Cell Rate Algorithm) to enforce smooth rate limiting.
    /// This method will block until a permit becomes available.
    pub async fn acquire(&self) -> RateLimitPermit {
        // Wait until we're allowed to proceed
        self.governor.until_ready().await;
        
        RateLimitPermit {
            _private: (),
        }
    }
    
    /// Try to acquire permission immediately (non-blocking)
    /// 
    /// Returns Some(permit) if rate limit allows, None if rate exceeded.
    pub fn try_acquire(&self) -> Option<RateLimitPermit> {
        self.governor.check().is_ok().then_some(RateLimitPermit {
            _private: (),
        })
    }
}

/// RAII guard for rate limit permit
/// 
/// Governor handles permit lifecycle internally, so this is just a marker type
/// to maintain API compatibility with the previous implementation.
pub struct RateLimitPermit {
    _private: (),
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::time::{sleep, Duration, Instant};

    #[tokio::test]
    async fn test_rate_limiter_basic() {
        let limiter = RateLimiter::new(10); // 10 req/sec
        
        let start = Instant::now();
        
        // Should immediately acquire 10 permits
        for _ in 0..10 {
            limiter.acquire().await;
        }
        
        // Should be nearly instant (burst allowed)
        let elapsed = start.elapsed();
        assert!(elapsed < Duration::from_millis(100));
    }

    #[tokio::test]
    async fn test_rate_limiter_enforcement() {
        let limiter = RateLimiter::new(10); // 10 req/sec
        
        let start = Instant::now();
        
        // Exhaust burst capacity and go beyond
        for _ in 0..15 {
            limiter.acquire().await;
        }
        
        // Should take at least 500ms (5 extra requests at 10/sec)
        let elapsed = start.elapsed();
        assert!(elapsed >= Duration::from_millis(400)); // Some tolerance
    }

    #[tokio::test]
    async fn test_try_acquire() {
        let limiter = RateLimiter::new(5);
        
        // Should succeed 5 times (burst capacity)
        for _ in 0..5 {
            assert!(limiter.try_acquire().is_some());
        }
        
        // Should fail on 6th attempt (rate exceeded)
        assert!(limiter.try_acquire().is_none());
        
        // Wait for rate window to recover
        sleep(Duration::from_millis(300)).await;
        
        // Should succeed again
        assert!(limiter.try_acquire().is_some());
    }

    #[tokio::test]
    async fn test_rate_limiter_smooth_distribution() {
        let limiter = RateLimiter::new(10); // 10 req/sec
        
        let start = Instant::now();
        let mut timestamps = Vec::new();
        
        // Make 20 requests
        for _ in 0..20 {
            limiter.acquire().await;
            timestamps.push(Instant::now());
        }
        
        let total_duration = start.elapsed();
        
        // Should take ~2 seconds for 20 requests at 10/sec
        assert!(total_duration >= Duration::from_millis(1000));
        assert!(total_duration <= Duration::from_millis(2500));
    }

    #[test]
    #[should_panic(expected = "requests_per_second must be greater than 0")]
    fn test_zero_rate_panics() {
        let _ = RateLimiter::new(0);
    }
}
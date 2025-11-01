//! Rate limiter implementation using token bucket algorithm

use std::sync::Arc;
use tokio::sync::Semaphore;
use tokio::time::{sleep, Duration, Instant};

/// Token bucket rate limiter
#[derive(Clone)]
pub struct RateLimiter {
    semaphore: Arc<Semaphore>,
    requests_per_second: u32,
    refill_interval: Duration,
}

impl RateLimiter {
    /// Create new rate limiter
    /// 
    /// # Arguments
    /// * `requests_per_second` - Maximum requests allowed per second
    pub fn new(requests_per_second: u32) -> Self {
        Self {
            semaphore: Arc::new(Semaphore::new(requests_per_second as usize)),
            requests_per_second,
            refill_interval: Duration::from_millis(1000 / requests_per_second as u64),
        }
    }
    
    /// Acquire permission to make a request (async, will wait if needed)
    pub async fn acquire(&self) -> RateLimitPermit {
        let permit = self.semaphore.clone().acquire_owned().await
            .expect("Semaphore should never be closed");
        
        // Schedule permit release after refill interval
        let semaphore = self.semaphore.clone();
        let refill_interval = self.refill_interval;
        
        tokio::spawn(async move {
            sleep(refill_interval).await;
            drop(permit); // Release permit back to pool
        });
        
        RateLimitPermit {
            _acquired_at: Instant::now(),
        }
    }
    
    /// Try to acquire permission immediately (non-blocking)
    pub fn try_acquire(&self) -> Option<RateLimitPermit> {
        let permit = self.semaphore.clone().try_acquire_owned().ok()?;
        
        let semaphore = self.semaphore.clone();
        let refill_interval = self.refill_interval;
        
        tokio::spawn(async move {
            sleep(refill_interval).await;
            drop(permit);
        });
        
        Some(RateLimitPermit {
            _acquired_at: Instant::now(),
        })
    }
    
    /// Get current available permits
    pub fn available_permits(&self) -> usize {
        self.semaphore.available_permits()
    }
}

/// RAII guard for rate limit permit
pub struct RateLimitPermit {
    _acquired_at: Instant,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_rate_limiter_basic() {
        let limiter = RateLimiter::new(10); // 10 req/sec
        
        // Should immediately acquire 10 permits
        for _ in 0..10 {
            limiter.acquire().await;
        }
        
        assert_eq!(limiter.available_permits(), 0);
    }

    #[tokio::test]
    async fn test_rate_limiter_refill() {
        let limiter = RateLimiter::new(10);
        
        // Exhaust permits
        for _ in 0..10 {
            limiter.acquire().await;
        }
        
        // Wait for refill
        sleep(Duration::from_millis(200)).await;
        
        // Should have some permits back
        assert!(limiter.available_permits() > 0);
    }

    #[tokio::test]
    async fn test_try_acquire() {
        let limiter = RateLimiter::new(5);
        
        // Should succeed 5 times
        for _ in 0..5 {
            assert!(limiter.try_acquire().is_some());
        }
        
        // Should fail on 6th attempt
        assert!(limiter.try_acquire().is_none());
    }
}
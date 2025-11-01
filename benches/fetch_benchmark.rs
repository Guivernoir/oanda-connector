//! Benchmark for OANDA connector performance
//! 
//! Run with: cargo bench

use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use oanda_connector::{OandaClient, OandaConfig, Granularity};
use tokio::runtime::Runtime;

fn create_client() -> OandaClient {
    let config = OandaConfig::from_env()
        .expect("Set OANDA_API_KEY and OANDA_ACCOUNT_ID for benchmarks");
    OandaClient::new(config).expect("Failed to create client")
}

fn benchmark_get_price(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let client = create_client();
    
    c.bench_function("get_current_price", |b| {
        b.to_async(&rt).iter(|| async {
            let result = client.get_current_price("EUR_USD").await;
            black_box(result)
        });
    });
}

fn benchmark_get_multiple_prices(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let client = create_client();
    
    let instruments = vec![
        "EUR_USD".to_string(),
        "GBP_USD".to_string(),
        "USD_JPY".to_string(),
    ];
    
    c.bench_function("get_multiple_prices", |b| {
        b.to_async(&rt).iter(|| async {
            let result = client.get_current_prices(&instruments).await;
            black_box(result)
        });
    });
}

fn benchmark_get_candles(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let client = create_client();
    
    let mut group = c.benchmark_group("get_candles");
    
    for count in [10, 50, 100, 500].iter() {
        group.bench_with_input(
            BenchmarkId::from_parameter(count),
            count,
            |b, &count| {
                b.to_async(&rt).iter(|| async {
                    let result = client.get_candles("EUR_USD", Granularity::M5, count).await;
                    black_box(result)
                });
            },
        );
    }
    
    group.finish();
}

fn benchmark_rate_limiter(c: &mut Criterion) {
    use oanda_connector::rate_limiter::RateLimiter;
    let rt = Runtime::new().unwrap();
    
    c.bench_function("rate_limiter_acquire", |b| {
        let limiter = RateLimiter::new(100);
        
        b.to_async(&rt).iter(|| async {
            let permit = limiter.acquire().await;
            black_box(permit)
        });
    });
}

criterion_group!(
    benches,
    benchmark_get_price,
    benchmark_get_multiple_prices,
    benchmark_get_candles,
    benchmark_rate_limiter
);
criterion_main!(benches);
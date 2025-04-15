use criterion::{black_box, criterion_group, criterion_main, Criterion};
use llama_headers_rs::{get_header, get_headers, Config};
use llama_headers_rs::user_agent::UserAgent;

fn single_header_benchmark(c: &mut Criterion) {
    c.bench_function("generate single header", |b| {
        b.iter(|| {
            let url = black_box("https://example.com");
            get_header(url, None).unwrap()
        })
    });
}

fn single_header_with_config_benchmark(c: &mut Criterion) {
    c.bench_function("generate single header with config", |b| {
        b.iter(|| {
            let url = black_box("https://example.com");
            let config = Config::new()
                .with_language("de-DE")
                .with_mobile(true);
            get_header(url, Some(config)).unwrap()
        })
    });
}

fn multiple_headers_benchmark(c: &mut Criterion) {
    c.bench_function("generate 10 headers", |b| {
        b.iter(|| {
            let url = black_box("https://example.com");
            get_headers(url, 10, None).unwrap()
        })
    });
}

fn user_agent_parsing_benchmark(c: &mut Criterion) {
    c.bench_function("parse user agent", |b| {
        b.iter(|| {
            let ua_string = black_box("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/98.0.4758.102 Safari/537.36");
            UserAgent::parse(ua_string).unwrap()
        })
    });
}

fn user_agent_random_generation_benchmark(c: &mut Criterion) {
    c.bench_function("generate random user agent", |b| {
        b.iter(|| {
            UserAgent::get_random_user_agent(black_box(false)).unwrap()
        })
    });
    
    c.bench_function("generate random mobile user agent", |b| {
        b.iter(|| {
            UserAgent::get_random_user_agent(black_box(true)).unwrap()
        })
    });
}

criterion_group!(
    benches,
    single_header_benchmark,
    single_header_with_config_benchmark,
    multiple_headers_benchmark,
    user_agent_parsing_benchmark,
    user_agent_random_generation_benchmark
);
criterion_main!(benches); 
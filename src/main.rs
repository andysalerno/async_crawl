//! Crate-level documentation in progress.

#![forbid(unsafe_code, rust_2018_idioms)]
#![deny(
    missing_debug_implementations,
    nonstandard_style,
    trivial_casts,
    trivial_numeric_casts
)]
#![warn(
    missing_docs,
    missing_doc_code_examples,
    unreachable_pub,
    future_incompatible
)]

mod crawler;

use crawler::Crawler;
use async_std::task;

fn main() {
    println!("Hello, world!");

    task::block_on(async {
        let (s, r) = async_channel::unbounded();
        let crawler = Crawler::new(s.clone());
        crawler.handle_dir("/home/andy".into()).await;

        while let Ok(joiner) = r.recv().await {
            joiner.await;
        }
    });
}

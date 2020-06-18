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

use async_std::task;
use crawler::Crawler;

fn main() {
    println!("Hello, world!");

    task::block_on(async {
        let (s, r) = async_channel::unbounded();

        let s_clone = s.clone();

        s.send(async_std::task::spawn(async move {
            // let crawler = Crawler::new(matcher, printer, buf_pool);
            // crawler.handle_file(&dir_child).await;
            let crawler = Crawler::new(s_clone);
            crawler.handle_dir("/home/andy".into()).await;
        }))
        .await;

        while let Ok(joiner) = r.try_recv() {
            joiner.await;
        }
    });
}

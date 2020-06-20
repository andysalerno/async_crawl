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

mod recursive_crawler;
mod scaled_crawler;

use async_std::sync::Arc;
use async_std::task;
use recursive_crawler::Crawler;
use scaled_crawler::{DirWork, Worker};
use std::sync::atomic::AtomicUsize;

fn main() {
    println!("Hello, world!");

    run_scaled();
}

fn run_scaled() {
    task::block_on(async {
        let idle_count = Arc::new(AtomicUsize::new(0));
        let stack = scaled_crawler::make_stack();
        stack.lock().await.push(DirWork::Path("/home/andy/".into()));

        let worker1 = Worker::new(stack.clone(), idle_count.clone());
        let worker2 = Worker::new(stack.clone(), idle_count.clone());
        let worker3 = Worker::new(stack.clone(), idle_count.clone());
        let worker4 = Worker::new(stack.clone(), idle_count.clone());

        let task1 = task::spawn(async {
            worker1.run().await;
        });

        let task2 = task::spawn(async {
            worker2.run().await;
        });

        let task3 = task::spawn(async {
            worker3.run().await;
        });

        let task4 = task::spawn(async {
            worker4.run().await;
        });

        task1.await;
        task2.await;
        task3.await;
        task4.await;
    });
}

fn run_recursive() {
    task::block_on(async {
        let (s, r) = async_channel::unbounded();

        let s_clone = s.clone();

        s.send(async_std::task::spawn(async move {
            let crawler = Crawler::new(s_clone);
            crawler.handle_dir("/home/andy".into()).await;
        }))
        .await;

        while let Ok(joiner) = r.try_recv() {
            joiner.await;
        }
    });
}

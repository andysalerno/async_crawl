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
mod scaled_crawler_async;
mod scaled_crawler_threaded;

fn main() {
    println!("Hello, world!");

    // run_scaled_async();
    run_scaled_threaded();
}

fn run_scaled_threaded() {
        use std::sync::Arc;
        use std::thread;
        use scaled_crawler_threaded::{DirWork, Worker};
        use std::sync::atomic::AtomicUsize;

        let idle_count = Arc::new(AtomicUsize::new(0));
        let stack = scaled_crawler_threaded::make_stack();
        stack.lock().unwrap().push(DirWork::Path("/home/andy/".into()));

        let worker1 = Worker::new(stack.clone(), idle_count.clone());
        let worker2 = Worker::new(stack.clone(), idle_count.clone());
        let worker3 = Worker::new(stack.clone(), idle_count.clone());
        let worker4 = Worker::new(stack.clone(), idle_count.clone());

        let task1 = thread::spawn( || worker1.run());
        let task2 = thread::spawn( || worker2.run());
        let task3 = thread::spawn( || worker3.run());
        let task4 = thread::spawn( || worker4.run());

        task1.join().unwrap();
        task2.join().unwrap();
        task3.join().unwrap();
        task4.join().unwrap();
}

fn run_scaled_async() {
    use async_std::sync::Arc;
    use async_std::task;
    use scaled_crawler_async::{DirWork, Worker};
    use std::sync::atomic::AtomicUsize;

    task::block_on(async {
        let idle_count = Arc::new(AtomicUsize::new(0));
        let stack = scaled_crawler_async::make_stack();
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
    use async_std::task;
    use recursive_crawler::Crawler;

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

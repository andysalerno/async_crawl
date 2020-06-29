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

mod recursive_crawler_async;
mod scaled_crawler_async;
mod scaled_crawler_single_thread;
mod scaled_crawler_threaded;

fn main() {
    println!("Hello, world!");

    let thread_count: usize = std::env::args()
        .nth(1)
        .unwrap_or("1".to_owned())
        .parse()
        .unwrap();

    println!("Running with {} threads/tasks.", thread_count);

    // run_scaled_async(thread_count);

    if thread_count == 1 {
        run_scaled_single_threaded();
    } else {
        run_scaled_threaded(thread_count);
    }
}

fn run_scaled_threaded(thread_count: usize) {
    use scaled_crawler_threaded::{DirWork, Worker};
    use std::sync::atomic::AtomicUsize;
    use std::sync::Arc;
    use std::thread;

    let active_count = Arc::new(AtomicUsize::new(thread_count));
    let stack = scaled_crawler_threaded::make_stack();
    stack
        .lock()
        .unwrap()
        .push(DirWork::Path("/home/andy/".into()));

    let mut handles = vec![];

    for _ in 0..thread_count {
        let worker = Worker::new(stack.clone(), active_count.clone());

        let handle = thread::spawn(|| worker.run());

        handles.push(handle);
    }

    handles.into_iter().for_each(|h| h.join().unwrap());
}

fn run_scaled_single_threaded() {
    use scaled_crawler_single_thread::Worker;
    let worker = Worker::new("/home/andy/".into());
    worker.run();
}

fn run_scaled_async(task_count: usize) {
    use async_std::sync::Arc;
    use async_std::task;
    use scaled_crawler_async::{DirWork, Worker};
    use std::sync::atomic::AtomicUsize;

    task::block_on(async {
        let mut handles = vec![];

        let idle_count = Arc::new(AtomicUsize::new(0));
        let stack = scaled_crawler_async::make_stack();
        stack.lock().await.push(DirWork::Path("/home/andy/".into()));

        for _ in 0..task_count {
            let worker = Worker::new(stack.clone(), idle_count.clone());

            let task = task::spawn(async {
                worker.run().await;
            });

            handles.push(task);
        }

        let mut handles = handles.into_iter();

        while let Some(handle) = handles.next() {
            handle.await;
        }
    });
}

fn run_recursive() {
    use async_std::task;
    use recursive_crawler_async::Crawler;

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

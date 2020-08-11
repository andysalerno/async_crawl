//! An async implementation of a Crawler that spawns a fixed number of concurrent tasks, like threads, as it crawls.

use crate::dir_work::r#async::AsyncDirWork;
use crate::AsyncCrawler;
use async_std::stream::StreamExt;
use async_std::sync::{Arc, Mutex};
use async_trait::async_trait;
use std::sync::atomic::AtomicUsize;
use std::sync::atomic::Ordering;

/// Creates the AsyncCrawler with the provided count of concurrent tasks.
pub fn make_crawler(task_count: usize) -> impl AsyncCrawler {
    WorkerManager { task_count }
}

struct Worker<F: Fn(AsyncDirWork)> {
    stack: SharedStack<AsyncDirWork>,
    active_count: Arc<AtomicUsize>,
    f: F,
}

type SharedStack<T> = Arc<Mutex<Vec<T>>>;
fn make_stack() -> SharedStack<AsyncDirWork> {
    Arc::new(Mutex::new(vec![]))
}

struct WorkerManager {
    task_count: usize,
}

#[async_trait]
impl AsyncCrawler for WorkerManager {
    async fn crawl<F: Fn(AsyncDirWork) + Clone + Send + Sync + 'static>(
        self,
        path: &std::path::Path,
        f: F,
    ) {
        use async_std::task;

        let mut handles = vec![];

        let active_count = Arc::new(AtomicUsize::new(self.task_count));
        let stack = make_stack();
        stack.lock().await.push(AsyncDirWork::Path(path.into()));

        for _ in 0..self.task_count {
            let worker = Worker::new(stack.clone(), active_count.clone(), f.clone());

            let task = task::spawn(async {
                worker.run().await;
            });

            handles.push(task);
        }

        let mut handles = handles.into_iter();

        while let Some(handle) = handles.next() {
            handle.await;
        }
    }
}

// TODO: try using all DirEntry instead of Path, may have better perf
impl<F: Fn(AsyncDirWork)> Worker<F> {
    fn new(stack: SharedStack<AsyncDirWork>, active_count: Arc<AtomicUsize>, f: F) -> Self {
        Self {
            stack,
            active_count,
            f,
        }
    }

    async fn run(self) {
        let mut is_active = true;

        loop {
            let work = self.stack.lock().await.pop();

            if work.is_none() {
                let all_idle = if !is_active {
                    self.active_count.load(Ordering::SeqCst) == 0
                } else {
                    self.active_count.fetch_sub(1, Ordering::SeqCst) == 1
                };

                is_active = false;

                if all_idle {
                    return;
                }

                async_std::task::yield_now().await;
                continue;
            } else if !is_active {
                self.active_count.fetch_add(1, Ordering::SeqCst);
                is_active = true;
            }

            self.run_one(work.unwrap()).await;
        }
    }

    async fn run_one(&self, work: AsyncDirWork) {
        if work.is_file().await {
            (self.f)(work);
            return;
        }

        if !work.is_dir().await {
            return;
        }

        let dir_children = async_std::fs::read_dir(work.path()).await;
        let mut dir_children = if let Ok(dir_children) = dir_children {
            dir_children
        } else {
            return;
        };

        while let Some(dir_child) = dir_children.next().await {
            // TODO: try locking once around the loop?  What does BurntSushi know that I don't...
            self.stack
                .lock()
                .await
                .push(AsyncDirWork::Entry(dir_child.unwrap()));
        }
    }
}

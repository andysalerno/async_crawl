//! An async implementation of a Crawler that recursively spawns more tasks as it crawls.

use crate::dir_work::r#async::AsyncDirWork;
use crate::AsyncCrawler;
use async_channel::Sender;
use async_std::path::PathBuf;
use async_std::stream::StreamExt;
use std::future::Future;
use std::pin::Pin;

pub(crate) fn make_crawler() -> impl AsyncCrawler {
    RecursiveCrawlerManager
}

struct RecursiveCrawlerManager;

struct RecursiveCrawler<F: Fn(AsyncDirWork)> {
    wait_pool: Sender<async_std::task::JoinHandle<()>>,
    f: F,
}

impl AsyncCrawler for RecursiveCrawlerManager {
    fn crawl<F: Fn(AsyncDirWork) + Clone + Send + 'static>(self, path: &std::path::Path, f: F) {
        use async_std::task;

        let path: async_std::path::PathBuf = path.into();

        task::block_on(async {
            let (s, r) = async_channel::unbounded();

            let s_clone = s.clone();

            s.send(async_std::task::spawn(async move {
                let crawler = RecursiveCrawler::new(s_clone, f.clone());
                crawler.handle_work(path).await;
            }))
            .await
            .expect("task failed.");

            while let Ok(x) = r.try_recv() {
                x.await;
            }
        });
    }
}

impl<F: Fn(AsyncDirWork) + Clone + Send + 'static> RecursiveCrawler<F> {
    fn new(wait_pool: Sender<async_std::task::JoinHandle<()>>, f: F) -> Self {
        Self { wait_pool, f }
    }

    fn handle_work(self, path: PathBuf) -> Pin<Box<dyn Future<Output = ()> + Send>> {
        Box::pin(async move {
            let is_sym = path
                .symlink_metadata()
                .await
                .unwrap()
                .file_type()
                .is_symlink();

            if is_sym || !path.is_dir().await {
                (self.f)(AsyncDirWork::Path(path));
            } else {
                self.handle_dir(path).await
            }
        })
    }

    fn handle_dir(self, path: PathBuf) -> Pin<Box<dyn Future<Output = ()> + Send>> {
        Box::pin(async move {
            let mut dir_children = {
                if let Ok(children) = async_std::fs::read_dir(path).await {
                    children
                } else {
                    return;
                }
            };

            while let Some(dir_child) = dir_children.next().await {
                let dir_child = dir_child.expect("Failed to make dir child.").path();

                let f = self.f.clone();

                let pool_copy = self.wait_pool.clone();

                self.wait_pool
                    .send(async_std::task::spawn(async move {
                        // let crawler = Crawler::new(matcher, printer, buf_pool);
                        // crawler.handle_file(&dir_child).await;
                        let crawler = RecursiveCrawler::new(pool_copy, f.clone());
                        crawler.handle_work(dir_child).await;
                    }))
                    .await
                    .expect("failed sending task to pool.");
            }
        })
    }
}

// rg has:
// One global stack
// All workers have an Arc to the stack, which they push/pop from

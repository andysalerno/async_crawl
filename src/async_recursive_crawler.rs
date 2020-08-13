//! An async implementation of a Crawler that recursively spawns more tasks as it crawls.

use crate::dir_work::r#async::AsyncDirWork;
use crate::AsyncCrawler;
use async_channel::Sender;
use async_std::path::PathBuf;
use async_std::stream::StreamExt;
use async_trait::async_trait;
use std::future::Future;
use std::pin::Pin;

/// Creates the recursive AsyncCrawler.
pub fn make_crawler() -> impl AsyncCrawler {
    RecursiveCrawlerManager
}

struct RecursiveCrawlerManager;

struct RecursiveCrawler<F, Fut, T>
where
    Fut: Future<Output = T> + 'static,
    F: Send + Sync + Clone + 'static + FnOnce(AsyncDirWork) -> Fut,
{
    wait_pool: Sender<async_std::task::JoinHandle<()>>,
    f: F,
}

#[async_trait]
impl AsyncCrawler for RecursiveCrawlerManager {
    async fn crawl<F, Fut, T>(self, path: &std::path::Path, f: F)
    where
        T: 'static,
        Fut: Future<Output = T> + 'static + Send,
        F: Send + Sync + Clone + 'static + FnOnce(AsyncDirWork) -> Fut,
    {
        use async_channel::TryRecvError;

        let path: async_std::path::PathBuf = path.into();

        // let (s, r) = async_channel::bounded(128);
        let (s, r) = async_channel::unbounded();

        let s_clone = s.clone();

        s.send(async_std::task::spawn(async move {
            let crawler = RecursiveCrawler::new(s_clone, f);
            crawler.handle_work(path).await;
        }))
        .await
        .expect("task failed.");

        drop(s);

        loop {
            match r.try_recv() {
                Ok(x) => x.await,
                Err(TryRecvError::Empty) => async_std::task::yield_now().await,
                _ => break,
            }
        }
    }
}

impl<F, Fut, T> RecursiveCrawler<F, Fut, T>
where
    T: 'static,
    Fut: Future<Output = T> + 'static + Send,
    F: Send + Sync + Clone + 'static + FnOnce(AsyncDirWork) -> Fut,
{
    fn new(wait_pool: Sender<async_std::task::JoinHandle<()>>, f: F) -> Self {
        Self { wait_pool, f }
    }

    fn handle_work(self, path: PathBuf) -> Pin<Box<dyn Future<Output = ()> + Send>> {
        Box::pin(async move {
            // let is_sym = path
            //     .symlink_metadata()
            //     .await
            //     .unwrap()
            //     .file_type()
            //     .is_symlink();

            // if is_sym {
            //     return;
            // }

            if path.is_file().await {
                (self.f)(AsyncDirWork::Path(path)).await;
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
                        let crawler = RecursiveCrawler::new(pool_copy, f);
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

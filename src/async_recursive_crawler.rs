use crate::Crawler;
use async_channel::Sender;
use async_std::path::{Path, PathBuf};
use async_std::stream::StreamExt;
use std::future::Future;
use std::pin::Pin;

pub(crate) fn make_crawler() -> impl Crawler {
    RecursiveCrawlerManager
}

struct RecursiveCrawlerManager;

struct RecursiveCrawler {
    wait_pool: Sender<async_std::task::JoinHandle<()>>,
}

impl Crawler for RecursiveCrawlerManager {
    fn crawl<F: Fn()>(self, path: &std::path::Path, f: F) {
        use async_std::task;

        let path: async_std::path::PathBuf = path.into();

        task::block_on(async {
            let (s, r) = async_channel::unbounded();

            let s_clone = s.clone();

            s.send(async_std::task::spawn(async move {
                let crawler = RecursiveCrawler::new(s_clone);
                crawler.handle_dir(path).await;
            }))
            .await
            .expect("task failed.");

            while let Ok(joiner) = r.try_recv() {
                joiner.await;
            }
        });
    }
}

impl RecursiveCrawler {
    fn new(wait_pool: Sender<async_std::task::JoinHandle<()>>) -> Self {
        Self { wait_pool }
    }

    async fn handle_file(&self, path: &Path) {
        // println!("{:?}", path);
    }

    fn handle_dir(self, path: PathBuf) -> Pin<Box<dyn Future<Output = ()> + Send>> {
        let result = Box::pin(async move {
            let is_sym = path
                .symlink_metadata()
                .await
                .unwrap()
                .file_type()
                .is_symlink();

            if is_sym || !path.is_dir().await {
                return self.handle_file(&path).await;
            }

            let mut dir_children = {
                if let Ok(children) = async_std::fs::read_dir(path).await {
                    children
                } else {
                    return;
                }
            };

            while let Some(dir_child) = dir_children.next().await {
                let dir_child = dir_child.expect("Failed to make dir child.").path();

                let pool_copy = self.wait_pool.clone();

                self.wait_pool
                    .send(async_std::task::spawn(async move {
                        // let crawler = Crawler::new(matcher, printer, buf_pool);
                        // crawler.handle_file(&dir_child).await;
                        let crawler = RecursiveCrawler::new(pool_copy);
                        crawler.handle_dir(dir_child).await;
                    }))
                    .await
                    .expect("failed sending task to pool.");
            }
        });

        return result;
    }
}

// rg has:
// One global stack
// All workers have an Arc to the stack, which they push/pop from

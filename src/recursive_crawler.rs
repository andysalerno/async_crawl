use async_channel::Sender;
use async_std::path::{Path, PathBuf};
use async_std::stream::StreamExt;
use std::future::Future;
use std::pin::Pin;

pub(crate) struct Crawler {
    wait_pool: Sender<async_std::task::JoinHandle<()>>,
}

impl Crawler {
    pub fn new(wait_pool: Sender<async_std::task::JoinHandle<()>>) -> Self {
        Self { wait_pool }
    }

    async fn handle_file(&self, path: &Path) {
        // println!("{:?}", path);
    }

    pub fn handle_dir(self, path: PathBuf) -> Pin<Box<dyn Future<Output = ()> + Send>> {
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
                        let crawler = Crawler::new(pool_copy);
                        crawler.handle_dir(dir_child).await;
                    }))
                    .await;
            }
        });

        return result;
    }
}

// rg has:
// One global stack
// All workers have an Arc to the stack, which they push/pop from

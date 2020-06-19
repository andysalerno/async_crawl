use async_std::path::PathBuf;
use async_std::stream::StreamExt;
use async_std::sync::{Arc, Mutex};

pub(crate) struct Worker {
    stack: SharedStack<PathBuf>,
}

pub(crate) type SharedStack<T> = Arc<Mutex<Vec<T>>>;
pub(crate) fn make_stack() -> SharedStack<PathBuf> {
    Arc::new(Mutex::new(vec![]))
}

struct WorkerManager {
    stack: SharedStack<PathBuf>,
    workers: Vec<Worker>,
}

// TODO: try using all DirEntry instead of Path, may have better perf
impl Worker {
    pub fn new(stack: SharedStack<PathBuf>) -> Self {
        Self { stack }
    }

    pub async fn run(self) {
        loop {
            let work = self.stack.lock().await.pop();

            if work.is_none() {
                async_std::task::yield_now().await;
                continue;
            }

            self.run_one(work.unwrap()).await;
        }
    }

    async fn work_handler(work: PathBuf) {
        println!("{}", work.to_str().unwrap());
    }

    async fn run_one(&self, work: PathBuf) {
        let is_symlink = work
            .symlink_metadata()
            .await
            .unwrap()
            .file_type()
            .is_symlink();

        if is_symlink || !work.is_dir().await {
            Self::work_handler(work).await;
            return;
        }

        // it's a dir, so we must read it and push its children as new work
        let mut dir_children = async_std::fs::read_dir(work).await.unwrap();

        while let Some(dir_child) = dir_children.next().await {
            // TODO: try locking once around the loop?  What does BurntSushi know that I don't...
            self.stack.lock().await.push(dir_child.unwrap().path());
        }
    }
}

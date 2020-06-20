use std::fs::DirEntry;
use std::path::PathBuf;
use std::sync::atomic::AtomicUsize;
use std::sync::atomic::Ordering;
use std::sync::{Arc, Mutex};

pub(crate) enum DirWork {
    Entry(DirEntry),
    Path(PathBuf),
}

impl DirWork {
    fn to_path(self) -> std::path::PathBuf {
        match self {
            DirWork::Entry(e) => e.path(),
            DirWork::Path(path) => path,
        }
    }

    fn is_dir(&self) -> bool {
        match self {
            DirWork::Entry(e) => e.metadata().unwrap().is_dir(),
            DirWork::Path(path) => path.is_dir(),
        }
    }

    fn is_symlink(&self) -> bool {
        match self {
            DirWork::Entry(e) => e.file_type().unwrap().is_symlink(),
            DirWork::Path(path) => path.symlink_metadata().unwrap().file_type().is_symlink(),
        }
    }

    fn path(self) -> PathBuf {
        match self {
            DirWork::Entry(e) => e.path(),
            DirWork::Path(path) => path,
        }
    }
}

pub(crate) struct Worker {
    stack: SharedStack<DirWork>,
    idle_count: Arc<AtomicUsize>,
}

pub(crate) type SharedStack<T> = Arc<Mutex<Vec<T>>>;

pub(crate) fn make_stack() -> SharedStack<DirWork> {
    Arc::new(Mutex::new(vec![]))
}

struct WorkerManager {
    stack: SharedStack<DirWork>,
    workers: Vec<Worker>,
}

// TODO: try using all DirEntry instead of Path, may have better perf
impl Worker {
    pub fn new(stack: SharedStack<DirWork>, idle_count: Arc<AtomicUsize>) -> Self {
        Self { stack, idle_count }
    }

    pub fn run(self) {
        let mut is_idle = false;

        loop {
            let work = self.stack.lock().unwrap().pop();

            if work.is_none() {
                let total_idle = if !is_idle {
                    // We just became idle -- need to update the idlers count.
                    self.idle_count.fetch_add(1, Ordering::SeqCst) + 1
                } else {
                    // We were already idle, so let's see what the current count is.
                    self.idle_count.load(Ordering::SeqCst)
                };

                if total_idle >= 4 {
                    return;
                }

                is_idle = true;

                std::thread::yield_now();
                continue;
            } else if is_idle {
                // We were idle, but no longer --
                // update the global count
                self.idle_count.fetch_sub(1, Ordering::SeqCst);
                is_idle = false;
            }

            self.run_one(work.unwrap());
        }
    }

    fn work_handler(work: DirWork) {
        println!("{}", work.to_path().display());
        // async_std::task::sleep(std::time::Duration::from_millis(100)).await;
    }

    fn run_one(&self, work: DirWork) {
        let is_symlink = work.is_symlink();

        if is_symlink || !work.is_dir() {
            Self::work_handler(work);
            return;
        }

        // it's a dir, so we must read it and push its children as new work
        let mut dir_children = std::fs::read_dir(work.path()).unwrap();

        while let Some(dir_child) = dir_children.next() {
            // TODO: try locking once around the loop?  What does BurntSushi know that I don't...
            self.stack
                .lock()
                .unwrap()
                .push(DirWork::Entry(dir_child.unwrap()));
        }
    }
}

use crate::Crawler;
use std::fs::DirEntry;
use std::path::PathBuf;
use std::sync::atomic::AtomicUsize;
use std::sync::atomic::Ordering;
use std::sync::{Arc, Mutex};

pub(crate) fn make_crawler(thread_count: usize) -> impl Crawler {
    WorkerManager { thread_count }
}

enum DirWork {
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

    fn is_file(&self) -> bool {
        match self {
            DirWork::Entry(e) => e.metadata().unwrap().is_file(),
            DirWork::Path(path) => path.is_file(),
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

impl Crawler for WorkerManager {
    fn crawl(self, path: &std::path::Path) {
        use std::thread;

        let active_count = Arc::new(AtomicUsize::new(self.thread_count));
        let stack = make_stack();
        stack.lock().unwrap().push(DirWork::Path(path.into()));

        let mut handles = vec![];

        for _ in 0..self.thread_count {
            let worker = Worker::new(stack.clone(), active_count.clone());
            let handle = thread::spawn(|| worker.run());
            handles.push(handle);
        }

        handles.into_iter().for_each(|h| h.join().unwrap());
    }
}

struct Worker {
    stack: SharedStack<DirWork>,
    active_count: Arc<AtomicUsize>,
}

type SharedStack<T> = Arc<Mutex<Vec<T>>>;

fn make_stack() -> SharedStack<DirWork> {
    Arc::new(Mutex::new(vec![]))
}

struct WorkerManager {
    thread_count: usize,
}

// TODO: try using all DirEntry instead of Path, may have better perf
impl Worker {
    fn new(stack: SharedStack<DirWork>, active_count: Arc<AtomicUsize>) -> Self {
        Self {
            stack,
            active_count,
        }
    }

    fn run(self) {
        let mut is_active = true;

        loop {
            let work = self.stack.lock().unwrap().pop();

            if work.is_none() {
                let all_idle = if !is_active {
                    // We're already idle, no need to update the count
                    self.active_count.load(Ordering::SeqCst) == 0
                } else {
                    // We just became idle -- need to update the idlers count.
                    self.active_count.fetch_sub(1, Ordering::SeqCst) == 1
                };

                is_active = false;

                if all_idle {
                    return;
                }

                std::thread::yield_now();
                continue;
            } else if !is_active {
                // We were idle, but no longer --
                // update the global count
                self.active_count.fetch_add(1, Ordering::SeqCst);
                is_active = true;
            }

            self.run_one(work.unwrap());
        }
    }

    fn work_handler(work: DirWork) {
        // println!("{}", work.to_path().display());
    }

    fn run_one(&self, work: DirWork) {
        if work.is_file() {
            Self::work_handler(work);
            return;
        }

        if !work.is_dir() {
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

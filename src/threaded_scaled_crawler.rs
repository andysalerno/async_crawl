use crate::dir_work::sync::DirWork;
use crate::Crawler;
use std::sync::atomic::AtomicUsize;
use std::sync::atomic::Ordering;
use std::sync::{Arc, Mutex};

pub(crate) fn make_crawler(thread_count: usize) -> impl Crawler {
    WorkerManager { thread_count }
}

impl Crawler for WorkerManager {
    fn crawl<F: Fn(DirWork) + Send + Clone + 'static>(self, path: &std::path::Path, f: F) {
        use std::thread;

        let active_count = Arc::new(AtomicUsize::new(self.thread_count));
        let stack = make_stack();
        stack.lock().unwrap().push(DirWork::Path(path.into()));

        let mut handles = vec![];

        for _ in 0..self.thread_count {
            let worker = Worker::new(stack.clone(), active_count.clone(), f);
            let handle = thread::spawn(|| worker.run());
            handles.push(handle);
        }

        handles.into_iter().for_each(|h| h.join().unwrap());
    }
}

struct Worker<F: Fn(DirWork)> {
    stack: SharedStack<DirWork>,
    active_count: Arc<AtomicUsize>,
    f: F,
}

type SharedStack<T> = Arc<Mutex<Vec<T>>>;

fn make_stack() -> SharedStack<DirWork> {
    Arc::new(Mutex::new(vec![]))
}

struct WorkerManager {
    thread_count: usize,
}

// TODO: try using all DirEntry instead of Path, may have better perf
impl<F: Fn(DirWork)> Worker<F> {
    fn new(stack: SharedStack<DirWork>, active_count: Arc<AtomicUsize>, f: F) -> Self {
        Self {
            stack,
            active_count,
            f,
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

    fn run_one(&self, work: DirWork) {
        if work.is_file() {
            (self.f)(work);
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

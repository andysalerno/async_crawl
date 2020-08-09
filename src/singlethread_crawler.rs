//! An implementation of Crawler using a single thread.

use crate::dir_work::sync::DirWork;
use crate::Crawler;
use std::path::PathBuf;

pub(crate) fn make_crawler() -> impl Crawler {
    Worker::new()
}

impl Crawler for Worker {
    fn crawl<T: Fn(DirWork)>(self, path: &std::path::Path, f: T) {
        self.run(path.into(), f);
    }
}

#[derive(Default)]
struct Worker {
    stack: Vec<DirWork>,
}

// TODO: try using all DirEntry instead of Path, may have better perf
impl Worker {
    fn new() -> Self {
        Worker { stack: vec![] }
    }

    fn run<F: Fn(DirWork)>(mut self, path: PathBuf, f: F) {
        self.stack.push(DirWork::Path(path));

        while let Some(work) = self.stack.pop() {
            self.run_one(work, &f)
                .unwrap_or_else(|_| { /* Fail silently */ })
        }
    }

    fn run_one<F: Fn(DirWork)>(&mut self, work: DirWork, f: &F) -> Result<(), Box<std::io::Error>> {
        if work.is_file() {
            (f)(work);
            return Ok(());
        }

        if !work.is_dir() {
            return Ok(());
        }

        // it's a dir, so we must read it and push its children as new work
        let dir_children = std::fs::read_dir(work.into_pathbuf())?;

        for child in dir_children {
            if let Ok(dir_child) = child {
                self.stack.push(DirWork::Entry(dir_child));
            }
        }

        Ok(())
    }
}

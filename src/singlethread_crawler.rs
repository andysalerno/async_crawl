use crate::Crawler;
use std::fs::DirEntry;
use std::path::PathBuf;

pub(crate) fn make_crawler() -> impl Crawler {
    Worker::new()
}

impl Crawler for Worker {
    fn crawl<T: Fn() + Send + Clone + 'static>(self, path: &std::path::Path, f: T) {
        self.run(path.into(), f);
    }
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

#[derive(Default)]
struct Worker {
    stack: Vec<DirWork>,
}

// TODO: try using all DirEntry instead of Path, may have better perf
impl Worker {
    fn new() -> Self {
        Worker { stack: vec![] }
    }

    fn run<F: Fn() + Clone>(mut self, path: PathBuf, f: F) {
        self.stack.push(DirWork::Path(path));

        while let Some(work) = self.stack.pop() {
            self.run_one(work, f.clone());
        }
    }

    fn run_one<F: Fn()>(&mut self, work: DirWork, f: F) {
        if work.is_file() {
            (f)();
            return;
        }

        if !work.is_dir() {
            return;
        }

        // it's a dir, so we must read it and push its children as new work
        let mut dir_children = std::fs::read_dir(work.path()).unwrap();

        while let Some(dir_child) = dir_children.next() {
            self.stack.push(DirWork::Entry(dir_child.unwrap()));
        }
    }
}

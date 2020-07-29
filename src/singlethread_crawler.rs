use crate::Crawler;
use std::fs::DirEntry;
use std::path::PathBuf;

pub(crate) fn make_crawler() -> impl Crawler {
    Worker::new()
}

impl Crawler for Worker {
    fn crawl(self, path: &std::path::Path) {
        self.run(path.into());
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
pub(crate) struct Worker {
    stack: Vec<DirWork>,
}

// TODO: try using all DirEntry instead of Path, may have better perf
impl Worker {
    pub fn new() -> Self {
        let mut worker = Self::default();

        worker.stack = vec![];

        worker
    }

    fn run(mut self, path: PathBuf) {
        self.stack.push(DirWork::Path(path));

        let mut is_active = true;

        loop {
            let work = self.stack.pop();

            if work.is_none() {
                return;
            }

            self.run_one(work.unwrap());
        }
    }

    fn work_handler(work: DirWork) {
        // println!("{}", work.to_path().display());
    }

    fn run_one(&mut self, work: DirWork) {
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
            self.stack.push(DirWork::Entry(dir_child.unwrap()));
        }
    }
}

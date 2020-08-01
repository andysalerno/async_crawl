//! Crate-level documentation in progress.

#![forbid(unsafe_code, rust_2018_idioms)]
#![deny(
    missing_debug_implementations,
    nonstandard_style,
    trivial_casts,
    trivial_numeric_casts
)]
#![warn(
    missing_docs,
    missing_doc_code_examples,
    unreachable_pub,
    future_incompatible
)]

mod async_recursive_crawler;
mod async_scaled_crawler;
mod dir_work;
mod singlethread_crawler;
mod threaded_scaled_crawler;

use dir_work::r#async::AsyncDirWork;
use dir_work::sync::DirWork;
use std::io::{self, Write};
use std::path::Path;
use std::thread;

trait Crawler {
    fn crawl<F: Fn(DirWork) + Send + Clone + 'static>(self, path: &std::path::Path, f: F);
}

trait AsyncCrawler {
    fn crawl<F: Fn(AsyncDirWork) + Send + Clone + 'static>(self, path: &std::path::Path, f: F);
}

fn main() {
    let thread_count: usize = std::env::args()
        .nth(1)
        .unwrap_or("1".to_owned())
        .parse()
        .unwrap();

    let dir = std::env::args()
        .nth(2)
        .expect("Usage: ./bin thread_count target_dir");

    // let (tx, rx) = channel::bounded::<DirEntry>(100);

    // let stdout_thread = thread::spawn(move || {
    //     let mut stdout = io::BufWriter::new(io::stdout());
    //     for dent in rx {
    //         write_path(&mut stdout, dent.path());
    //     }
    // });

    let action = |work: DirWork| {
        let stdout = io::BufWriter::new(io::stdout());
        write_path(stdout, &work.into_pathbuf());
    };

    // let async_crawler = async_scaled_crawler::make_crawler(thread_count);
    // async_crawler.crawl(&std::path::PathBuf::from("/home/andy/"), action);

    // let threaded_crawler = threaded_scaled_crawler::make_crawler(thread_count);
    // threaded_crawler.crawl(&std::path::PathBuf::from("/home/andy/"), action);

    let singlethread_crawler = singlethread_crawler::make_crawler();
    singlethread_crawler.crawl(&std::path::PathBuf::from(dir), action);

    // let async_recursive_crawler = async_scaled_crawler::make_crawler(thread_count);
    // async_recursive_crawler.crawl(&std::path::PathBuf::from("/home/andy/"), action);
}

fn write_path<W: Write>(mut wtr: W, path: &Path) {
    use std::os::unix::ffi::OsStrExt;
    wtr.write(path.as_os_str().as_bytes()).unwrap();
    wtr.write(b"\n").unwrap();
}

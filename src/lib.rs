//! A crate providing the Crawler trait, which can walk a directory tree and trigger a closure on each item,
//! as well as several implementations of Crawler.

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

mod dir_work;
mod example;

pub mod async_recursive_crawler;
pub mod async_scaled_crawler;
pub mod singlethread_crawler;
pub mod threaded_scaled_crawler;

use dir_work::r#async::AsyncDirWork;
use dir_work::sync::DirWork;

/// A trait that describes a directory crawler,
/// which accepts a closure that will be triggered on each entry.
pub trait Crawler {
    /// Crawls the directory starting with the provided path as the root,
    /// invoking the closure on each entry.
    fn crawl<F: Fn(DirWork) + Send + Clone + 'static>(self, path: &std::path::Path, f: F);
}

/// A trait that describes a directory crawler,
/// which accepts a closure that will be triggered on each entry.
pub trait AsyncCrawler {
    /// Crawls the directory starting with the provided path as the root,
    /// invoking the closure on each entry.
    fn crawl<F: Fn(AsyncDirWork) + Send + Sync + Clone + 'static>(
        self,
        path: &std::path::Path,
        f: F,
    );
}

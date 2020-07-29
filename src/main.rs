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
mod singlethread_crawler;
mod threaded_scaled_crawler;

trait Crawler {
    fn crawl(self, path: &std::path::Path);
}

fn main() {
    let thread_count: usize = std::env::args()
        .nth(1)
        .unwrap_or("1".to_owned())
        .parse()
        .unwrap();

    let async_crawler = async_scaled_crawler::make_crawler(thread_count);
    async_crawler.crawl(&std::path::PathBuf::from("/home/andy/"));

    let threaded_crawler = threaded_scaled_crawler::make_crawler(thread_count);
    threaded_crawler.crawl(&std::path::PathBuf::from("/home/andy/"));

    let singlethread_crawler = singlethread_crawler::make_crawler();
    singlethread_crawler.crawl(&std::path::PathBuf::from("/home/andy/"));

    let async_recursive_crawler = async_scaled_crawler::make_crawler(thread_count);
    async_recursive_crawler.crawl(&std::path::PathBuf::from("/home/andy/"));

    println!("Running with {} threads/tasks.", thread_count);
}

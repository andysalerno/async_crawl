#![allow(dead_code)]

use crate::dir_work::r#async::AsyncDirWork;
use crate::dir_work::sync::DirWork;
use crate::{AsyncCrawler, Crawler};
use async_std::path::Path as AsyncPath;
use crossbeam_channel::bounded;
use std::io::{self, Write};
use std::path::Path;
use std::thread;

fn run() {
    let thread_count: usize = std::env::args()
        .nth(1)
        .unwrap_or("1".to_owned())
        .parse()
        .unwrap();

    let strategy = std::env::args().nth(2).unwrap_or("1".to_owned());

    let dir = std::env::args()
        .nth(3)
        .expect("Usage: ./bin thread_count strategy target_dir");

    match strategy.as_str() {
        "async" => {
            let (tx, rx) = bounded::<AsyncDirWork>(128);

            let stdout_thread = thread::spawn(move || {
                let mut stdout = io::BufWriter::new(io::stdout());
                for dent in rx {
                    write_path_async(&mut stdout, &dent.into_pathbuf());
                }
            });

            let async_action = move |work: AsyncDirWork| async move {
                tx.send(work).expect("send to printer");
            };

            if thread_count > 1 {
                let async_crawler = crate::async_scaled_crawler::make_crawler(thread_count);
                async_std::task::block_on(async {
                    async_crawler
                        .crawl(&std::path::PathBuf::from(dir), async_action)
                        .await;
                });
            } else {
                let async_recursive_crawler = crate::async_recursive_crawler::make_crawler();
                async_std::task::block_on(async {
                    async_recursive_crawler
                        .crawl(&std::path::PathBuf::from(dir), async_action)
                        .await;
                });
            }

            stdout_thread.join().expect("join stdout thread");
        }
        "sync" => {
            let (tx, rx) = bounded::<DirWork>(128);

            let stdout_thread = thread::spawn(move || {
                let mut stdout = io::BufWriter::new(io::stdout());
                for dent in rx {
                    write_path(&mut stdout, &dent.into_pathbuf());
                }
            });

            let action = move |work: DirWork| {
                tx.send(work).expect("send to printer");
            };

            if thread_count > 1 {
                let threaded_crawler = crate::threaded_scaled_crawler::make_crawler(thread_count);
                threaded_crawler.crawl(&std::path::PathBuf::from(dir), action);
            } else {
                let singlethread_crawler = crate::singlethread_crawler::make_crawler();
                singlethread_crawler.crawl(&std::path::PathBuf::from(dir), action);
            }

            stdout_thread.join().expect("join stdout thread");
        }
        _ => panic!("Options are 'async' or 'sync'"),
    }
}

fn write_path<W: Write>(mut wtr: W, path: &Path) {
    use std::os::unix::ffi::OsStrExt;
    wtr.write(path.as_os_str().as_bytes()).unwrap();
    wtr.write(b"\n").unwrap();
}

fn write_path_async<W: Write>(mut wtr: W, path: &AsyncPath) {
    use std::os::unix::ffi::OsStrExt;
    wtr.write(path.as_os_str().as_bytes()).unwrap();
    wtr.write(b"\n").unwrap();
}

pub(crate) mod sync {
    use std::fs::DirEntry;
    use std::path::{Path, PathBuf};

    pub(crate) enum DirWork {
        Entry(DirEntry),
        Path(PathBuf),
    }

    impl DirWork {
        pub(crate) fn into_pathbuf(self) -> std::path::PathBuf {
            match self {
                DirWork::Entry(e) => e.path(),
                DirWork::Path(path) => path,
            }
        }

        pub(crate) fn is_dir(&self) -> bool {
            match self {
                DirWork::Entry(e) => e.metadata().and_then(|m| Ok(m.is_dir())).unwrap_or(false),
                DirWork::Path(path) => path.is_dir(),
            }
        }

        pub(crate) fn is_file(&self) -> bool {
            match self {
                DirWork::Entry(e) => e.metadata().and_then(|m| Ok(m.is_file())).unwrap_or(false),
                DirWork::Path(path) => path.is_file(),
            }
        }

        pub(crate) fn is_symlink(&self) -> bool {
            match self {
                DirWork::Entry(e) => e.file_type().unwrap().is_symlink(),
                DirWork::Path(path) => path.symlink_metadata().unwrap().file_type().is_symlink(),
            }
        }

        // pub(crate) fn path(&self) -> &Path {
        //     match *self {
        //         DirWork::Entry(ref e) => &e.path(),
        //         DirWork::Path(ref path) => path,
        //     }
        // }
    }
}

pub(crate) mod r#async {
    use async_std::fs::DirEntry;
    use async_std::path::PathBuf;

    pub(crate) enum AsyncDirWork {
        Entry(DirEntry),
        Path(PathBuf),
    }

    impl AsyncDirWork {
        pub(crate) fn to_path(self) -> async_std::path::PathBuf {
            match self {
                Self::Entry(e) => e.path(),
                Self::Path(path) => path,
            }
        }

        pub(crate) async fn is_dir(&self) -> bool {
            match self {
                Self::Entry(e) => e.metadata().await.unwrap().is_dir(),
                Self::Path(path) => path.is_dir().await,
            }
        }

        pub(crate) async fn is_symlink(&self) -> bool {
            match self {
                Self::Entry(e) => e.file_type().await.unwrap().is_symlink(),
                Self::Path(path) => path
                    .symlink_metadata()
                    .await
                    .unwrap()
                    .file_type()
                    .is_symlink(),
            }
        }

        pub(crate) fn path(self) -> PathBuf {
            match self {
                Self::Entry(e) => e.path(),
                Self::Path(path) => path,
            }
        }
    }
}

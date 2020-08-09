pub(crate) mod sync {
    use std::fs::DirEntry;
    use std::path::PathBuf;

    #[derive(Debug)]
    pub enum DirWork {
        Entry(DirEntry),
        Path(PathBuf),
    }

    impl DirWork {
        pub fn into_pathbuf(self) -> std::path::PathBuf {
            match self {
                DirWork::Entry(e) => e.path(),
                DirWork::Path(path) => path,
            }
        }

        pub fn is_dir(&self) -> bool {
            match self {
                DirWork::Entry(e) => e.metadata().and_then(|m| Ok(m.is_dir())).unwrap_or(false),
                DirWork::Path(path) => path.is_dir(),
            }
        }

        pub fn is_file(&self) -> bool {
            match self {
                DirWork::Entry(e) => e.metadata().and_then(|m| Ok(m.is_file())).unwrap_or(false),
                DirWork::Path(path) => path.is_file(),
            }
        }

        #[allow(dead_code)]
        pub fn is_symlink(&self) -> bool {
            match self {
                DirWork::Entry(e) => e.file_type().unwrap().is_symlink(),
                DirWork::Path(path) => path.symlink_metadata().unwrap().file_type().is_symlink(),
            }
        }
    }
}

pub(crate) mod r#async {
    use async_std::fs::DirEntry;
    use async_std::path::PathBuf;

    #[derive(Debug)]
    pub enum AsyncDirWork {
        Entry(DirEntry),
        Path(PathBuf),
    }

    impl AsyncDirWork {
        pub fn into_pathbuf(self) -> async_std::path::PathBuf {
            match self {
                Self::Entry(e) => e.path(),
                Self::Path(path) => path,
            }
        }

        pub async fn is_dir(&self) -> bool {
            match self {
                Self::Entry(e) => e
                    .metadata()
                    .await
                    .and_then(|m| Ok(m.is_dir()))
                    .unwrap_or(false),
                Self::Path(path) => path.is_dir().await,
            }
        }

        pub async fn is_file(&self) -> bool {
            match self {
                Self::Entry(e) => e
                    .metadata()
                    .await
                    .and_then(|m| Ok(m.is_file()))
                    .unwrap_or(false),
                Self::Path(path) => path.is_file().await,
            }
        }

        pub fn path(self) -> PathBuf {
            match self {
                Self::Entry(e) => e.path(),
                Self::Path(path) => path,
            }
        }
    }
}

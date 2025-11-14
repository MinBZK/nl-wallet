use std::fmt::Display;
use std::fs::File;
use std::io::Read;
use std::io::Seek;
use std::io::Write;
use std::os::fd::AsFd;
use std::path::Path;
use std::path::PathBuf;

use nutype::nutype;
use rustix::io::Errno;
use tokio::task::JoinError;

use utils::path::prefix_local_path;

#[nutype(derive(Debug, Clone, TryFrom, Into, AsRef, PartialEq, Deserialize), validate(with=PublishDir::validate, error=PublishDirError))]
pub struct PublishDir(PathBuf);

impl Display for PublishDir {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.as_ref().display().fmt(f)
    }
}

#[derive(Debug, thiserror::Error)]
pub enum PublishDirError {
    #[error("publish dir IO error: {0}")]
    IO(std::io::Error),

    #[error("publish dir is not a directory")]
    NotADirectory,
}

impl PublishDir {
    fn validate(path: &Path) -> Result<(), PublishDirError> {
        let path = prefix_local_path(path);
        let metadata = std::fs::metadata(&path).map_err(PublishDirError::IO)?;
        if !metadata.is_dir() {
            return Err(PublishDirError::NotADirectory);
        }
        Ok(())
    }

    fn path_with_extension(&self, external_id: &str, extension: &str) -> PathBuf {
        let mut path = self.as_ref().join(external_id);
        path.set_extension(extension);
        path
    }

    pub fn tmp_path(&self, external_id: &str) -> PathBuf {
        self.path_with_extension(external_id, "tmp")
    }

    pub fn jwt_path(&self, external_id: &str) -> PathBuf {
        self.path_with_extension(external_id, "jwt")
    }

    pub fn lock_for(&self, external_id: &str) -> PublishLock {
        PublishLock(self.path_with_extension(external_id, "lock"))
    }
}

#[derive(Debug, thiserror::Error)]
pub enum PublishLockError {
    #[error("could not create `{0}`: {1}")]
    Create(PathBuf, #[source] std::io::Error),

    #[error("could not flock on `{0}`: {1}")]
    Flock(PathBuf, Errno),

    #[error("could not join: {0}")]
    Join(#[from] JoinError),

    #[error("could not open `{0}`: {1}")]
    Open(PathBuf, #[source] std::io::Error),
}

pub struct PublishLock(PathBuf);

impl PublishLock {
    const CREATE_VERSION: usize = 0;

    pub fn create(&self) -> Result<(), PublishLockError> {
        File::create(&self.0)
            .and_then(|mut file| Self::write_version(&mut file, Self::CREATE_VERSION))
            .map_err(|err| PublishLockError::Create(self.0.clone(), err))
    }

    /// Get a lock and execute the passed `func` argument if the version is older
    ///
    /// Only executes when the `version` argument is greater than the version
    /// stored in the lock file. Note that a `version` of `0` is the same as an
    /// empty publication, i.e. if the file lock cannot be read the `version`
    /// is `0`.
    pub async fn with_lock_if_newer<F, E>(&self, version: usize, func: F) -> Result<bool, E>
    where
        E: From<PublishLockError>,
        F: AsyncFnOnce() -> Result<(), E>,
    {
        let path = self.0.clone();
        let (mut file, lock_version) = tokio::task::spawn_blocking(move || {
            let mut file = std::fs::OpenOptions::new()
                .read(true)
                .write(true)
                .open(&path)
                .map_err(|err| PublishLockError::Open(path.clone(), err))?;

            // Using flock for which OS will release lock when file is closed
            rustix::fs::flock(file.as_fd(), rustix::fs::FlockOperation::LockExclusive)
                .map_err(|err| PublishLockError::Flock(path.clone(), err))?;

            let lock_version = match Self::read_version(&mut file) {
                Ok(version) => version,
                Err(err) => {
                    // Default to CREATE_VERSION if reading fails
                    tracing::warn!("Could not read lock file `{}`: {}", path.display(), err);
                    Self::CREATE_VERSION
                }
            };
            Ok::<_, PublishLockError>((file, lock_version))
        })
        .await
        .map_err(PublishLockError::Join)??;

        if lock_version >= version {
            return Ok(false);
        }

        func().await?;

        // Ignore error when writing version lock file fails
        let path = self.0.clone();
        tokio::task::spawn_blocking(move || {
            _ = file
                .rewind()
                .and_then(|_| Self::write_version(&mut file, version))
                .inspect_err(|err| tracing::warn!("Could not write lock file `{}`: {}", path.display(), err));
        })
        .await
        .map_err(Into::into)?;

        Ok(true)
    }

    fn read_version(file: &mut File) -> Result<usize, std::io::Error> {
        let mut buf = [0; size_of::<usize>()];
        file.read_exact(&mut buf).map(|_| usize::from_le_bytes(buf))
    }

    fn write_version(file: &mut File, version: usize) -> Result<(), std::io::Error> {
        file.write_all(version.to_le_bytes().as_ref())
    }
}

#[cfg(test)]
mod tests {
    use std::io::Seek;
    use std::sync::atomic::AtomicBool;
    use std::sync::atomic::Ordering;

    use assert_matches::assert_matches;
    use rstest::rstest;
    use tempfile::NamedTempFile;

    use super::*;

    #[test]
    fn publish_dir_should_succeed_on_dir() {
        let tempdir = tempfile::tempdir().unwrap();
        let result = PublishDir::try_new(tempdir.path().to_path_buf());
        assert_matches!(result, Ok(_));
    }

    #[test]
    fn publish_dir_should_fail_on_non_existing_dir() {
        let tempdir = tempfile::tempdir().unwrap();
        let path = tempdir.path().to_path_buf();
        drop(tempdir);

        let result = PublishDir::try_new(path);
        assert_matches!(result, Err(PublishDirError::IO(_)));
    }

    #[test]
    fn publish_dir_should_fail_on_non_dir() {
        let tempfile = NamedTempFile::new().unwrap();
        let result = PublishDir::try_new(tempfile.path().to_path_buf());
        assert_matches!(result, Err(PublishDirError::NotADirectory));
    }

    #[test]
    fn create_lock_file_should_write_version() {
        let mut file = NamedTempFile::new().unwrap();

        let lock = PublishLock(file.path().to_path_buf());
        lock.create().unwrap();
        file.rewind().unwrap();
        assert_eq!(0, PublishLock::read_version(file.as_file_mut()).unwrap());
    }

    #[rstest]
    #[case(2, 1, false)]
    #[case(2, 2, false)]
    #[case(2, 3, true)]
    #[tokio::test]
    async fn publish_with_lock_should_only_called_when_newer(
        #[case] lock_version: usize,
        #[case] version: usize,
        #[case] publish: bool,
    ) {
        let mut file = NamedTempFile::new().unwrap();
        PublishLock::write_version(file.as_file_mut(), lock_version).unwrap();
        file.rewind().unwrap();

        let hit = AtomicBool::new(false);
        let lock = PublishLock(file.path().to_path_buf());
        let published = lock
            .with_lock_if_newer(version, async || {
                hit.store(true, Ordering::Relaxed);
                Ok::<_, PublishLockError>(())
            })
            .await
            .unwrap();
        assert_eq!(hit.load(Ordering::Acquire), publish);
        assert_eq!(published, publish);

        file.rewind().unwrap();
        // Check lock file contents to verify if the file is overwritten
        let mut lock_contents = Vec::new();
        file.read_to_end(&mut lock_contents).unwrap();
        if publish {
            assert_eq!(lock_contents, version.to_le_bytes());
        } else {
            assert_eq!(lock_contents, lock_version.to_le_bytes());
        }
    }

    #[tokio::test]
    async fn publish_with_lock_should_work_with_empty_lock_file() {
        let mut file = NamedTempFile::new().unwrap();

        let hit = AtomicBool::new(false);
        let lock = PublishLock(file.path().to_path_buf());
        let published = lock
            .with_lock_if_newer(1, async || {
                hit.store(true, Ordering::Relaxed);
                Ok::<_, PublishLockError>(())
            })
            .await
            .unwrap();

        assert!(hit.load(Ordering::Acquire));
        assert!(published);

        file.rewind().unwrap();
        assert_eq!(PublishLock::read_version(file.as_file_mut()).unwrap(), 1);
    }

    #[tokio::test]
    async fn publish_with_lock_should_fail_if_lock_does_not_exist() {
        let file = NamedTempFile::new().unwrap();
        let path = file.path().to_path_buf();
        drop(file);

        let lock = PublishLock(path.to_path_buf());
        let result = lock.with_lock_if_newer(1, async || Ok(())).await;

        assert_matches!(result, Err(PublishLockError::Open(err_path, _)) if err_path == path);
    }
}

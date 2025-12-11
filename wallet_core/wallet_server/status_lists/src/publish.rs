use std::fmt::Display;
use std::fs::File;
use std::io::Read;
use std::io::Seek;
use std::io::Write;
use std::os::fd::AsFd;
use std::path::Path;
use std::path::PathBuf;

use chrono::DateTime;
use chrono::Utc;
use nutype::nutype;
use rustix::io::Errno;
use tokio::task::JoinError;

use utils::path::prefix_local_path;

#[nutype(
    derive(Debug, Clone, TryFrom, Into, AsRef, PartialEq, Deserialize),
    sanitize(with=PublishDir::sanitize),
    validate(with=PublishDir::validate, error=PublishDirError)
)]
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
    #[expect(clippy::needless_pass_by_value, reason = "nutype sanitize signature")]
    fn sanitize(path: PathBuf) -> PathBuf {
        prefix_local_path(&path).into_owned()
    }

    fn validate(path: &Path) -> Result<(), PublishDirError> {
        let metadata = std::fs::metadata(path).map_err(PublishDirError::IO)?;
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

/// Version information inside the lock file that describes the version of the status list.
///
/// Multiple writers can try to publish a status list. Although a file lock is
/// used to prevent concurrent writes, it is also necessary to order the writes.
/// A writer that writes later, can have an older view of the status list from
/// the database.
///
/// We can use the number of revocations as major version because revocations
/// are irreversible and the database will always return committed rows, so no
/// interleaving can happen. The expiration is used as minor version to update
/// a list with the same revocations for a newer one.
#[derive(Debug, Default, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct LockVersion {
    number_of_revocations: usize,
    expiration: i64,
}

impl LockVersion {
    pub fn from(number_of_revocations: usize, expiration: DateTime<Utc>) -> Self {
        Self {
            number_of_revocations,
            expiration: expiration.timestamp(),
        }
    }

    fn read_from_io(reader: &mut impl Read) -> Result<Self, std::io::Error> {
        let mut buf = [0; { size_of::<usize>() + size_of::<i64>() }];
        reader.read_exact(&mut buf)?;

        // Unwrap is safe as the buf is the length of both combined
        let number_of_revocations = usize::from_le_bytes(buf[..size_of::<usize>()].try_into().unwrap());
        let expiration = i64::from_le_bytes(buf[size_of::<usize>()..].try_into().unwrap());

        Ok(Self {
            number_of_revocations,
            expiration,
        })
    }

    fn write_to_io(&self, writer: &mut impl Write) -> Result<(), std::io::Error> {
        let mut buf = [0; { size_of::<usize>() + size_of::<i64>() }];
        buf[..size_of::<usize>()].copy_from_slice(&self.number_of_revocations.to_le_bytes());
        buf[size_of::<usize>()..].copy_from_slice(&self.expiration.to_le_bytes());
        writer.write_all(&buf)
    }
}

impl PublishLock {
    pub fn create(&self, expiration: DateTime<Utc>) -> Result<(), PublishLockError> {
        let version = LockVersion::from(0, expiration);
        File::create(&self.0)
            .and_then(|mut file| version.write_to_io(&mut file))
            .map_err(|err| PublishLockError::Create(self.0.clone(), err))
    }

    /// Get a lock and execute the passed `func` argument if the version is older
    ///
    /// Only executes when the `version` argument is newer than the version
    /// stored in the lock file. If the file lock cannot be read the `version`
    /// is considered to be default and published again.
    pub async fn with_lock_if_newer<F, E>(&self, version: LockVersion, func: F) -> Result<bool, E>
    where
        E: From<PublishLockError>,
        F: AsyncFnOnce() -> Result<(), E>,
    {
        let path = self.0.clone();
        let (mut file, file_version) = tokio::task::spawn_blocking(move || {
            let mut file = std::fs::OpenOptions::new()
                .read(true)
                .write(true)
                .open(&path)
                .map_err(|err| PublishLockError::Open(path.clone(), err))?;

            // Using flock for which OS will release lock when file is closed
            rustix::fs::flock(file.as_fd(), rustix::fs::FlockOperation::LockExclusive)
                .map_err(|err| PublishLockError::Flock(path.clone(), err))?;

            let file_version = match LockVersion::read_from_io(&mut file) {
                Ok(version) => version,
                Err(err) => {
                    // Default to zero version if reading fails, so will republish
                    tracing::warn!("Could not read lock file `{}`: {}", path.display(), err);
                    LockVersion::default()
                }
            };
            Ok::<_, PublishLockError>((file, file_version))
        })
        .await
        .map_err(PublishLockError::Join)??;

        if file_version >= version {
            return Ok(false);
        }

        func().await?;

        // Ignore error when writing version lock file fails
        let path = self.0.clone();
        tokio::task::spawn_blocking(move || {
            _ = file
                .rewind()
                .and_then(|_| file.set_len(0))
                .and_then(|_| version.write_to_io(&mut file))
                .inspect_err(|err| tracing::warn!("Could not write lock file `{}`: {}", path.display(), err));
        })
        .await
        .map_err(Into::into)?;

        Ok(true)
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
    fn default_lock_version() {
        let version = LockVersion::default();
        assert_eq!(version.number_of_revocations, 0);
        assert_eq!(version.expiration, 0);
    }

    #[test]
    fn lock_version_serialize_deserialize() {
        let version = LockVersion {
            number_of_revocations: 1337,
            expiration: Utc::now().timestamp(),
        };

        let read = {
            let mut buf = Vec::new();
            version.write_to_io(&mut buf).unwrap();
            LockVersion::read_from_io(&mut buf.as_slice()).unwrap()
        };
        assert_eq!(version, read);
    }

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

        let dt = Utc::now();
        let lock = PublishLock(file.path().to_path_buf());
        lock.create(dt).unwrap();
        file.rewind().unwrap();
        assert_eq!(
            LockVersion::from(0, dt),
            LockVersion::read_from_io(file.as_file_mut()).unwrap()
        );
    }

    async fn publish_with_lock_if_newer(file: &NamedTempFile, version: LockVersion) -> bool {
        let hit = AtomicBool::new(false);
        let lock = PublishLock(file.path().to_path_buf());
        let published = lock
            .with_lock_if_newer(version, async || {
                hit.store(true, Ordering::Relaxed);
                Ok::<_, PublishLockError>(())
            })
            .await
            .unwrap();
        assert_eq!(hit.load(Ordering::Acquire), published);
        published
    }

    #[rstest]
    #[case(LockVersion { number_of_revocations: 1, expiration: 3 }, false)]
    #[case(LockVersion { number_of_revocations: 2, expiration: 1 }, false)]
    #[case(LockVersion { number_of_revocations: 2, expiration: 2 }, false)]
    #[case(LockVersion { number_of_revocations: 2, expiration: 3 }, true)]
    #[case(LockVersion { number_of_revocations: 3, expiration: 1 }, true)]
    #[tokio::test]
    async fn publish_with_lock_should_only_called_when_newer(#[case] version: LockVersion, #[case] publish: bool) {
        let mut file = NamedTempFile::new().unwrap();
        LockVersion {
            number_of_revocations: 2,
            expiration: 2,
        }
        .write_to_io(file.as_file_mut())
        .unwrap();
        file.rewind().unwrap();

        let published = publish_with_lock_if_newer(&file, version.clone()).await;
        assert_eq!(published, publish);

        file.rewind().unwrap();
        // Check lock file contents to verify if the file is overwritten
        let mut lock_contents = Vec::new();
        file.read_to_end(&mut lock_contents).unwrap();
        if publish {
            assert_eq!(lock_contents[0], version.number_of_revocations as u8);
            assert_eq!(lock_contents[size_of::<usize>()], version.expiration as u8);
        } else {
            assert_eq!(lock_contents[0], 2);
            assert_eq!(lock_contents[size_of::<usize>()], 2);
        }
    }

    #[tokio::test]
    async fn publish_with_lock_should_work_with_empty_lock_file() {
        let mut file = NamedTempFile::new().unwrap();

        let version = LockVersion::from(1, Utc::now());
        let published = publish_with_lock_if_newer(&file, version.clone()).await;
        assert!(published);

        file.rewind().unwrap();
        assert_eq!(version, LockVersion::read_from_io(file.as_file_mut()).unwrap());
    }

    #[tokio::test]
    async fn publish_with_lock_should_rewrite_lock_file() {
        let mut file = NamedTempFile::new().unwrap();
        file.write_all(&[0; 20]).unwrap();
        file.rewind().unwrap();

        let version = LockVersion::from(2, Utc::now());
        let published = publish_with_lock_if_newer(&file, version.clone()).await;
        assert!(published);

        file.rewind().unwrap();
        assert_eq!(version, LockVersion::read_from_io(file.as_file_mut()).unwrap());

        let mut buf = Vec::new();
        file.read_to_end(&mut buf).unwrap();
        assert_eq!(buf.len(), 0);
    }

    #[tokio::test]
    async fn publish_with_lock_should_fail_if_lock_does_not_exist() {
        let file = NamedTempFile::new().unwrap();
        let path = file.path().to_path_buf();
        drop(file);

        let lock = PublishLock(path.to_path_buf());
        let result = lock
            .with_lock_if_newer(LockVersion::from(1, Utc::now()), async || Ok(()))
            .await;

        assert_matches!(result, Err(PublishLockError::Open(err_path, _)) if err_path == path);
    }
}

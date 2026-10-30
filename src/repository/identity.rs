//! Exact streaming file/blob identities shared by snapshots and isolated indexing.

use sha1::Sha1;
use sha2::{Digest, Sha256};
use std::{
    fs::{self, File, Metadata},
    io::Read,
    path::Path,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct FileIdentity {
    pub blob: String,
    pub sha256: String,
    pub blob_sha256: String,
    pub mode: u32,
    pub link: bool,
}

pub(crate) fn open_regular_file(path: &Path) -> std::io::Result<std::fs::File> {
    let mut options = std::fs::File::options();
    options.read(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.custom_flags(libc::O_NOFOLLOW | libc::O_NONBLOCK);
    }
    let file = options.open(path)?;
    if !file.metadata()?.is_file() {
        return Err(std::io::Error::other("Index/input is not a regular file"));
    }
    Ok(file)
}

pub(crate) fn read_identity(
    path: &Path,
    limit: u64,
    check: impl Fn() -> Result<(), String>,
) -> Result<FileIdentity, String> {
    check()?;
    let metadata = fs::symlink_metadata(path).map_err(|e| e.to_string())?;
    if metadata.file_type().is_symlink() {
        return Ok(bytes_identity(
            link_bytes(&fs::read_link(path).map_err(|e| e.to_string())?)?,
            0,
            true,
        ));
    }
    if !metadata.is_file() || metadata.len() > limit {
        return Err("Input type or size changed".into());
    }
    let mut file = open_regular_file(path).map_err(|e| e.to_string())?;
    hash_file(
        &mut file,
        metadata.len(),
        file_mode(&metadata),
        check,
        |_| Ok(()),
    )
}

pub(crate) fn hash_file(
    file: &mut File,
    size: u64,
    mode: u32,
    check: impl Fn() -> Result<(), String>,
    mut consume: impl FnMut(&[u8]) -> Result<(), String>,
) -> Result<FileIdentity, String> {
    let mut blob = Sha1::new();
    blob.update(format!("blob {size}\0"));
    let mut blob256 = Sha256::new();
    blob256.update(format!("blob {size}\0"));
    let mut digest = Sha256::new();
    let mut copied = 0;
    let mut buffer = [0_u8; 64 * 1024];
    loop {
        check()?;
        let count = file.read(&mut buffer).map_err(|e| e.to_string())?;
        if count == 0 {
            break;
        }
        copied += count as u64;
        if copied > size {
            return Err("Input grew while preparing the index".into());
        }
        let bytes = &buffer[..count];
        blob.update(bytes);
        blob256.update(bytes);
        digest.update(bytes);
        consume(bytes)?;
    }
    if copied != size {
        return Err("Input shrank while preparing the index".into());
    }
    Ok(FileIdentity {
        blob: format!("{:x}", blob.finalize()),
        sha256: format!("{:x}", digest.finalize()),
        blob_sha256: format!("{:x}", blob256.finalize()),
        mode,
        link: false,
    })
}

pub(crate) fn bytes_identity(bytes: &[u8], mode: u32, link: bool) -> FileIdentity {
    let mut blob = Sha1::new();
    blob.update(format!("blob {}\0", bytes.len()));
    blob.update(bytes);
    let mut blob256 = Sha256::new();
    blob256.update(format!("blob {}\0", bytes.len()));
    blob256.update(bytes);
    FileIdentity {
        blob: format!("{:x}", blob.finalize()),
        sha256: format!("{:x}", Sha256::digest(bytes)),
        blob_sha256: format!("{:x}", blob256.finalize()),
        mode,
        link,
    }
}

#[cfg(unix)]
pub(crate) fn link_bytes(path: &Path) -> Result<&[u8], String> {
    use std::os::unix::ffi::OsStrExt;
    Ok(path.as_os_str().as_bytes())
}
#[cfg(not(unix))]
pub(crate) fn link_bytes(path: &Path) -> Result<&[u8], String> {
    path.to_str()
        .map(str::as_bytes)
        .ok_or("Link target is not UTF-8".into())
}

#[cfg(unix)]
pub(crate) fn file_mode(metadata: &Metadata) -> u32 {
    use std::os::unix::fs::PermissionsExt;
    metadata.permissions().mode() & 0o777
}
#[cfg(not(unix))]
pub(crate) fn file_mode(metadata: &Metadata) -> u32 {
    u32::from(metadata.permissions().readonly())
}

//! Managment for the files kept on disk
//!
//! These include both the files for goup, as well as the actual Go installation files

use crate::version::GoVersion;
use directories::UserDirs;
use serde::Deserialize;
use serde::Serialize;
use std::collections::BTreeSet;
use std::fs;
use std::io;
#[cfg(target_os = "linux")]
use std::os::unix::fs::symlink;
use std::path::PathBuf;

pub fn goup_dir() -> io::Result<PathBuf> {
    UserDirs::new()
        .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "unable to find home dir"))
        .map(|dirs| dirs.home_dir().join(".goup"))
}

pub fn install_dir(version: GoVersion) -> io::Result<PathBuf> {
    goup_dir().map(|p| p.join(format!("{}", version)))
}

fn record_file() -> io::Result<PathBuf> {
    goup_dir().map(|p| p.join("versions.json"))
}

/// Enable the given version of Go
#[cfg(target_os = "linux")]
pub fn enable_version(version: GoVersion) -> io::Result<()> {
    let mut records_file = RecordFile::load()?;
    if !records_file.installed.contains(&version) {
        return Err(io::Error::new(
            io::ErrorKind::NotFound,
            format!("Version {} is not installed", version),
        ));
    }

    if let Err(e) = fs::remove_file(goup_dir()?.join("go")) {
        if !matches!(e.kind(), io::ErrorKind::NotFound) {
            return Err(e);
        }
    }

    let res = symlink(install_dir(version)?.join("go"), goup_dir()?.join("go"));
    if res.is_ok() {
        records_file.enabled = version;
        records_file.store()?;
    }
    res
}

#[derive(Debug, Default, Deserialize, Serialize)]
pub struct RecordFile {
    pub enabled: GoVersion,
    pub installed: BTreeSet<GoVersion>,
}

impl RecordFile {
    pub fn load() -> io::Result<RecordFile> {
        match fs::read_to_string(record_file()?) {
            Ok(x) => serde_json::from_str(&x).map_err(|_| {
                io::Error::new(
                    io::ErrorKind::InvalidData,
                    "unable to parse version information file",
                )
            }),
            Err(e) if matches!(e.kind(), io::ErrorKind::NotFound) => Ok(Default::default()),
            Err(e) => Err(e),
        }
    }

    pub fn store(&self) -> io::Result<()> {
        let payload = serde_json::to_string_pretty(&self).map_err(|_| {
            io::Error::new(
                io::ErrorKind::InvalidData,
                "unable to serialize record file",
            )
        })?;
        fs::write(record_file()?, payload)
    }
}

use directories::UserDirs;
use flate2::read::GzDecoder;
use regex::Regex;
use serde::de::Visitor;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
use std::fmt::{self, Display};
#[cfg(unix)]
use std::os::unix::fs::symlink;
use std::path::PathBuf;
use std::str::FromStr;
use std::{env, fs, io};
use tar::Archive;

/// A semantic version tag, in Go format
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct GoVersion {
    pub major: u32,
    pub minor: u32,
    pub patch: u32,
}

impl Serialize for GoVersion {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&format!("{}", self))
    }
}

// Serde visitor for handling deserialization
struct GoVersionVisitor;

impl<'de> Visitor<'de> for GoVersionVisitor {
    type Value = GoVersion;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("a version number, prefixed by 'go' (eg: go1.19.1)")
    }

    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        v.parse().map_err(|e| serde::de::Error::custom(e))
    }
}

impl<'de> Deserialize<'de> for GoVersion {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_str(GoVersionVisitor)
    }
}

impl FromStr for GoVersion {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        lazy_static! {
            static ref PARSING_REGEX: Regex = Regex::new(r"go(\d+)\.(\d+)(?:\.(\d+))?").unwrap();
        }

        match PARSING_REGEX.captures(s) {
            Some(x) => {
                return Ok(Self {
                    major: x
                        .get(1)
                        .map(|x| x.as_str().parse::<u32>().unwrap())
                        .unwrap_or_default(),
                    minor: x
                        .get(2)
                        .map(|x| x.as_str().parse::<u32>().unwrap())
                        .unwrap_or_default(),
                    patch: x
                        .get(3)
                        .map(|x| x.as_str().parse::<u32>().unwrap())
                        .unwrap_or_default(),
                });
            }
            None => return Err("unable to parse go version"),
        }
    }
}

impl Display for GoVersion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "go{}.{}.{}", self.major, self.minor, self.patch)
    }
}

#[derive(Debug, Default, Deserialize, Serialize)]
pub struct VersionFile {
    pub enabled: Option<GoVersion>,
    pub installed: BTreeSet<GoVersion>,
    pub pinned: BTreeSet<GoVersion>,
}

impl VersionFile {
    pub fn load() -> io::Result<VersionFile> {
        match fs::read_to_string(version_file()?) {
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
        fs::write(version_file()?, payload)
    }
}

#[derive(Debug, Deserialize)]
struct VersionInfo {
    version: GoVersion,
    // stable: bool,
    files: Vec<FileInfo>,
}

#[derive(Debug, Deserialize)]
pub struct FileInfo {
    pub filename: String,
    pub os: String,
    pub arch: String,
    pub version: String,
    pub sha256: String,
    pub size: u64,
    pub kind: String,
}

/// Get the set of available versions of Go from Go's website.
pub fn available_go_versions() -> Result<BTreeMap<GoVersion, FileInfo>, String> {
    let available = ureq::get("https://go.dev/dl/?mode=json")
        .call()
        .map_err(|_| "Failed to request version info")?
        .into_json::<Vec<VersionInfo>>()
        .map_err(|_| "Unable to parse version info from remote")?
        .into_iter()
        .filter_map(|group| {
            group
                .files
                .into_iter()
                .find(|file| file.arch == arch() && file.os == env::consts::OS)
                .map(|f| (group.version, f))
        })
        .collect();

    Ok(available)
}

pub fn download_version(version: GoVersion, file: &FileInfo) -> Result<(), String> {
    let mut version_file = VersionFile::load().map_err(|e| e.to_string())?;
    let needs_install = version_file.installed.insert(version);

    if needs_install {
        let stream_reader = ureq::get(&format!("https://go.dev/dl/{}", file.filename))
            .call()
            .map_err(|_| "Failed to get file from go.dev")?
            .into_reader();
        Archive::new(GzDecoder::new(stream_reader))
            .unpack(install_dir(version).map_err(|e| e.to_string())?)
            .map_err(|e| e.to_string())?;
        version_file.store().map_err(|e| e.to_string())?;
    }

    Ok(())
}

#[cfg(target_os = "linux")]
pub fn enable_version(version: GoVersion) -> io::Result<()> {
    let mut records_file = VersionFile::load()?;
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
        records_file.enabled = Some(version);
        records_file.store()?;
    }
    res
}

pub fn remove_version(version: GoVersion) -> io::Result<()> {
    let mut records_file = VersionFile::load()?;
    if !records_file.installed.remove(&version) {
        return Err(io::Error::new(
            io::ErrorKind::NotFound,
            format!("Version {} is not installed", version),
        ));
    } else if records_file.pinned.contains(&version) {
        return Err(io::Error::new(
            io::ErrorKind::PermissionDenied,
            format!("Version {} is pinned", version),
        ));
    }

    if records_file.enabled.is_some() && records_file.enabled.unwrap() == version {
        records_file.enabled = None;
        println!(
            "Version {} is currently enabled. Use 'goup enable' to select another.",
            version
        );
    }

    fs::remove_dir_all(install_dir(version)?)?;
    records_file.store()?;
    Ok(())
}

pub fn version_folders() -> io::Result<BTreeSet<GoVersion>> {
    let mut versions = BTreeSet::new();
    for entry in fs::read_dir(goup_dir()?)? {
        let version = entry?
            .file_name()
            .to_str()
            .and_then(|name| name.parse::<GoVersion>().ok());
        if let Some(v) = version {
            versions.insert(v);
        }
    }

    Ok(versions)
}

/// A mapping of the architecture from what Rust calls it to what Go calls it
fn arch() -> &'static str {
    match env::consts::ARCH {
        "x86" => "386",
        "x86_64" => "amd64",
        "aarch64" => "arm64",
        "powerpc64" => "ppc64le",
        "s390x" => "s390x",
        _ => "",
    }
}

/// The directory that goup uses to install Go versions and manage its internal config
fn goup_dir() -> io::Result<PathBuf> {
    UserDirs::new()
        .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "unable to find home dir"))
        .map(|dirs| dirs.home_dir().join(".goup"))
}

/// The directory that the provided Go version should be installed into
pub fn install_dir(version: GoVersion) -> io::Result<PathBuf> {
    goup_dir().map(|p| p.join(format!("{}", version)))
}

/// The location of the file describing the versions installed and enabled
fn version_file() -> io::Result<PathBuf> {
    goup_dir().map(|p| p.join("versions.json"))
}

use anyhow::{anyhow, Context, Result};
use flate2::read::GzDecoder;
use indicatif::{ProgressBar, ProgressStyle};
use regex::Regex;
use serde::de::Visitor;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
use std::env::{var, VarError};
use std::fmt::{self, Display};
use std::io::Read;
#[cfg(unix)]
use std::os::unix::fs::symlink;
use std::path::{Path, PathBuf};
use std::str::FromStr;
use std::sync::LazyLock;
use std::{env, fs, io};
use tar::Archive;

static PARSING_REGEX: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"go(\d+)\.(\d+)(?:\.(\d+))?").unwrap());

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
        serializer.serialize_str(self.to_string().as_str())
    }
}

// Serde visitor for handling deserialization
struct GoVersionVisitor;

impl Visitor<'_> for GoVersionVisitor {
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
        match PARSING_REGEX.captures(s) {
            Some(x) => Ok(Self {
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
            }),
            None => Err("unable to parse go version"),
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
    pub fn load() -> Result<VersionFile> {
        match fs::read_to_string(version_file()?) {
            Ok(x) => serde_json::from_str(&x).with_context(|| "Unable to parse version file"),
            Err(e) if matches!(e.kind(), io::ErrorKind::NotFound) => Ok(Default::default()),
            Err(e) => Err(anyhow!(e)),
        }
    }

    pub fn store(&self) -> Result<()> {
        let payload = serde_json::to_string_pretty(&self)
            .with_context(|| "Unable to serialize version file")?;
        fs::write(version_file()?, payload).with_context(|| "Unable to write version file")
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
    // pub version: String,
    // pub sha256: String,
    pub size: u64,
    // pub kind: String,
}

/// A shim that will count the number of bytes read out of the given reader and display it
/// on a progress bar.
#[derive(Debug)]
struct ByteCounter<R: Read> {
    inner: R,
    bar: ProgressBar,
}

impl<R: Read> ByteCounter<R> {
    pub fn new(inner: R, total_bytes: u64) -> Self {
        let bar = ProgressBar::new(total_bytes).with_style(
            ProgressStyle::with_template(
                "[{elapsed_precise}] [{bar:40.cyan/blue}] {bytes}/{total_bytes}",
            )
            .unwrap()
            .progress_chars("=> "),
        );

        Self { inner, bar }
    }
}

impl<R: Read> Read for ByteCounter<R> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        let res = self.inner.read(buf);
        if let Ok(size) = res {
            self.bar.inc(size as u64);
        }
        res
    }
}

impl<R: Read> Drop for ByteCounter<R> {
    fn drop(&mut self) {
        if self.bar.position() >= self.bar.length().unwrap_or_default() {
            self.bar.finish();
        } else {
            self.bar.abandon();
        }
    }
}

/// Get the set of available versions of Go from Go's website.
pub fn available_go_versions() -> Result<BTreeMap<GoVersion, FileInfo>> {
    let available = ureq::get("https://go.dev/dl/?mode=json")
        .call()
        .with_context(|| "Failed to request version info")?
        .body_mut()
        .read_json::<Vec<VersionInfo>>()
        .with_context(|| "Unable to parse version info from remote")?
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

pub fn download_version(version: GoVersion, file: &FileInfo) -> Result<()> {
    let mut version_file = VersionFile::load()?;
    let needs_install = version_file.installed.insert(version);

    if needs_install {
        let mut response_body = ureq::get(&format!("https://go.dev/dl/{}", file.filename))
            .call()
            .with_context(|| "Failed to get version archive from go.dev")?
            .into_body();
        Archive::new(GzDecoder::new(ByteCounter::new(
            response_body.as_reader(),
            file.size,
        )))
        .unpack(install_dir(version)?)
        .with_context(|| "Failed to unpack downloaded archive")?;
        version_file.store()?;
    }

    Ok(())
}

#[cfg(unix)]
pub fn enable_version(version: GoVersion) -> Result<()> {
    let mut records_file = VersionFile::load()?;
    if !records_file.installed.contains(&version) {
        return Err(anyhow!("Version {} is not installed", version));
    }

    if let Err(e) = fs::remove_file(goup_dir()?.join("go")) {
        if !matches!(e.kind(), io::ErrorKind::NotFound) {
            return Err(anyhow!(e));
        }
    }

    let res = symlink(install_dir(version)?.join("go"), goup_dir()?.join("go"))
        .with_context(|| "Unable to make symlink");
    if res.is_ok() {
        records_file.enabled = Some(version);
        records_file.store()?;
    }
    res
}

pub fn remove_version(version: GoVersion) -> Result<()> {
    let mut records_file = VersionFile::load()?;
    if !records_file.installed.remove(&version) {
        return Err(anyhow!("Version {} is not installed", version));
    } else if records_file.pinned.contains(&version) {
        return Err(anyhow!("Version {} is pinned", version));
    }

    if records_file.enabled.is_some() && records_file.enabled.unwrap() == version {
        records_file.enabled = None;
        println!(
            "Version {} was enabled. Use 'goup enable' to select another.",
            version
        );
    }

    fs::remove_dir_all(install_dir(version)?)?;
    records_file.store()?;
    Ok(())
}

pub fn version_folders() -> Result<BTreeSet<GoVersion>> {
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
fn goup_dir() -> Result<PathBuf> {
    match var("GOPATH") {
        Ok(p) => Ok(Path::new(&p).join("goup")),
        Err(VarError::NotPresent) => Err(anyhow!("GOPATH variable is not set")),
        Err(VarError::NotUnicode(_)) => Err(anyhow!("Unable to read GOPATH variable")),
    }
}

/// The directory that the provided Go version should be installed into
pub fn install_dir(version: GoVersion) -> Result<PathBuf> {
    goup_dir().map(|p| p.join(format!("{}", version)))
}

/// The location of the file describing the versions installed and enabled
fn version_file() -> Result<PathBuf> {
    goup_dir().map(|p| p.join("versions.json"))
}

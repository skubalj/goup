use flate2::read::GzDecoder;
use regex::Regex;
use serde::de::Visitor;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::env;
use std::fmt::{self, Display};
use std::process::Command;
use std::str::FromStr;
use tar::Archive;

use crate::records;

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

/// Get the current version of Go that is
pub fn current_go_version() -> Result<GoVersion, String> {
    let output = Command::new("go")
        .arg("version")
        .output()
        .expect("failed while getting go version");
    String::from_utf8(output.stdout)
        .map_err(|_| "output of Go command is not UTF-8")?
        .parse()
        .map_err(|_| String::from("unable to parse go version"))
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

pub fn download_version(version: GoVersion, file: &FileInfo) -> Result<(), String> {
    let stream_reader = ureq::get(&format!("https://go.dev/dl/{}", file.filename))
        .call()
        .map_err(|_| "Failed to get file from go.dev")?
        .into_reader();
    Archive::new(GzDecoder::new(stream_reader))
        .unpack(records::install_dir(version).map_err(|e| e.to_string())?)
        .map_err(|e| e.to_string())
}

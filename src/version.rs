use regex::Regex;
use serde::Deserialize;
use std::collections::BTreeMap;
use std::env;
use std::fmt::Display;
use std::process::Command;
use std::str::FromStr;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct GoVersion {
    pub major: u32,
    pub minor: u32,
    pub patch: u32,
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
        if self.patch == 0 {
            write!(f, "go{}.{}", self.major, self.minor)
        } else {
            write!(f, "go{}.{}.{}", self.major, self.minor, self.patch)
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct VersionInfo {
    pub version: String,
    pub stable: bool,
    pub files: Vec<FileInfo>,
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
        .map_err(|_| "failed to request version info")?
        .into_json::<Vec<VersionInfo>>()
        .map_err(|_| "unable to parse version info from remote")?
        .into_iter()
        .filter_map(|version| {
            version
                .files
                .into_iter()
                .find(|file| file.arch == arch() && file.os == env::consts::OS)
        })
        .filter_map(|file| {
            file.version
                .parse::<GoVersion>()
                .map(|version| (version, file))
                .ok()
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

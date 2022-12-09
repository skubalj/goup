use regex::Regex;
use serde::Deserialize;
use std::env;
use std::process::Command;
use std::str::FromStr;

#[macro_use]
extern crate lazy_static;

#[derive(Debug, Deserialize)]
struct VersionInfo {
    version: String,
    stable: bool,
    files: Vec<FileInfo>,
}

#[derive(Debug, Deserialize)]
struct FileInfo {
    filename: String,
    os: String,
    arch: String,
    version: String,
    sha256: String,
    size: u64,
    kind: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
struct GoVersion {
    major: u32,
    minor: u32,
    patch: u32,
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

fn main() {
    let output = Command::new("go")
        .arg("version")
        .output()
        .expect("failed while getting go version");
    let current_version: GoVersion = String::from_utf8(output.stdout).unwrap().parse().unwrap();
    println!("Current Version: {:?}", current_version);

    let version_info = ureq::get("https://go.dev/dl/?mode=json")
        .call()
        .expect("failed to request version.info")
        .into_json::<Vec<VersionInfo>>()
        .expect("unable to parse version info");

    let available_files: Vec<&FileInfo> = version_info
        .iter()
        .filter_map(|version| {
            version
                .files
                .iter()
                .find(|file| file.arch == arch() && file.os == env::consts::OS)
        })
        .collect();

    let mut versions: Vec<GoVersion> = available_files
        .iter()
        .map(|f| f.version.parse::<GoVersion>().unwrap())
        .collect();
    versions.sort();
    versions.reverse();

    println!("{versions:?}");
}

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

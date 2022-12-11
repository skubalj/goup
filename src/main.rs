use std::io;

use crate::records::RecordFile;
use clap::{Parser, Subcommand};
use version::GoVersion;

#[macro_use]
extern crate lazy_static;

mod records;
mod version;

/// Go version manager and multiplexor
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// List the set of available Go versions, as well as those that are installed.
    List,
    /// Update all versions of Go to their latest patch releases.
    Update,
    /// Install a new version of Go.
    Install {
        /// The version of Go that will be installed
        version: GoVersion,
    },
    /// Enable the given go version. This can be used to roll back updates, for example.
    Enable {
        /// The version of Go that will be enabled
        version: GoVersion,
    },
    /// Remove old Go versions.
    Clean,
}

fn main() {
    let args = Args::parse();

    match args.command {
        Commands::List => list_versions(),
        Commands::Update => update(),
        Commands::Install { version } => install(version),
        Commands::Enable { version } => enable(version),
        Commands::Clean => println!("Clean"),
    }
}

fn list_versions() {
    match version::available_go_versions() {
        Ok(v) => v
            .values()
            .rev()
            .map(|file_info| &file_info.version)
            .for_each(|v| println!(" {}", v)),
        Err(e) => {
            eprintln!("{}", e);
            return;
        }
    }
}

fn update() {
    let current_version = match version::current_go_version() {
        Ok(x) => x,
        Err(e) => {
            eprintln!("{}", e);
            return;
        }
    };

    let available_versions = match version::available_go_versions() {
        Ok(v) => v,
        Err(e) => {
            eprintln!("{}", e);
            return;
        }
    };

    let (highest_version, update_file) = match available_versions.iter().max_by_key(|x| x.0) {
        Some(x) => x,
        None => {
            eprintln!("No versions available");
            return;
        }
    };

    if &current_version < highest_version {
        println!("NEEDS UPDATE");
    } else {
        println!("Already up to date with version {}", current_version);
        return;
    }
}

fn install(v: GoVersion) {
    let mut rf = RecordFile::load().expect("");
    rf.installed.insert(v);

    let available = match version::available_go_versions() {
        Ok(x) => x,
        Err(e) => {
            eprintln!("{}", e);
            return;
        }
    };

    let res = available
        .get(&v)
        .ok_or_else(|| format!("Version {} not available for download", v))
        .and_then(|f| version::download_version(v, f));
    match res {
        Ok(_) => {
            if let Err(e) = rf.store() {
                eprintln!("Unable to write out version info file {}", e);
            };
            println!("{} installed successfully", v);
        }
        Err(e) => eprintln!("{}", e),
    }
}

fn enable(version: GoVersion) {
    if let Err(e) = records::enable_version(version) {
        match e.kind() {
            io::ErrorKind::NotFound => eprintln!("Version {} is not installed.", version),
            _ => eprintln!("{}", e),
        }
    }
}

use anyhow::{anyhow, Context, Result};
use clap::{Parser, Subcommand};
use console::style;
use std::collections::BTreeSet;
use std::fs;
use version::{GoVersion, VersionFile};

mod version;

/// Go version manager and multiplexer
///
/// goup allows users to install new versions of Go to their user directory, as well as switch
/// between installed versions.
#[derive(Parser, Debug)]
#[command(author, version, about)]
struct Args {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// List the set of available Go versions, as well as those that are installed.
    List,
    /// Automatically install and enable the latest version of Go
    Update,
    /// Install a new version of Go.
    Install {
        /// The version of Go that will be installed
        version: GoVersion,
    },
    /// Enable the given Go version. This can be used to roll back updates, for example.
    Enable {
        /// The version of Go that will be enabled
        version: GoVersion,
    },
    /// Remove an installed Go version
    Remove {
        /// The version of Go that will be removed
        version: GoVersion,
    },
    /// Pin the given Go version to keep it from being removed
    Pin {
        /// The version of Go that will be pinned
        version: GoVersion,
    },
    /// Unpin the given Go version, allowing it to be removed
    Unpin {
        /// The version of Go that will be unpinned
        version: GoVersion,
    },
    /// Remove Go versions that are out of date (no longer available from go.dev)
    Clean,
}

fn main() {
    let args = Args::parse();

    let res = match args.command {
        Commands::List => list_versions(),
        Commands::Update => update(),
        Commands::Install { version } => install(version),
        Commands::Enable { version } => enable(version),
        Commands::Remove { version } => remove(version),
        Commands::Pin { version } => pin(version),
        Commands::Unpin { version } => unpin(version),
        Commands::Clean => clean(),
    };

    if let Err(e) = res {
        eprintln!("Error: {:#}", e);
    }
}

fn list_versions() -> Result<()> {
    let VersionFile {
        enabled,
        installed,
        pinned,
    } = VersionFile::load()?;
    let available = version::available_go_versions()?
        .into_keys()
        .collect::<BTreeSet<_>>();

    let mut versions = Vec::new();
    for v in installed.union(&available) {
        let is_installed = installed.contains(v);
        let is_available = available.contains(v);
        let is_enabled = enabled.is_some() && *v == enabled.unwrap();
        let is_pinned = pinned.contains(v);

        let bullet = if is_enabled {
            "*"
        } else if is_installed {
            "i"
        } else {
            " "
        };
        let pinned_text = if is_pinned { " (PINNED)" } else { "" };
        let string = format!("{} {}{}", bullet, v, pinned_text);

        let paint = match (is_installed, is_available, is_enabled) {
            (true, true, _) => style(string).green(),
            (true, false, true) => style(string).red(),
            (true, false, false) => style(string).yellow(),
            _ => style(string),
        };

        versions.push(paint);
    }

    for v in versions.iter().rev() {
        println!("{}", v);
    }

    Ok(())
}

fn update() -> Result<()> {
    let records = VersionFile::load()?;
    let available = version::available_go_versions()?;
    let (&latest_version, file_info) = available
        .last_key_value()
        .ok_or_else(|| anyhow!("Found no available go versions"))?;

    if records.installed.contains(&latest_version) {
        enable(latest_version)?;
        println!("The latest version is {}", latest_version);
        println!("Already up to date!");
        return Ok(());
    } else {
        println!("Version {} is available", latest_version);
    }

    version::download_version(latest_version, file_info)?;
    version::enable_version(latest_version)?;
    println!("Installed and enabled version {}", latest_version);
    println!(
        "Use 'goup clean' to remove old versions, or 'goup enable {}' to roll back",
        records.enabled.unwrap_or_default()
    );
    Ok(())
}

fn install(v: GoVersion) -> Result<()> {
    let mut rf = VersionFile::load()?;
    rf.installed.insert(v);

    version::available_go_versions()?
        .get(&v)
        .ok_or_else(|| anyhow!("Version {} not available for download", v))
        .and_then(|f| version::download_version(v, f))?;

    rf.store()
        .with_context(|| "Unable to write out version file")?;

    println!("{} installed successfully", v);
    Ok(())
}

fn enable(version: GoVersion) -> Result<()> {
    version::enable_version(version)
}

fn remove(version: GoVersion) -> Result<()> {
    version::remove_version(version)?;
    println!("{} uninstalled successfully", version);
    Ok(())
}

fn pin(version: GoVersion) -> Result<()> {
    let mut version_file = VersionFile::load()?;
    if !version_file.installed.contains(&version) {
        return Err(anyhow!("Version {} is not installed.", version));
    }

    version_file.pinned.insert(version);
    version_file.store()?;
    Ok(())
}

fn unpin(version: GoVersion) -> Result<()> {
    let mut version_file = VersionFile::load()?;
    version_file.pinned.remove(&version);
    version_file.store()?;
    Ok(())
}

fn clean() -> Result<()> {
    let mut version_file = VersionFile::load()?;
    let folder_versions = version::version_folders()?;

    // Fix our list of installed versions to only include those that are actually on disk.
    // This would indicate that someone was tampering with our .goup directory.
    version_file.installed = version_file
        .installed
        .intersection(&folder_versions)
        .copied()
        .collect();
    version_file.pinned = version_file
        .installed
        .intersection(&version_file.pinned)
        .copied()
        .collect();

    // Keep any version of Go that is still available, that is pinned, or enabled.
    let allowlist: BTreeSet<_> = version::available_go_versions()?
        .into_keys()
        .chain(version_file.pinned.iter().copied())
        .chain(version_file.enabled)
        .collect();

    for version in folder_versions.difference(&allowlist) {
        version_file.installed.remove(version);
        fs::remove_dir_all(version::install_dir(*version)?)?;
    }

    version_file.store()?;
    Ok(())
}

use clap::{Parser, Subcommand};

#[macro_use]
extern crate lazy_static;

mod version;

/// Simple program to greet a person
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// List the set of values that
    List,
    /// Update the latest version of Go
    Update,
}

fn main() {
    let args = Args::parse();

    match args.command {
        Some(Commands::List) => list_versions(),
        Some(Commands::Update) => update(),
        None => println!("f"),
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

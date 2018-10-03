#[macro_use]
extern crate serde_derive;
extern crate serde;
#[macro_use]
extern crate log;
extern crate log4rs;
#[macro_use]
extern crate structopt;
extern crate git2;
extern crate directories;
#[macro_use]
extern crate lazy_static;

// Crates in my workspace.
extern crate path_encoding;

use structopt::StructOpt;
use git2::Repository;
use std::path::{Path, PathBuf};
use std::env;

// If some of my modules export macros, they must be imported before they are used
// (order matters where macros are concerned).
#[macro_use] mod timer;
mod mru_list;
use mru_list::OafMruList;
mod utils;
mod paths;

// This produces various constants about the build environment which can be referred to using ::PKG_... syntax.
pub mod built_info {
    include!(concat!(env!("OUT_DIR"), "/built.rs"));
}

#[derive(StructOpt, Debug)]
struct Arguments {
    /// Turn off loading of all config files, use compiled-in defaults for all settings.
    #[structopt(long = "no-config")]
    no_config: bool,

    /// Turn off all logging.
    #[structopt(long = "no-logging")]
    no_logging: bool,  

    /// Optional list of directories to open. The directories are expected to be
    /// git repositories. If no directory is passed, the current directory is
    /// assumed.
    #[structopt(parse(from_os_str))]
    directories: Vec<PathBuf>
}

lazy_static! {
    static ref PATHS: paths::WellKnownPaths = { paths::WellKnownPaths::new() };
}

fn main() {
    std::env::set_var("IN_OAF", "1");
    let mut args = Arguments::from_args();

    // Configure logging as early as possible (because, obviously, we want to log
    // in the rest of the initialization phase).
    if !args.no_logging {
        configure_logging(PATHS.logging_config_file());
        log_built_info();
    }

    let mut mru = OafMruList::new(PATHS.mru_file());
    mru.read_from_file();

    if args.directories.is_empty() {
        match env::current_dir() {
            Ok(dir) => args.directories.push(dir),
            Err(e) => warn!("Error getting current directory: {}", e)
        }
    }

    // TODO:
    // Deal with .git/bare repositories.
    // IO functions are actually Results.
    // We really want a Command(OpenRepository(dir)).
    let _x: Vec<_> = args.directories.iter()
        .filter_map(|dir| {
            if !dir.exists() {
                warn!("The directory '{}' does not exist, ignoring.", dir.display());
                return None;
            }

            if !dir.is_dir() {
                warn!("The path '{}' is not a directory.", dir.display());
                return None;
            }

            let dir = match dir.canonicalize() {
                Ok(canon_dir) => canon_dir,
                Err(_) => {
                    warn!("The path '{}' cannot be canonicalized, ignoring.", dir.display());
                    return None;
                }
            };

            match Repository::open(&dir) {
                Ok(repo) => {
                    info!("Successfully opened Git repository at '{}'", dir.display());
                    mru.add_path(&dir);
                    Some(repo)
                },
                Err(e) => {
                    warn!("Failed to initialize repository: {}", e);
                    None
                }
            }
        })
        .collect();

    mru.write_to_file();
}

fn configure_logging(logging_config_file: &Path) {
    if logging_config_file.exists() {
        log4rs::init_file(&logging_config_file, Default::default()).expect("Cannot configure logging.");
        // Use a messge that makes it very easy to find the start of one run in a log file.
        info!("========== Logging initialized using file at {:?} ==========", logging_config_file);
    }
}

fn log_built_info() {
    info!("This is version {}{}, built for {} by {}.",
        built_info::PKG_VERSION,
        built_info::GIT_VERSION.map_or_else(|| "".to_owned(), |v| format!(" (git {})", v)),
        built_info::TARGET,
        built_info::RUSTC_VERSION);

//    trace!("I was built with profile \"{}\", features \"{}\" on {} using {}",
//        built_info::PROFILE,
//        built_info::FEATURES_STR,
//        built_info::BUILT_TIME_UTC,
//        built_info::DEPENDENCIES_STR);
}

#[macro_use] extern crate serde_derive;
extern crate serde;
#[macro_use] extern crate log;
extern crate log4rs;
#[macro_use] extern crate structopt;
extern crate xdg;
extern crate git2;

// Crates in my workspace.
extern crate path_encoding;

use structopt::StructOpt;
use git2::Repository;
use std::path::PathBuf;
use std::env;

// If some of my modules export macros, they must be imported before they are used
// (order matters where macros are concerned).
#[macro_use] mod timer;

mod mru_list;
use mru_list::OafMruList;


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



fn main() {
    std::env::set_var("IN_OAF", "1");

    let mut args = Arguments::from_args();

    let base_dirs = xdg::BaseDirectories::with_prefix(built_info::PKG_NAME)
        .expect("Could not locate xdg base directories, cannot initialize.");

    // Configure logging as early as possible (because, obviously, we want to log
    // in the rest of the initialization phase).
    if !args.no_logging {
        configure_logging(&base_dirs);
        log_built_info();
    }

    let mru_file = base_dirs.place_config_file("mru.txt").unwrap();
    let mut mru = OafMruList::new(&mru_file);
    mru.read_from_file();

    if args.directories.is_empty() {
        match env::current_dir() {
            Ok(dir) => args.directories.push(dir),
            Err(e) => warn!("Error getting current directory: {}", e)
        }
    }

    // TODO: Canonicalize paths, expand ~ and .
    // Directory may not exist.
    // Only save MRU once.
    // Running twice reverses the direction of the MRU list.
    let _x: Vec<_> = args.directories.iter()
        .filter_map(|dir| {
            info!("Attempting to open Git repository at {}", dir.display());
            match Repository::open(&dir) {
                Ok(repo) => {
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

fn configure_logging(base_dirs: &xdg::BaseDirectories) {
    if let Ok(path) = base_dirs.place_config_file("logging.toml") {
        if path.exists() {
            log4rs::init_file(&path, Default::default()).expect("Cannot configure logging.");
            // Use a messge that makes it very easy to find the start of one run in a log file.
            info!("========== Logging initialized using file at {:?} ==========", path);
        }
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

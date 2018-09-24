#[macro_use] extern crate serde_derive;
extern crate serde;
#[macro_use] extern crate log;
extern crate log4rs;
#[macro_use] extern crate structopt;
extern crate xdg;

extern crate path_encoding;

use structopt::StructOpt;
use std::path::PathBuf;

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
}



fn main() {
    std::env::set_var("IN_OAF", "1");

    let args = Arguments::from_args();

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
}

fn configure_logging(base_dirs: &xdg::BaseDirectories) {
    if let Ok(path) = base_dirs.place_config_file("logging.toml") {
        if path.exists() {
            log4rs::init_file(&path, Default::default()).expect("Cannot configure logging.");
            info!("Logging initialized using file at {:?}", path);
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

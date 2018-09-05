#[macro_use] extern crate serde_derive;
extern crate serde;
#[macro_use] extern crate log;
extern crate log4rs;
extern crate xdg;

// This produces various constants about the build environment which can be referred to using ::PKG_... syntax.
pub mod built_info {
    include!(concat!(env!("OUT_DIR"), "/built.rs"));
}

fn main() {
    std::env::set_var("IN_OAF", "1");

    let base_dirs = xdg::BaseDirectories::with_prefix(built_info::PKG_NAME)
        .expect("Could not locate xdg base directories, cannot initialize.");

    // Configure logging as early as possible (because, obviously, we want to log
    // in the rest of the initialization phase).
    configure_logging(&base_dirs);
    log_built_info();
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

#[macro_use] extern crate serde_derive;
extern crate serde;
#[macro_use] extern crate log;
extern crate log4rs;
extern crate xdg;

fn main() {
    std::env::set_var("IN_OAF", "1");

    let base_dirs = xdg::BaseDirectories::with_prefix("oaf")
        .expect("Could not locate xdg base directories, cannot initialize.");

    // Configure logging as early as possible (because, obviously, we want to log
    // in the rest of the initialization phase).
    configure_logging(&base_dirs);
}

fn configure_logging(base_dirs: &xdg::BaseDirectories) {
    if let Ok(path) = base_dirs.place_config_file("logging.toml") {
        if path.exists() {
            log4rs::init_file(&path, Default::default()).expect("Cannot configure logging.");
            info!("Logging initialized using file at {:?}", path);
        }
    }
}

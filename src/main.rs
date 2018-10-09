#[macro_use]
extern crate log;
extern crate log4rs;
#[macro_use]
extern crate structopt;
extern crate git2;
extern crate directories;
#[macro_use]
extern crate lazy_static;
extern crate cursive;

// Crates in my workspace.
extern crate path_encoding;

use structopt::StructOpt;
use std::path::{Path, PathBuf};
use std::env;
use cursive::Cursive;
use cursive::align::HAlign;
use cursive::traits::*;
use cursive::views::{Dialog, DummyView, LinearLayout, TextView, SelectView};
use cursive::theme::BaseColor;
use cursive::theme::Color;
use cursive::theme::Effect;
use cursive::theme::Style;
use cursive::utils::markup::StyledString;
use cursive::utils::span::SpannedString;

// If some of my modules export macros, they must be imported before they are used
// (order matters where macros are concerned).
#[macro_use] mod timer;
mod mru_list;
use mru_list::OafMruList;
mod utils;
mod paths;
mod repositories;
use repositories::Repositories;

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
    /// git repositories. If no directory is passed, the current directory is assumed.
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
    if let Err(e) = mru.read_from_file() {
        warn!("Error reading from MRU file '{}', ignoring. Error = {}", PATHS.mru_file().display(), e);
    }

    // Get all the directories specified (including the current directory if none
    // were specified) and try and open them all. This also validates paths
    // and ascends the directory to tree to try and find a valid repo.
    verify_directories(&mut args.directories);
    let mut repos = Repositories::new(mru);

    for dir in &args.directories {
        let _ = repos.open(dir);
    }

    run_cursive(repos);
}

fn run_cursive(repos: Repositories) {
    // If we managed to open at least 1, display it, else show the opening view.
    let mut repolist = String::new();
    for r in &repos {
        repolist.push_str(&r.path().display().to_string());
        repolist.push('\n');
    }

    let mut siv = Cursive::default();
    siv.add_global_callback('q', |s| s.quit());

    let mut select = SelectView::new()
        .h_align(HAlign::Left);
    select.add_all_str(repolist.lines());

    siv.add_layer(Dialog::around(
        LinearLayout::vertical()
            .child(TextView::new(format!("{} {}", built_info::PKG_NAME, built_info::PKG_VERSION)).h_align(HAlign::Center))
            .child(DummyView.fixed_height(2))
            .child(TextView::new(in_bold("Choose a repository")))
            .child(DummyView.fixed_height(1))
            .child(select.scrollable().scroll_x(true).fixed_width(50)))
    );

    siv.run();
}

fn in_bold(s: &str) -> SpannedString<Style> {
    let mut ss = StyledString::styled(s, Effect::Bold);
    ss.append(StyledString::styled(" please", Style::from(Color::Light(BaseColor::Red))));
    ss
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

/// Verify the directories specified on the command line are valid, existing, non-duplicate etc.
fn verify_directories(directories: &mut Vec<PathBuf>) {
    if directories.is_empty() {
        match env::current_dir() {
            Ok(dir) => directories.push(dir),
            Err(e) => warn!("Error getting current directory, no attempt will be made to open it: {}", e)
        }
    }

    // Use a vec to hold intermediary result rather than a HashSet in order
    // to preserve the order the user specified on the command line.
    let mut result = Vec::new();

    for dir in directories.iter() {
        if !dir.exists() {
            warn!("The directory '{}' does not exist, ignoring.", dir.display());
            continue;
        }

        if !dir.is_dir() {
            warn!("The path '{}' is not a directory, ignoring.", dir.display());
            continue;
        }

        let dir = dir.canonicalize().unwrap_or(dir.to_path_buf());
        if !result.contains(&dir) {
            result.push(dir);
        }
    }

    directories.clear();
    directories.extend(result);
}

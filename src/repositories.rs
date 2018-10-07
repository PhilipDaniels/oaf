use std::path::Path;
use git2::{Repository, RepositoryOpenFlags};
use mru_list::OafMruList;

pub struct Repositories {
    mru: OafMruList,
    repos: Vec<Repository>
}

impl Repositories {
    pub fn new(mru: OafMruList) -> Self {
        Repositories {
            mru: mru,
            repos: Vec::new()
        }
    }

    fn repo_is_open<P>(&self, path: P) -> bool
        where P: AsRef<Path>
    {
        let path = path.as_ref();
        self.repos.iter().any(|repo| repo.path() == path || repo.workdir() == Some(path))
    }

    pub fn open<P>(&mut self, path: P) -> Option<&Repository>
        where P: AsRef<Path>
    {
        // Do not allow a repository to be opened more than once. This is not
        // actually sufficient, because we may search up for the actual path
        // (i.e. we may start oaf in a subdirectory of the repository).
        let path = path.as_ref();
        if self.repo_is_open(path) {
            warn!("The repository at path '{}' is already open, ignoring.", path.display());
            return None;
        }

        match Repository::open_ext(path, RepositoryOpenFlags::empty(), vec![::PATHS.home_dir()]) {
            Ok(repo) => {
                if self.repo_is_open(repo.path()) {
                    warn!("The repository at path '{}' is already open, ignoring.", path.display());
                    return None;
                }

                info!("Successfully opened Git repository at '{}'", repo.path().display());
                self.repos.push(repo);
                self.mru.add_path(path);
                if let Err(e) = self.mru.write_to_file() {
                    warn!("Error writing to MRU file '{}', ignoring. Error = {}", self.mru.filename().display(), e);
                }

                return Some(&self.repos[self.repos.len() - 1]);
            },
            Err(e) => {
                warn!("Failed to initialize repository, ignoring: {}", e);
            }
        }

        None
    }
}


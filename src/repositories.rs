use std::path::Path;
use std::ops::Index;
use std::slice;
use git2::{Repository, RepositoryOpenFlags};
use mru_list::OafMruList;
use paths;

pub trait RepositoryExtensions {
    fn display_name(&self) -> String;
}

impl RepositoryExtensions for Repository {
    fn display_name(&self) -> String {
        let path = self.workdir().unwrap_or(self.path());
        let compressed_path = paths::compress_tilde(path);
        compressed_path.display().to_string()
    }
}

pub struct Repositories {
    pub mru: OafMruList,
    pub repos: Vec<Repository>
}

impl Repositories {
    pub fn new(mru: OafMruList) -> Self {
        Repositories {
            mru: mru,
            repos: Vec::new()
        }
    }

    pub fn iter(&self) -> slice::Iter<Repository> {
        self.repos.iter()
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

impl Index<usize> for Repositories {
    type Output = Repository;

    fn index(&self, index: usize) -> &Self::Output {
        &self.repos[index]
    }
}

impl<'a> IntoIterator for &'a Repositories {
    type Item = &'a Repository;
    type IntoIter = slice::Iter<'a, Repository>;
    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

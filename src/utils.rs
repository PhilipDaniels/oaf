use std::path::{Path, PathBuf};

/// TODO: Can we use Cow here?

/// Canonicalizes the specified path, then replaces the leading path components
/// with '~' if they match the user's home directory. Canonicalization can fail,
/// in which case the original path is returned; it *may* be safe to use. We
/// can at least try.
/// The results of this function are designed to be stored, e.g. in the MRU list.
pub fn path_to_canon<P>(path: P) -> PathBuf
    where P: AsRef<Path>
{
    let path = path.as_ref().to_path_buf();
    let path = path.canonicalize().unwrap_or(path);
    path_compress_tilde(path)
}

/// The inverse of `path_to_canon`. Use this to transform stored paths into
/// 'proper' paths that the program can use.
pub fn path_from_canon<P>(path: P) -> PathBuf
    where P: AsRef<Path>
{
    path_expand_tilde(path)
}

/// Examines the leading components of a path to see if they match the home
/// directory, if they do they are replaced with '~'.
pub fn path_compress_tilde<P>(path: P) -> PathBuf
    where P: AsRef<Path>
{
    path_compress_tilde_impl(path, "/home/phil")
}

/// If the path starts with a tilde, expands it to the user's home directory.
/// Only a tilde at the beginning is considered. Shell constructions such as
/// '~bob' are also not expanded, only '~', '~/' or '~/pics'.
pub fn path_expand_tilde<P>(path: P) -> PathBuf
    where P: AsRef<Path>
{
    path_expand_tilde_impl(path, "/home/phil") 
}

/// Inner helper function to make things testable.
fn path_compress_tilde_impl<P, Q>(path: P, home: Q) -> PathBuf
    where P: AsRef<Path>,
          Q: AsRef<Path>
{
    let home = home.as_ref();
    let path = path.as_ref().to_path_buf();
    if path.starts_with(home) {
        let mut result = PathBuf::from("~");
        for comp in path.components().skip(home.components().count()) {
            result.push(comp);
        }
        return result;
    }

    path
}

/// Inner helper function to make things testable.
fn path_expand_tilde_impl<P, Q>(path: P, home: Q) -> PathBuf
    where P: AsRef<Path>,
          Q: AsRef<Path>
{
    let path = path.as_ref().to_path_buf();

    if path.starts_with("~") {
        let mut result = home.as_ref().to_path_buf();
        for comp in path.components().skip(1) {
            result.push(comp);
        }
        return result;
    }

    path
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn path_compress_tilde_impl_works() {
        let home = "/home/phil";
        assert_eq!(path_compress_tilde_impl("", home), PathBuf::from(""));
        assert_eq!(path_compress_tilde_impl("/", home), PathBuf::from("/"));
        assert_eq!(path_compress_tilde_impl(".", home), PathBuf::from("."));
        assert_eq!(path_compress_tilde_impl("/home/bob", home), PathBuf::from("/home/bob"));
        assert_eq!(path_compress_tilde_impl("/home/philip", home), PathBuf::from("/home/philip"));
        assert_eq!(path_compress_tilde_impl("/home/phil", home), PathBuf::from("~"));
        assert_eq!(path_compress_tilde_impl("/home/phil/", home), PathBuf::from("~/"));
        assert_eq!(path_compress_tilde_impl("/home/phil/pics", home), PathBuf::from("~/pics"));
    }

    #[test]
    fn path_expand_tilde_impl_works() {
        let home = "/home/phil";
        assert_eq!(path_expand_tilde_impl("", home), PathBuf::from(""));
        assert_eq!(path_expand_tilde_impl("/", home), PathBuf::from("/"));
        assert_eq!(path_expand_tilde_impl(".", home), PathBuf::from("."));
        assert_eq!(path_expand_tilde_impl("/home/bob", home), PathBuf::from("/home/bob"));
        assert_eq!(path_expand_tilde_impl("/home/philip", home), PathBuf::from("/home/philip"));
        assert_eq!(path_expand_tilde_impl("~", home), PathBuf::from("/home/phil"));
        assert_eq!(path_expand_tilde_impl("~/", home), PathBuf::from("/home/phil/"));
        assert_eq!(path_expand_tilde_impl("~/pics", home), PathBuf::from("/home/phil/pics"));
    }

}


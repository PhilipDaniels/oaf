
/// TODO: Can we use Cow here?
/// Tried to do it, but run into lifetime issues with the AsRef...Cow in expand_tilde_impl etc.

mod path {
    use std::path::{Path, PathBuf};

    /// Canonicalizes the specified path, then replaces the leading path components
    /// with '~' if they match the user's home directory. Canonicalization can fail,
    /// in which case the original path is returned; it *may* be safe to use. We
    /// can at least try.
    /// The results of this function are designed to be stored, e.g. in the MRU list.
    pub fn to_canon<P>(path: P) -> PathBuf
        where P: AsRef<Path>
    {
        let path = path.as_ref().to_path_buf();
        let path = path.canonicalize().unwrap_or(path);
        compress_tilde(path)
    }

    /// The inverse of `path_to_canon`. Use this to transform stored paths into
    /// 'proper' paths that the program can use.
    pub fn from_canon<P>(path: P) -> PathBuf
        where P: AsRef<Path>
    {
        expand_tilde(path)
    }

    /// Examines the leading components of a path to see if they match the home
    /// directory, if they do they are replaced with '~'.
    pub fn compress_tilde<P>(path: P) -> PathBuf
        where P: AsRef<Path>
    {
        compress_tilde_impl(path, "/home/phil")
    }

    /// If the path starts with a tilde, expands it to the user's home directory.
    /// Only a tilde at the beginning is considered. Shell constructions such as
    /// '~bob' are also not expanded, only '~', '~/' or '~/pics'.
    pub fn expand_tilde<P>(path: P) -> PathBuf
        where P: AsRef<Path>
    {
        expand_tilde_impl(path, "/home/phil") 
    }

    /// Inner helper function to make things testable.
    fn compress_tilde_impl<P, Q>(path: P, home: Q) -> PathBuf
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
    fn expand_tilde_impl<P, Q>(path: P, home: Q) -> PathBuf
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

    /*
    // This almost works, but expand_tilde_impl2 can't be called from a test ,easily.
    use std::borrow::Cow;

    fn expand_tilde_impl2<'a, P, Q>(path: &'a P, home: Q) -> Cow<'a, Path>
        where P: AsRef<Path>,
              Q: AsRef<Path>
    {
        let path = path.as_ref();

        if path.starts_with("~") {
            let mut result = home.as_ref().to_path_buf();
            for comp in path.components().skip(1) {
                result.push(comp);
            }
            return Cow::Owned(result);
        }

        Cow::Borrowed(path)
    }

    fn exp2<'a, P, Q>(path: &'a P, home: Q) -> Cow<'a, Path>
        where P: AsRef<Path>,
              Q: AsRef<Path>
    {
        Cow::Borrowed(path.as_ref())
    }
    */

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn compress_tilde_impl_works() {
            let home = "/home/phil";
            assert_eq!(compress_tilde_impl("", home), PathBuf::from(""));
            assert_eq!(compress_tilde_impl("/", home), PathBuf::from("/"));
            assert_eq!(compress_tilde_impl(".", home), PathBuf::from("."));
            assert_eq!(compress_tilde_impl("/home/bob", home), PathBuf::from("/home/bob"));
            assert_eq!(compress_tilde_impl("/home/philip", home), PathBuf::from("/home/philip"));
            assert_eq!(compress_tilde_impl("/home/phil", home), PathBuf::from("~"));
            assert_eq!(compress_tilde_impl("/home/phil/", home), PathBuf::from("~/"));
            assert_eq!(compress_tilde_impl("/home/phil/pics", home), PathBuf::from("~/pics"));
        }

        #[test]
        fn expand_tilde_impl_works() {
            let home = "/home/phil";
            assert_eq!(expand_tilde_impl("", home), PathBuf::from(""));
            assert_eq!(expand_tilde_impl("/", home), PathBuf::from("/"));
            assert_eq!(expand_tilde_impl(".", home), PathBuf::from("."));
            assert_eq!(expand_tilde_impl("/home/bob", home), PathBuf::from("/home/bob"));
            assert_eq!(expand_tilde_impl("/home/philip", home), PathBuf::from("/home/philip"));
            assert_eq!(expand_tilde_impl("~", home), PathBuf::from("/home/phil"));
            assert_eq!(expand_tilde_impl("~/", home), PathBuf::from("/home/phil/"));
            assert_eq!(expand_tilde_impl("~/pics", home), PathBuf::from("/home/phil/pics"));
        }
    }
}


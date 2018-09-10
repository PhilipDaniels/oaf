use std::borrow::Cow;
use std::path::{Path, PathBuf};

//type PathString<'a> = Cow<'a, str>;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PathString<'a> {
    Printable(&'a str),
    Encoded(String)
}

use PathString::{Printable, Encoded};

fn should_be_encoded(s: &str) -> bool
{
    true
}

fn encode(s: &str) -> String 
{
    s.to_string()
}

fn path_to_path_string<'a>(p: &'a Path) -> PathString<'a> {
    match p.to_str() {
        Some(s) => { Printable(s) },
        None => { Encoded("".to_string()) }
    }
}



impl<'a> From<&'a Path> for PathString<'a> {
    fn from(p: &'a Path) -> Self {
        path_to_path_string(p)
    }
}

impl<'a> From<&'a PathBuf> for PathString<'a> {
    fn from(pb: &'a PathBuf) -> Self {
        path_to_path_string(&pb)
    }
}

impl<'a> From<String> for PathString<'a> {
    fn from(s: String) -> Self {
        path_to_path_string(&PathBuf::from(s))
    }
}

#[cfg(test)]
mod tests {
    use std::path::{Path, PathBuf};
    use super::*;
    use super::PathString::*;

    #[test]
    fn from_path_converts() {
        let pb = PathBuf::from("hello");
        let p = pb.as_path();
        let ps = PathString::from(p);
        assert_eq!(Printable("hello"), ps);
    }

    #[test]
    fn from_pathbuf_converts() {
        let pb = PathBuf::from("hello");
        let ps = PathString::from(&pb);
        assert_eq!(Printable("hello"), ps);
    }

    #[test]
    fn from_string_converts() {
        let s = "hello".to_string();
        let ps = PathString::from(s);
        assert_eq!(Printable("hello"), ps);
    }

}

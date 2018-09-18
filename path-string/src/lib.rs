extern crate base64;

use std::borrow::Cow;
use std::path::{Path, PathBuf};
use std::ffi::{OsStr, OsString};

/// Drive letters must be A-Z, single character only. Therefore this
/// always represents an invalid path (note also that ':' is illegal in Windows paths).
#[cfg(windows)]
const PREFIX: &str = "b64:\\_";

/// On Unix, filenames can contain any byte except '\0' and '/', which makes formulating
/// an impossible filename very difficult (since we can't use a zero-byte in a printable
/// string and '/' is the usual directory separator). You can even use filenames such as
/// '/../../b64' in the shell and File::create() and they work ok because the '..' file
/// in the root directory is a link back to the root directory making it impossible to
/// 'escape' the filesystem (very clever, Unix guys).
/// However, you cannot have a file under '/dev/null' because it is defined as a file
/// in POSIX! http://pubs.opengroup.org/onlinepubs/9699919799/basedefs/V1_chap10.html
/// Therefore any path beginning with '/dev/null' will be an invalid path.
/// Baldrick levels of cunning going on here.
#[cfg(not(windows))]
const PREFIX: &str = "/dev/null/b64_";


/// Even if a Path can be converted to a valid UTF-8 string we still might want
/// to encode it: it's difficult to write filenames with newlines or '\b' in a sensible
/// manner, for example.
fn should_be_encoded(s: &str) -> bool
{
    s.chars().any(|c| c.is_control())
}

/// A small wrapper around the 'encode' call to the base64 library to ensure
/// we do it the same way every time.
fn encode_bytes(bytes: &[u8]) -> String {
    let mut b64 = PREFIX.to_string();
    base64::encode_config_buf(bytes, base64::STANDARD, &mut b64);
    b64
}

/// A small wrapper around the 'decode' call to the base64 library to ensure
/// we do it the same way every time.
fn decode_bytes(encoded_str: &str) -> Vec<u8> {
    let encoded_bytes = &encoded_str[PREFIX.len()..];
    let bytes = base64::decode_config(encoded_bytes, base64::STANDARD)
        .expect("FIXME: The conversion might fail because a user might edit the encoded data incorrectly.");
    bytes
}

#[cfg(unix)]
fn encode_os(s: &OsStr) -> String {
    use std::os::unix::ffi::OsStrExt;
    
    let bytes = s.as_bytes();
    encode_bytes(bytes)
}

#[cfg(windows)]
fn encode_os(s: &OsStr) -> String {
    use std::os::windows::ffi::OsStrExt;

    let wide_chars = s.encode_wide().collect::<Vec<_>>();
    let bytes = u16_slice_to_byte_array(&wide_chars);
    encode_bytes(&bytes)
}

#[cfg(unix)]
fn decode_os(bytes: Vec<u8>) -> OsString {
    use std::os::unix::ffi::OsStringExt;
    
    OsString::from_vec(bytes)
}

#[cfg(windows)]
fn decode_os(bytes: Vec<u8>) -> OsString {
    use std::os::windows::ffi::OsStringExt;

    let mut wide_chars = Vec::with_capacity(bytes.len() / 2);
    let mut i = 0;
    while i < bytes.len() - 1 {
        let wide = bytes_to_u16(bytes[i], bytes[i + 1]);
        wide_chars.push(wide);
        i += 2;
    }

    OsString::from_wide(&wide_chars)
}

#[cfg(windows)]
fn bytes_to_u16(b1: u8, b2: u8) -> u16 {
    let result = ((b1 as u16) << 8) + b2 as u16;
    result
}

#[cfg(windows)]
fn u16_to_bytes(value: u16) -> [u8; 2] {
    let b1: u8 = ((value >> 8) & 0xff) as u8;
    let b2: u8 = (value & 0xff) as u8;
    return [b1, b2]
}

#[cfg(windows)]
fn u16_slice_to_byte_array(wides: &[u16]) -> Vec<u8> {
    let mut bytes = Vec::with_capacity(wides.len() * 2);
    for &wc in wides {
        let a = u16_to_bytes(wc);
        bytes.push(a[0]);
        bytes.push(a[1]);
    }
    bytes
}

pub fn path_to_path_string<P>(p: &P) -> Cow<str>
    where P: AsRef<Path>
{
    let p = p.as_ref();

    if let Some(s) = p.to_str() {
        if !should_be_encoded(s) {
            return Cow::Borrowed(s);
        }
    }

    Cow::Owned(encode_os(p.as_os_str()))
}

pub fn path_string_to_path_buf<S>(s: S) -> PathBuf
    where S: AsRef<str>
{
    let s = s.as_ref();
    if s.starts_with(PREFIX) {
        let bytes = decode_bytes(s);
        let os_str = decode_os(bytes);
        PathBuf::from(os_str)
    } else {
        PathBuf::from(s)
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;
    use super::*;
    //use super::PathString::*;

    // On Unix, only the '\0' and '/' are invalid in filenames but any
    // other byte sequence is valid.
    //
    // For UTF-8 these bytes are forbidden *anywhere* in the byte sequence
    // (see https://en.wikipedia.org/wiki/UTF-8#Codepage_layout):
    //
    //     0xc0 (192), 0xc1 (193)
    //     0xf5 (245) to 0xff (255) inclusive
    //
    // Therefore sequence including such bytes will be valid paths but not a valid Rust String.
    #[cfg(unix)]
    const INVALID_UTF8_BYTE_SEQUENCE: [u8; 6] = [0x48, 0x64, 0x64, 0x6c, 0x6f, 0xc0];

    // On Windows, the following characters are invalid in filenames according to
    // https://docs.microsoft.com/en-us/windows/desktop/fileio/naming-a-file
    //
    //     < (less than)
    //     > (greater than)
    //     : (colon - sometimes works, but is actually NTFS Alternate Data Streams)
    //     " (double quote)
    //     / (forward slash)
    //     \ (backslash)
    //     | (vertical bar or pipe)
    //     ? (question mark)
    //     * (asterisk)
    //
    // However, note that these are all printable characters.
    // Windows also bans bytes 0..31 (the ASCII control characters) - so no
    // tabs, bells or newlines in filenames.
    //
    // On Windows, paths are UTF-16-le, not UTF-8. So we need to make a UTF-16
    // string that is not a valid UTF-8 string.
    // This is an invalid byte sequence according to http://unicode.org/faq/utf_bom.html#utf16-7
    // path.display() works, and prints "Hello\u{d800}H", but path.to_str() will return None.
    // Windows will accept this as a valid path, but it is not a valid Rust String.
    #[cfg(windows)]
    const INVALID_UTF16_BYTE_SEQUENCE: [u16; 7] = [0x48, 0x65, 0x6c, 0x6c, 0x6f, 0xd800, 0x48]; // "Hello\u{d800}H"

    #[test]
    fn path_to_path_string_for_utf8_which_does_not_need_encoding() {
        let pb = PathBuf::new();
        let s = path_to_path_string(&pb);
        assert_eq!(s, "", "Empty paths should be empty strings.");
        let pb2 = path_string_to_path_buf(&s);
        assert_eq!(pb2, pb, "Empty paths should be round-trippable.");

        let pb = PathBuf::from("hello");
        let s = path_to_path_string(&pb);
        assert_eq!(s, "hello", "Valid UTF-8 paths without control chars should be encoded as-is.");
        let pb2 = path_string_to_path_buf(&s);
        assert_eq!(pb2, pb, "Valid UTF-8 paths without control chars should be round-trippable.");
    }

    #[cfg(unix)]
    #[test]
    fn path_to_path_string_for_valid_utf8_needing_unix_encoding() {
        // There are separate Unix and Windows tests because on Windows a valid UTF-8 string
        // will still be treated as UTF-16 wide chars by the time it is encoded.
        let pb = PathBuf::from("hello\tworld");
        let s = path_to_path_string(&pb);
        assert_eq!(s, format!("{}aGVsbG8Jd29ybGQ=", PREFIX),
            "Paths with control characters in them should be base-64 encoded.");
        let pb2 = path_string_to_path_buf(&s);
        assert_eq!(pb2, pb, "Paths with control characters in them should be round-trippable.");
    }

    #[cfg(windows)]
    #[test]
    fn path_to_path_string_for_valid_utf8_needing_windows_encoding() {
        // There are separate Unix and Windows tests because on Windows a valid UTF-8 string
        // will still be treated as UTF-16 wide chars by the time it is encoded.
        let pb = PathBuf::from("hello\tworld");
        let s = path_to_path_string(&pb);
        assert_eq!(s, format!("{}AGgAZQBsAGwAbwAJAHcAbwByAGwAZA==", PREFIX),
            "Paths with control characters in them should be base-64 encoded.");
        let pb2 = path_string_to_path_buf(&s);
        assert_eq!(pb2, pb, "Paths with control characters in them should be round-trippable.");
    }

    #[cfg(unix)]
    #[test]
    fn path_to_path_string_for_invalid_utf8() {
        let os = decode_os(INVALID_UTF8_BYTE_SEQUENCE.to_vec());
        let pb = PathBuf::from(os);
        let s = path_to_path_string(&pb);
        assert_eq!(s, format!("{}SGRkbG/A", PREFIX),
            "Invalid UTF-8 byte sequences should be base-64 encoded.");
        let pb2 = path_string_to_path_buf(&s);
        assert_eq!(pb2, pb, "Invalid UTF-8 byte sequences should be round-trippable.");
    }

    #[cfg(windows)]
    #[test]
    fn path_to_path_string_for_invalid_utf16() {
        let bytes = u16_slice_to_byte_array(&INVALID_UTF16_BYTE_SEQUENCE);
        let os = decode_os(bytes);
        let pb = PathBuf::from(os);
        let s = path_to_path_string(&pb);
        assert_eq!(s, format!("{}AEgAZQBsAGwAb9gAAEg=", PREFIX),
            "Invalid UTF-16 byte sequences should be base-64 encoded.");
        let pb2 = path_string_to_path_buf(&s);
        assert_eq!(pb2, pb, "Invalid UTF-16 byte sequences should be round-trippable.");
    }
}





/*
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PathString<'a> {
    Printable(&'a str),
    Encoded(String)
}

use PathString::{Printable, Encoded};

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
*/

/*
#[cfg(test)]
mod tests {
    use std::path::{Path, PathBuf};
    use super::*;
    //use super::PathString::*;

   
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
*/

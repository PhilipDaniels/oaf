extern crate path_string;

use std::path::{Path, PathBuf};
use std::fs;
use std::ffi::OsString;

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

#[cfg(unix)]
fn value_to_bytes(i: u16) -> Vec<u8> {
    vec![i as u8]
}

#[cfg(windows)]
fn value_to_bytes(i: u16) -> Vec<u8> {
    let b1 = ((i >> 8) & 0xff) as u8;
    let b2 = (i & 0xff) as u8;
    vec![b1, b2]
}

fn value_to_pathbuf(dir: &Path, i: u16) -> PathBuf {
    let bytes = value_to_bytes(i);
    let os = decode_os(bytes);
    let mut p = dir.to_path_buf();
    let filename = PathBuf::from(&os);
    p.push(filename);
    p
}

fn create_files(min: u16, max: u16) {
    let dir = Path::new("awkward");
    if !dir.exists() {
        fs::create_dir(&dir).unwrap();
    }

    for i in min..=max {
        let filename = value_to_pathbuf(&dir, i);
        match fs::File::create(filename) {
            Err(_e) => println!("Could not create file for {}", i),
            Ok(_) => {},
        }
    }
}

#[cfg(unix)]
fn main() {
    create_files(1, 255);
}

#[cfg(windows)]
fn main() {
    create_files(1, std::u16::MAX);
}


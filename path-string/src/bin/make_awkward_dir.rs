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
fn u16_to_bytes(value: u16) -> [u8; 2] {
    let b1: u8 = ((value >> 8) & 0xff) as u8;
    let b2: u8 = (value & 0xff) as u8;
    return [b1, b2]
}


#[cfg(unix)]
fn value_to_bytes(i: i32) -> Vec<u8> {
    let mut bytes = vec![];
    bytes.push(i as u8);
    bytes
}

#[cfg(windows)]
fn value_to_bytes(i: i32) -> Vec<u8> {
    let mut bytes = vec![];
    let pair = u16_to_bytes(i as u16);
    bytes.push(pair[0], pair[1]);
    bytes
}

fn value_to_pathbuf(dir: &Path, i: i32) -> PathBuf {
    let bytes = value_to_bytes(i);
    let os = decode_os(bytes);
    let mut p = dir.to_path_buf();
    let filename = PathBuf::from(&os);
    p.push(filename);
    p
}


fn create_files(min: i32, max: i32) {
    let dir = Path::new("awkward");
    if !dir.exists() {
        fs::create_dir(&dir).unwrap();
    }

    for i in min..max {
        println!("Creating file for value {}", i);
        let filename = value_to_pathbuf(&dir, i);
        match fs::File::create(filename) {
            Err(_e) => println!("Could not create file for {}", i),
            Ok(_) => {},
        }
    }
}

#[cfg(unix)]
fn main() {
    create_files(1, 256);
}

#[cfg(windows)]
fn main() {
    create_files(1, std::u16::MAX);
}


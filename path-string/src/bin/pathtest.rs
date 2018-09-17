extern crate path_string;
extern crate separator;

use std::io;
use std::io::prelude::*;
use std::ffi::OsString;
use std::os::unix::ffi::OsStringExt;
use std::path::PathBuf;
use std::borrow::Cow;
use separator::Separatable;

struct PathRequiringEncoding {
    original_path: PathBuf,
    encoded_form: String,
}

// Recommended usage     : find DIR -print0 | target/release/pathtest
// Scan your whole drive : sudo find / -print0 | target/release/pathtest

fn main() {
    println!("Counting paths according to base64 encoding needs.");

    let stdin = io::stdin();
    let mut total_paths = 0;
    let mut num_borrowed = 0;
    let mut num_encoded = 0;
    let mut paths_needing_encoding = Vec::new();

    for p in stdin.lock().split(0).take(2_000_000) {
        let path_as_bytes = p.unwrap();
        //println!("Reading path_as_bytes = {:?}", path_as_bytes);
        let os = OsString::from_vec(path_as_bytes);
        //println!("Reading os = {:?}", os);
        let pb = PathBuf::from(os);
        let coded = path_string::path_to_path_string(&pb);

        total_paths += 1;

        match coded {
            Cow::Borrowed(_s) => num_borrowed += 1,
            Cow::Owned(encoded_path) => {
                num_encoded += 1;
                let round_tripped = path_string::path_string_to_path_buf(&encoded_path);
                println!("  Found a path requiring encoding: {:?}", pb);
                println!("  Encoded form is                : {:?}", encoded_path);
                println!("  Round tripped back to path is  : {:?}", round_tripped);
                assert_eq!(pb, round_tripped);

                // Accumulate these for printing them all at the end.
                let pre = PathRequiringEncoding { original_path: pb.clone(), encoded_form: encoded_path };
                paths_needing_encoding.push(pre);
            }
        }

        if total_paths % 1000 == 0 {
            print_msg(num_borrowed, num_encoded);

        }
    }
    
    print_msg(num_borrowed, num_encoded);
    println!("Counting complete.");

    if num_encoded > 0 {
        let percentage = 100f64 * num_encoded as f64 / total_paths as f64;

        println!("\n{} out of {} paths needed encoding (and were successfully round-tripped).",
                 num_encoded, total_paths.separated_string());
        println!("This represents {}% of the total path count.\n", percentage);

        for pre in paths_needing_encoding {
            println!("Original path: {:?}", pre.original_path);
            println!("Encoded form : {:?}", pre.encoded_form);
        }
    }
}

fn print_msg(num_borrowed: usize, num_encoded: usize) {
    println!("num_borrowed = {}, num_encoded = {}",
             num_borrowed, num_encoded);
}


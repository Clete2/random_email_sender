use glob::glob;
use rand::Rng;
use std::io::prelude::Read;
use std::{fs::File, path::PathBuf};
use xz2::read::XzDecoder;

use crate::email_format::Email;

pub fn get_random_email(glob_path: &str) -> Email {
    let paths = glob(glob_path).unwrap().filter_map(Result::ok);
    let paths: Vec<PathBuf> = paths.collect();

    let random_file = get_random_item(&paths);
    let file = read_lzma_file_to_string(random_file);

    let emails: Vec<Email> = serde_json::from_str(file.as_str()).unwrap();

    get_random_item(&emails).clone()
}

fn read_lzma_file_to_string(file: &PathBuf) -> String {
    let file = File::open(file).unwrap();
    let mut decompressor = XzDecoder::new(file);
    let mut file_as_string = String::new();
    decompressor.read_to_string(&mut file_as_string).unwrap();

    file_as_string
}

fn get_random_item<T>(collection: &Vec<T>) -> &T {
    let index = rand::thread_rng().gen_range(0..collection.len() - 1);

    collection.get(index).unwrap()
}

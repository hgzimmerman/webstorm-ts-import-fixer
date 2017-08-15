extern crate regex;

use regex::Regex;

use std::fs::{File, DirEntry};
use std::fs;
use std::io::prelude::*;
use std::io::BufReader;
use std::path::Path;
use std::io;
use std::fs::OpenOptions;
use std::io::{Write};

fn main() {

    let starting_path: &Path = Path::new(".");
    let _ = visit_dirs(starting_path, &fix_file);

}


fn visit_dirs(dir: &Path, cb: &Fn(&DirEntry)) -> io::Result<()> {
    if dir.is_dir() {
        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_dir() {
                visit_dirs(&path, cb)?;
            } else if is_ts_file(&entry) {
                cb(&entry);
            }
        }
    }
    Ok(())
}

fn is_ts_file(entry: &DirEntry) -> bool {
    //We don't want to clobber the node_modules directory
    let file_path = entry.file_name().to_str().unwrap().to_string();
    if file_path.contains("node_modules") || file_path.contains("typings") {
        return false
    }
    if file_path.contains(".ts") {
        println!("{}", file_path);
        return true
    }
    false
}

fn fix_file(dir_entry : &DirEntry ) {
    let filename: String = dir_entry.path().to_str().expect("couldn't get file name").to_string();


    let mut contents = String::new();
    let mut new_contents = String::new();
    let mut newer_contents = String::new();
    {
        let file: File = OpenOptions::new().read(true).open(&filename).unwrap();

        let mut buf_reader = BufReader::new(&file);
        match buf_reader.read_to_string(&mut contents) {
            Ok(_) => {},
            Err(e) => println!("couldn't read the file {} because {}", filename, e)
        }

        // fixes the spacing between the imported namespace
        let no_space_import_regex: Regex = Regex::new(r"import \{(?P<y>\S.*\S)\} from (?P<i>.*)").unwrap();
        let lines = contents.split("\n");
        for line in lines {
            let replacement = no_space_import_regex.replace_all(line, "import { $y } from $i");
            let str_with_newline = format!("{}{}", &replacement, "\n");
            new_contents.push_str(str_with_newline.as_str());
        }
        let _ = new_contents.pop();

        // fixes the quotation marks to be '' instead of ""
        let replace_quotes_regex: Regex = Regex::new( r###"import (?P<c>.*) from "(?P<q>.*)".*"### ).unwrap();
        let lines = new_contents.split("\n");
        for line in lines {
            let replacement = replace_quotes_regex.replace_all(line, "import $c from '$q'");
            let str_with_newline = format!("{}{}", &replacement, "\n");
            newer_contents.push_str(str_with_newline.as_str());
        }
        let _ = newer_contents.pop();
    }
    let mut file: File = OpenOptions::new().write(true).truncate(true).open(&filename).unwrap();

    let buffer = &newer_contents.into_bytes()[..];
    let _ = file.write_all(buffer);

}

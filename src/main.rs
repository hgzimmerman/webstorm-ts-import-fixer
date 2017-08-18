extern crate regex;
extern crate clap;

use clap::{Arg, App};
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


    let matches = App::new("webstorm-ts-import-fixer")
        .version("0.1.0")
        .author("Henry Zimmerman")
        .about("Fixes imports that webstorm created. It will recursively search from the directory where it was called, altering all files that it detects as '.ts' files if needed, in valid directories.")
        .arg(Arg::with_name("ignore")
            .short("i")
            .long("ignore")
            .value_name("REGEX")
            .help("A regex of all file paths to ignore. By default, this will ignore node_modules/ and typings/ directories.")
            .takes_value(true)
            .required(false)
            )
        .get_matches();


    let starting_path: &Path = Path::new(".");

    let regex: Option<Regex> = match  matches.value_of("ignore") {
        Some(regex_string) => {
            match Regex::new(regex_string) {
                Ok(valid_regex) => Some(valid_regex),
                Err(e) => panic!("{}", e)
            }
        }
        None => None
    };

    let _ = visit_dirs(starting_path, &fix_file, &regex);

}


fn visit_dirs(dir: &Path, cb: &Fn(&DirEntry), optional_regex: &Option<Regex>) -> io::Result<()> {
    if dir.is_dir() {
        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_dir() {
                visit_dirs(&path, cb, optional_regex)?;
            } else if is_ts_file(&entry, optional_regex) {
                cb(&entry);
            }
        }
    }
    Ok(())
}

fn is_ts_file(entry: &DirEntry, optional_regex: &Option<Regex>) -> bool {
    //We don't want to clobber the node_modules directory
    let file_path = entry.path().to_str().unwrap().to_string();

    // If the regex is provided, filter based on the regex, otherwise, fall back to ignoring
    // node_modules and typings
    match *optional_regex {
        Some(ref regex) => {
            if regex.is_match(file_path.as_str()) {
                return false
            }

        },
        None => {
            if file_path.contains("node_modules/") || file_path.contains("typings/") {
                return false
            }
        }
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
    {
        let file: File = OpenOptions::new().read(true).open(&filename).unwrap();

        let mut buf_reader = BufReader::new(&file);
        match buf_reader.read_to_string(&mut contents) {
            Ok(_) => {},
            Err(e) => println!("couldn't read the file {} because {}", filename, e)
        }

        new_contents = fix_namespace_spacing(contents);
        new_contents = fix_quotes(new_contents);
        new_contents = promote_imports_out_of_logging(new_contents);

      }
    let mut file: File = OpenOptions::new().write(true).truncate(true).open(&filename).unwrap();

    let buffer: &[u8] = &new_contents.into_bytes()[..];
    let _ = file.write_all(buffer);

}

fn fix_namespace_spacing(file_contents: String) -> String {
    // fixes the spacing between the imported namespace
    let mut new_contents = String::new();
    let no_space_import_regex: Regex = Regex::new(r"import \{(?P<y>\S.*\S)\} from (?P<i>.*)").unwrap();
    let lines = file_contents.split("\n");
    for line in lines {
        let replacement = no_space_import_regex.replace_all(line, "import { $y } from $i");
        let str_with_newline = format!("{}{}", &replacement, "\n");
        new_contents.push_str(str_with_newline.as_str());
    }
    let _ = new_contents.pop();// pop off the last \n
    new_contents
}

fn fix_quotes(file_contents: String) -> String {
      // fixes the quotation marks to be '' instead of ""
    let mut new_contents = String::new();
    let replace_quotes_regex: Regex = Regex::new( r###"import (?P<c>.*) from "(?P<q>.*)".*"### ).unwrap();
    let lines = file_contents.split("\n");
    for line in lines {
        let replacement = replace_quotes_regex.replace_all(line, "import $c from '$q';");
        let str_with_newline = format!("{}{}", &replacement, "\n");
        new_contents.push_str(str_with_newline.as_str());
    }
    let _ = new_contents.pop();
    new_contents
}

fn promote_imports_out_of_logging(file_contents: String) -> String {
    let mut new_contents = String::new();
    let mut has_found_logger: bool = false;
    let any_import_regex: Regex = Regex::new(r"^import.*from.*;").unwrap();
    let logging_import_regex: Regex = Regex::new(r"import \{.*\}.*from.*logger/xgLog2.*").unwrap();
    let lines = file_contents.split("\n");
    for line in lines {
        if has_found_logger {
            if any_import_regex.is_match(line) {
                // promote the import
                new_contents = [line, "\n", new_contents.as_str()].concat();
            } else {
                new_contents.push_str([line, "\n"].concat().as_str());
            }
        } else {
            if logging_import_regex.is_match(line) {
                has_found_logger = true;
            }
            new_contents.push_str([line, "\n"].concat().as_str());

        }
    }
    let _ = new_contents.pop();
    new_contents
}

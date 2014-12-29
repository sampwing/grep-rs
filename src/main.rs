#![feature(phase)]
#[phase(plugin, link)] extern crate log;
#[phase(plugin)] extern crate regex_macros;
extern crate getopts;
extern crate regex;

use std::io::File;
use std::io::BufferedReader;
use std::io::fs;
use std::io::fs::PathExtensions;
use std::os;

use std::clone;


use getopts::{optflag, getopts, OptGroup};
use regex::Regex;

struct LineContent{
    line: uint, 
    content: String,
}

fn search_path(path: &Path, re: &Regex, line_numbers: bool) {
    let mut file = BufferedReader::new(File::open(path));
    let line_matches: Vec<LineContent> = file 
        // get the Lines from the file
        .lines()
        // get only the ok Results
        .filter(|x| x.is_ok())
        // unwrap the Optionals for consumption
        .map(|x| x.unwrap())
        // enumerate the results
        .enumerate()
        // map to a line match object
        .map(|(idx, line)| LineContent{line: idx + 1, content: line.to_string()})
        // filter out lines which do not match the regex
        .filter(|line_content| re.is_match(line_content.content.as_slice()))
        // collect into the linematch vector
        .collect();

    if line_numbers {
        for line_match in line_matches.iter() {
            print!("{}:{}:{}", path.display(), line_match.line, line_match.content)
        }
    } else {
         for line_match in line_matches.iter() {
            print!("{}:{}", path.display(), line_match.content)
        }
    }
}


fn print_usage(program: &str, _opts: &[OptGroup]) {
    println!("Usage: {} [options]", program);
    println!("-n --line-number\n\tPrint Line Numbers");
    println!("-h --help\n\tUsage");
}

fn main() {

    let args: Vec<String> = os::args();
    let program = args[0].clone();
    let opts = &[
        optflag("n", "line-number", "display line number"),
        optflag("h", "help", "print this help menu"),
        optflag("r", "recursive", "recursively walk paths")
    ];

    let matches = match getopts(args.tail(), opts) {
        Ok(m) => m,
        Err(err) => panic!(err.to_string())
    };

    // determine if help requested
    if matches.opt_present("h") {
        print_usage(program.as_slice(), opts);
        return;
    }

    // get the pattern and path
    let (input_pattern, input_path) = if matches.free.len() == 2 {
        (matches.free[0].clone(), matches.free[1].clone())
    } else {
        print_usage(program.as_slice(), opts);
        return;
    };

    let re = match Regex::new(input_pattern.as_slice()) {
        Ok(re) => re,
        Err(err) => {
            debug!("{}", err);
            error!("Invalid search pattern specified.");
            return
        }
    };
    
    let path = Path::new(input_path);
    if !path.exists() {
        error!("Invalid Path Specified.");
        return;
    }
    let line_numbers = matches.opt_present("n");
    if matches.opt_present("r") {
        if !path.is_file(){
            let paths = match fs::walk_dir(&path) {
                Ok(paths) => paths.filter(|path| path.is_file()).collect::<Vec<Path>>(),
                Err(err) => {
                    debug!("{}", err);
                    error!("Unable to walk paths recursively.");
                    return
                }
            };
            for path in paths.iter() {
                //spawn(move || {
                //    search_path(&path.clone(), &re.clone(), line_numbers);
                //});
                search_path(path, &re, line_numbers);

            }
        } else {
            search_path(&path, &re, line_numbers);
        }
    } else {
        if !path.is_file() {
            error!("Path is not a file.");
            return;
        }
        search_path(&path, &re, line_numbers);
    }
}

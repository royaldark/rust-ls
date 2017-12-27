extern crate clap;
use self::clap::{Arg, ArgMatches, App};

use format;

#[derive(Debug)]
pub struct LsOptions {
    pub paths: Vec<String>,
    pub format: format::Format
}

fn parse_opts<'a>() -> ArgMatches<'a> {
    App::new("ls")
        .version("1.0")
        .author("Joe Einertson")
        .about("you know, it's ls")
        .arg(Arg::with_name("LONG")
            .short("l")
            .help("Sets a custom config file"))
        .arg(Arg::with_name("PATHS")
            .help("Sets the input file to use")
            .multiple(true)
            .default_value("."))
        .get_matches()
}

pub fn parse_cli() -> LsOptions {
    let matches = parse_opts();

    LsOptions {
        paths: matches.values_of("PATHS").unwrap_or_default().map(String::from).collect(),
        format: if matches.is_present("LONG") {
            format::Format::Long
         } else {
            format::Format::Short
         }
    }
}

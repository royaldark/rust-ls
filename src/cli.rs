extern crate clap;
use self::clap::{Arg, ArgMatches, App};

use format;

#[derive(Debug, PartialEq)]
pub enum OutputFilter {
    All,
    AlmostAll,
    Visible
}

#[derive(Debug)]
pub struct LsOptions {
    pub paths: Vec<String>,
    pub output_format: format::OutputFormat,
    pub size_format: format::SizeFormat,
    pub color: format::ColorOption,
    pub output_filter: OutputFilter,
    pub show_dir_headers: bool,
    pub list_dir_contents: bool
}

fn parse_opts<'a>() -> ArgMatches<'a> {
    App::new("ls")
        .version("1.0")
        .author("Joe Einertson")
        .about("you know, it's ls")
        .arg(Arg::with_name("ALL")
            .short("a")
            .help("Show all files, including hidden files"))
        .arg(Arg::with_name("ALMOST_ALL")
            .short("A")
            .help("Like -a, but excludes . and .."))
        .arg(Arg::with_name("DIR_NAME_ONLY")
            .short("d")
            .help("Show directories themselves, not their contents"))
        .arg(Arg::with_name("GROUP_LONG")
            .short("g")
            .help("Like -l, but only shows group, not owner"))
        .arg(Arg::with_name("HUMAN_SIZES")
            .short("h")
            .help("Use human-readable sizes based on kibibytes (1024 bytes)"))
        .arg(Arg::with_name("LONG")
            .short("l")
            .help("Long form output"))
        .arg(Arg::with_name("COLOR")
            .long("color")
            .default_value("always")
            .possible_values(&["always", "never", "auto"])
            .help("Color options"))
        .arg(Arg::with_name("SI_SIZES")
            .long("si")
            .help("Like -h, but uses SI sizes (1000 bytes)"))
        .arg(Arg::with_name("PATHS")
            .help("Sets the input file to use")
            .multiple(true)
            .default_value("."))
        .get_matches()
}

pub fn parse_cli() -> LsOptions {
    let matches = parse_opts();

    let color = matches.value_of("COLOR").unwrap_or_default();
    let paths: Vec<String> = matches.values_of("PATHS").unwrap_or_default().map(String::from).collect();
    let num_paths: usize = paths.len();

    LsOptions {
        paths: paths,
        show_dir_headers: num_paths > 1,
        list_dir_contents: !matches.is_present("DIR_NAME_ONLY"),
        output_format: if matches.is_present("GROUP_LONG") {
            format::OutputFormat::GroupLong
        } else if matches.is_present("LONG") {
            format::OutputFormat::Long
        } else {
            format::OutputFormat::Short
        },
        size_format: if matches.is_present("SI_SIZES") {
            format::SizeFormat::HumanSI
        } else if matches.is_present("HUMAN_SIZES") {
            format::SizeFormat::Human
        } else {
            format::SizeFormat::Machine
        },
        color: match color {
            "always" => format::ColorOption::Always,
            "auto" => format::ColorOption::Auto,
            "never" => format::ColorOption::Never,
            _ => panic!(format!("Invalid color: {}", color))
        },
        output_filter: if matches.is_present("ALL") {
            OutputFilter::All
        } else if matches.is_present("ALMOST_ALL") {
            OutputFilter::AlmostAll
        } else {
            OutputFilter::Visible
        }
    }
}

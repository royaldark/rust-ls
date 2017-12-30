extern crate colored;
extern crate chrono;
extern crate users;
extern crate libc;

use std::cmp::{Ord, Ordering};
use std::fs::{FileType, Metadata};
use std::path::{PathBuf};
use std::u32;
use std::os::unix::fs::{FileTypeExt, MetadataExt, PermissionsExt};

use self::colored::*;
use self::chrono::{DateTime, NaiveDateTime, Local, TimeZone};

use cli;

pub struct FsEntry {
    pub path: PathBuf,
    pub meta: Metadata
}

impl Eq for FsEntry {}

impl Ord for FsEntry {
    fn cmp(&self, other: &FsEntry) -> Ordering {
        self.path.cmp(&other.path)
    }
}

impl PartialOrd for FsEntry {
    fn partial_cmp(&self, other: &FsEntry) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq for FsEntry {
    fn eq(&self, other: &FsEntry) -> bool {
        self.path == other.path
    }
}

#[derive(Debug)]
pub enum OutputFormat {
    Short,    // Name only
    Long,     // Permissions, group, owner, size, etc.
    GroupLong // Like Long, but listing only each file's group, not owner
}

#[derive(Debug)]
pub enum SizeFormat {
    Machine,
    Human
}

#[derive(Debug)]
pub enum ColorOption {
    Always,
    Auto,
    Never
}

#[derive(Copy, Clone)]
enum SizeUnit {
    K = 1024,
    M = 1024 * 1024,
    G = 1024 * 1024 * 1024,
    T = 1024 * 1024 * 1024 * 1024,
    P = 1024 * 1024 * 1024 * 1024 * 1024
}

fn extract_bits_from_right(value: u32, start_pos: u32, end_pos: u32) -> u32 {
    let mask = (1 << (end_pos - start_pos)) - 1;
    (value >> start_pos) & mask
}

fn perm_mode_string(value: u32) -> String {
    let mut acc = String::new();

    acc.push_str(if value & 4 == 4 { "r" } else { "-" });
    acc.push_str(if value & 2 == 2 { "w" } else { "-" });
    acc.push_str(if value & 1 == 1 { "x" } else { "-" });

    acc
}

fn file_type_string(file_type: FileType) -> String {
    String::from(
        if file_type.is_file() {
            "-"
        } else if file_type.is_dir() {
            "d"
        } else if file_type.is_symlink() {
            "l"
        } else if file_type.is_block_device() {
            "b"
        } else if file_type.is_char_device() {
            "c"
        } else if file_type.is_fifo() {
            "p'"
        } else if file_type.is_socket() {
            "s'"
        } else {
            "?"
        }
    )
}

fn permissions_string(meta: &Metadata) -> String {
    let mut acc = String::new();
    let perms = meta.permissions();
    let file_type = meta.file_type();
    let mode = perms.mode();
    let wmode = extract_bits_from_right(mode, 0, 3);
    let gmode = extract_bits_from_right(mode, 3, 6);
    let umode = extract_bits_from_right(mode, 6, 9);

    acc.push_str(&file_type_string(file_type));
    acc.push_str(&perm_mode_string(umode));
    acc.push_str(&perm_mode_string(gmode));
    acc.push_str(&perm_mode_string(wmode));

    acc
}

fn human_size_string_for(len: u64, unit: SizeUnit, label: &str) -> String {
    let precision = if len < 10 * unit as u64 { 1 } else { 0 };
    let rounded = if len < 10 * unit as u64 {
       (len as f64 / unit as u64 as f64 * 10_f64).ceil() / 10_f64
    } else {
        (len as f64 / unit as u64 as f64).ceil()
    };

    format!("{:.precision$}{}",
        rounded,
        label,
        precision = precision)
}

fn human_size_string(len: u64) -> String {
    if len < SizeUnit::K as u64 {
        format!("{}", len)
    } else if len < SizeUnit::M as u64 {
        human_size_string_for(len, SizeUnit::K, "K")
    } else if len < SizeUnit::G as u64 {
        human_size_string_for(len, SizeUnit::M, "M")
    } else if len < SizeUnit::T as u64 {
        human_size_string_for(len, SizeUnit::G, "G")
    } else if len < SizeUnit::P as u64 {
        human_size_string_for(len, SizeUnit::T, "T")
    } else {
        human_size_string_for(len, SizeUnit::P, "P")
    }
}

fn size_string(len: u64, opts: &cli::LsOptions) -> String {
    match opts.size_format {
        SizeFormat::Machine => format!("{}", len),
        SizeFormat::Human => human_size_string(len)
    }
}

fn user_name(meta: &Metadata) -> String {
    match users::get_user_by_uid(meta.uid()) {
        Some(x) => x.name().to_owned(),
        None => "?".to_owned()
    }
}

fn group_name(meta: &Metadata) -> String {
    match users::get_group_by_gid(meta.gid()) {
        Some(x) => x.name().to_owned(),
        None => "?".to_owned()
    }
}

fn timestamp(meta: &Metadata) -> String {
    let format = "%b %d %H:%M";
    let ndt = NaiveDateTime::from_timestamp(meta.mtime(), 0 as u32);
    let dt: DateTime<Local> = Local::from_utc_datetime(&Local, &ndt);

    dt.format(format).to_string()
}

struct FormatEntry {
    permissions: String,
    nlinks: String,
    user: String,
    group: String,
    size: String,
    timestamp: String,
    file_name: String,
    file_type: String
}

fn to_format_entry(file: &FsEntry, opts: &cli::LsOptions) -> FormatEntry {
    FormatEntry {
        permissions: permissions_string(&file.meta),
        nlinks: format!("{}", file.meta.nlink()),
        user: user_name(&file.meta),
        group: group_name(&file.meta),
        size: size_string(file.meta.len(), &opts),
        timestamp: timestamp(&file.meta),
        file_name: file.path.file_name().unwrap().to_str().unwrap().to_owned(),
        file_type: file_type_string(file.meta.file_type())
    }
}

fn to_format_entries(entries: &Vec<FsEntry>, opts: &cli::LsOptions) -> Vec<FormatEntry> {
    let mut acc: Vec<FormatEntry> = Vec::with_capacity(entries.len());

    for entry in entries {
        acc.push(to_format_entry(&entry, opts));
    }

    acc
}

fn max_len<F>(entries: &Vec<FormatEntry>, f: F) -> usize 
    where F: Fn(&FormatEntry) -> usize {
    let mut max = 0 as usize;

    for entry in entries {
        let value: usize = f(&entry);
        if value > max {
            max = value;
        }
    }
    
    max
}

fn is_tty() -> bool {
    (unsafe { libc::isatty(libc::STDOUT_FILENO as i32) } != 0)
}

fn should_color(color: &ColorOption) -> bool {
    match color {
        &ColorOption::Always => true,
        &ColorOption::Never => false,
        &ColorOption::Auto => is_tty()
    }
}

fn color_file_name(file_name: String, file_type: String, color: &ColorOption) -> String {
    if should_color(color) {
        match file_type.as_str() {
            "b" | "c" => format!("{}", file_name.yellow().bold().on_black()).to_owned(),
            "d" => format!("{}", file_name.blue().bold()).to_owned(),
            _ => file_name
        }
    } else {
        file_name
    }
}

fn print_dir_header_if_needed(root: Option<FsEntry>, opts: &cli::LsOptions) -> () {
    if let Some(root) = root { if opts.show_dir_headers && root.meta.is_dir() {
            println!("{}:", root.path.to_str().unwrap());
        }
    }
}

fn long_form(root: Option<FsEntry>, entries: &Vec<FsEntry>, opts: &cli::LsOptions) -> () {
    let fmt_entries = to_format_entries(entries, opts);
    let nlinks_width = max_len(&fmt_entries, |x| x.nlinks.len());
    let size_width = max_len(&fmt_entries, |x| x.size.len());
    let user_width = max_len(&fmt_entries, |x| x.user.len());
    let group_width = max_len(&fmt_entries, |x| x.group.len());
    let timestamp_width = max_len(&fmt_entries, |x| x.timestamp.len());

    print_dir_header_if_needed(root, opts);

    for file in fmt_entries {
        println!("{} {:>nwidth$} {:<uwidth$} {:<gwidth$} {:>swidth$} {:<twidth$} {}",
                file.permissions,
                file.nlinks,
                file.user,
                file.group,
                file.size,
                file.timestamp,
                color_file_name(file.file_name, file.file_type, &opts.color),
                nwidth = nlinks_width,
                swidth = size_width,
                uwidth = user_width,
                gwidth = group_width,
                twidth = timestamp_width);
    }
}

fn group_long_form(root: Option<FsEntry>, entries: &Vec<FsEntry>, opts: &cli::LsOptions) -> () {
    let fmt_entries = to_format_entries(entries, opts);
    let nlinks_width = max_len(&fmt_entries, |x| x.nlinks.len());
    let size_width = max_len(&fmt_entries, |x| x.size.len());
    let group_width = max_len(&fmt_entries, |x| x.group.len());
    let timestamp_width = max_len(&fmt_entries, |x| x.timestamp.len());

    print_dir_header_if_needed(root, opts);

    for file in fmt_entries {
        println!("{} {:>nwidth$} {:<gwidth$} {:>swidth$} {:<twidth$} {}",
                file.permissions,
                file.nlinks,
                file.group,
                file.size,
                file.timestamp,
                color_file_name(file.file_name, file.file_type, &opts.color),
                nwidth = nlinks_width,
                swidth = size_width,
                gwidth = group_width,
                twidth = timestamp_width);
    }
}

fn short_form(root: Option<FsEntry>, entries: &Vec<FsEntry>, opts: &cli::LsOptions) -> () {
    print_dir_header_if_needed(root, opts);

    for file in to_format_entries(entries, opts) {
        println!("{}", color_file_name(file.file_name, file.file_type, &opts.color));
    }
}

pub fn print_entries(root: Option<FsEntry>, entries: &Vec<FsEntry>, opts: &cli::LsOptions) {
    match opts.output_format {
        OutputFormat::Long => long_form(root, &entries, &opts),
        OutputFormat::GroupLong => group_long_form(root, &entries, &opts),
        OutputFormat::Short => short_form(root, &entries, &opts)
    }
}
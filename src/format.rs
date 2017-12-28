extern crate users;
extern crate chrono;

use std::fs;
use std::u32;
use std::os::unix::fs::{FileTypeExt, MetadataExt, PermissionsExt};

use self::chrono::{DateTime, NaiveDateTime, Local, TimeZone};

use cli;

#[derive(Debug)]
pub enum OutputFormat {
    Short,
    Long
}

#[derive(Debug)]
pub enum SizeFormat {
    Machine,
    Human
}

fn extract_bits_from_right(value: u32, start_pos: u32, end_pos: u32) -> u32 {
    let mask = (1 << (end_pos - start_pos)) - 1;
    (value >> start_pos) & mask
}

fn perm_mode_string(value: u32) -> String {
    let mut acc = String::new();

    acc.push_str(
        if value & 4 == 4 {
            "r"
        } else {
            "-"
        }
    );

    acc.push_str(
        if value & 2 == 2 {
            "w"
        } else {
            "-"
        }
    );

    acc.push_str(
        if value & 1 == 1 {
            "x"
        } else {
            "-"
        }
    );

    acc
}

fn file_type_string(file_type: fs::FileType) -> String {
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

fn permissions_string(meta: &fs::Metadata) -> String {
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

#[derive(Copy, Clone)]
enum SizeUnit {
    K = 1024,
    M = 1024 * 1024,
    G = 1024 * 1024 * 1024,
    T = 1024 * 1024 * 1024 * 1024,
    P = 1024 * 1024 * 1024 * 1024 * 1024
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

fn user_name(meta: &fs::Metadata) -> String {
    match users::get_user_by_uid(meta.uid()) {
        Some(x) => x.name().to_owned(),
        None => "?".to_owned()
    }
}

fn group_name(meta: &fs::Metadata) -> String {
    match users::get_group_by_gid(meta.uid()) {
        Some(x) => x.name().to_owned(),
        None => "?".to_owned()
    }
}

fn timestamp(meta: &fs::Metadata) -> String {
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
    file_name: String
}

fn to_format_entry(file: &fs::DirEntry, opts: &cli::LsOptions) -> FormatEntry {
    let metadata = file.metadata().unwrap();

    FormatEntry {
        permissions: permissions_string(&metadata),
        nlinks: format!("{}", metadata.nlink()),
        user: user_name(&metadata),
        group: group_name(&metadata),
        size: size_string(metadata.len(), &opts),
        timestamp: timestamp(&metadata),
        file_name: file.file_name().into_string().unwrap()
    }
}

fn to_format_entries(entries: &Vec<fs::DirEntry>, opts: &cli::LsOptions) -> Vec<FormatEntry> {
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

pub fn long_form(entries: &Vec<fs::DirEntry>, opts: &cli::LsOptions) -> () {
    let fmt_entries = to_format_entries(entries, opts);
    let size_width = max_len(&fmt_entries, |x| x.size.len());

    for file in fmt_entries {
        println!("{} {} {} {} {:>swidth$} {} {}",
                file.permissions,
                file.nlinks,
                file.user,
                file.group,
                file.size,
                file.timestamp,
                file.file_name,
                swidth = size_width);
    }
}

pub fn short_form(entries: &Vec<fs::DirEntry>, opts: &cli::LsOptions) -> () {
    for file in to_format_entries(entries, opts) {
        println!("{}", file.file_name);
    }
}

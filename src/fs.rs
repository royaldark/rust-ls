use std::path::{PathBuf};
use std::error::{Error};
use std::fs;
use std::io;

use cli::{LsOptions, OutputFilter};
use format::{FsEntry, OutputFormat};
use format;

fn is_hidden(entry: &FsEntry) -> bool {
    entry.path.file_name().unwrap().to_str().unwrap().starts_with(".")
}

fn add_implied_dirs(dir: &str, fs_entries: &mut Vec<FsEntry>) -> () {
    let cur_path: PathBuf = [dir, "/."].iter().collect();
    let cur_meta = fs::metadata(cur_path.clone()).unwrap();
    let parent_path: PathBuf = [dir, "/.."].iter().collect();
    let parent_meta = fs::metadata(parent_path.clone()).unwrap();

    fs_entries.push(FsEntry {
        path: cur_path,
        meta: cur_meta
    });

    fs_entries.push(FsEntry {
        path: parent_path,
        meta: parent_meta
    });
}

fn to_fs_entries(dir: &FsEntry, dir_entries: &Vec<fs::DirEntry>, opts: &LsOptions) -> Vec<FsEntry> {
    let mut fs_entries: Vec<FsEntry> = Vec::new();

    for dir_entry in dir_entries {
        let fs_entry = FsEntry {
            path: dir_entry.path(),
            meta: dir_entry.metadata().unwrap()
        };

        if opts.output_filter == OutputFilter::All ||
                opts.output_filter == OutputFilter::AlmostAll ||
                (opts.output_filter == OutputFilter::Visible && !is_hidden(&fs_entry)) {
            fs_entries.push(fs_entry);
        }
    }

    if opts.output_filter == OutputFilter::All {
        //add_implied_dirs(dir, &mut fs_entries)
    }

    fs_entries.sort();

    fs_entries
}

fn ls_print_entries(root: Option<FsEntry>, entries: &Vec<FsEntry>, opts: &LsOptions) -> io::Result<u8> {
    match opts.output_format {
        OutputFormat::Long => format::long_form(root, &entries, &opts),
        OutputFormat::Short => format::short_form(root, &entries, &opts),
    }

    Ok(0)
}

fn ls_print_directory_contents(dir: FsEntry, opts: &LsOptions) -> io::Result<u8> {
    let entries = to_fs_entries(&dir, &fs::read_dir(&dir.path)?.map(|entry| entry.unwrap()).collect(), &opts);

    ls_print_entries(Some(dir), &entries, &opts)
}

fn ls_print_file(file: FsEntry, opts: &LsOptions) -> io::Result<u8> {
    let entries: Vec<FsEntry> = vec![file];

    ls_print_entries(None, &entries, &opts)
}

fn get_input_meta(inputs: &Vec<String>) -> io::Result<Vec<FsEntry>> {
    let mut acc: Vec<FsEntry> = vec![];

    for input in inputs {
        match fs::metadata(input) {
            Ok(meta) => {
                acc.push(FsEntry {
                    path: PathBuf::from(input),
                    meta: meta
                });
            },
            Err(e) => {
                let message = format!("cannot access '{}': {}", input, e.description());
                return Err(io::Error::new(io::ErrorKind::Other, message));
            }
        }
    }

    Ok(acc)
}

fn ls_print_paths(entries: Vec<FsEntry>, opts: LsOptions) -> io::Result<u8> {
    for entry in entries {
        if entry.meta.is_dir() {
            ls_print_directory_contents(entry, &opts)?;
        } else {
            ls_print_file(entry, &opts)?;
        }
    }

    Ok(0) 
}

pub fn ls_print_input(opts: LsOptions) -> io::Result<u8> {
    ls_print_paths(get_input_meta(&opts.paths)?, opts)
}
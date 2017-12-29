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

fn ls_print_entries(entries: &Vec<FsEntry>, opts: &LsOptions) -> io::Result<u8> {
    match opts.output_format {
        OutputFormat::Long => format::long_form(&entries, &opts),
        OutputFormat::Short => format::short_form(&entries, &opts),
    }

    Ok(0)
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

fn to_fs_entries(dir: &str, dir_entries: &Vec<fs::DirEntry>, opts: &LsOptions) -> Vec<FsEntry> {
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
        add_implied_dirs(dir, &mut fs_entries)
    }

    fs_entries.sort();

    fs_entries
}

fn ls_print_directory_contents(dir: &str, opts: LsOptions) -> io::Result<u8> {
    let entries = to_fs_entries(dir, &fs::read_dir(dir)?.map(|entry| entry.unwrap()).collect(), &opts);

    ls_print_entries(&entries, &opts)
}

fn ls_print_file(file: &str, meta: fs::Metadata, opts: LsOptions) -> io::Result<u8> {
    let path = PathBuf::from(file);
    let entry = FsEntry {
        path: path,
        meta: meta
    };
    let entries: Vec<FsEntry> = vec![entry];

    ls_print_entries(&entries, &opts)
}

pub fn ls_print_input(opts: LsOptions) -> io::Result<u8> {
    let path = opts.paths[0].clone();

    match fs::metadata(&path) {
        Ok(meta) => {
            if meta.is_dir() {
                ls_print_directory_contents(&path, opts)
            } else if meta.is_file() {
                ls_print_file(&path, meta, opts)
            } else {
                Ok(0) 
            }
        },
        Err(e) => {
            let message = format!("cannot access '{}': {}", path, e.description());
            Err(io::Error::new(io::ErrorKind::Other, message))
        }
    }
}
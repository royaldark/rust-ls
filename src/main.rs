use std::fs;
use std::io;

mod cli;
mod format;

fn sort_by_basename(entries: &mut Vec<fs::DirEntry>) -> () {
    entries.sort_by_key(|a| a.path().into_os_string().into_string().unwrap());
}

fn is_hidden(entry: &fs::DirEntry) -> bool {
    entry.file_name().into_string().unwrap().starts_with(".")
}

fn ls_print_directory_conents(dir: &str, opts: cli::LsOptions) -> io::Result<u8> {
    let mut entries: Vec<fs::DirEntry> = fs::read_dir(dir)?.map(|entry| entry.unwrap()).collect();
    entries.retain(|ref x| !is_hidden(&x));
    sort_by_basename(&mut entries);

    for file in entries {
        match opts.format {
            format::Format::Long => println!("{}", format::long_form(&file)),
            format::Format::Short => println!("{}", format::short_form(&file)),
        }
    }

    Ok(0)
}

fn ls_print_input(opts: cli::LsOptions) -> io::Result<u8> {
    let path = opts.paths[0].clone();
    let meta = fs::metadata(&path)?;

    if meta.is_dir() {
        ls_print_directory_conents(&path, opts)
    } else if meta.is_file() {
        Ok(0)
    } else {
        Ok(0) 
    }
}

fn main() {
    let opts = cli::parse_cli();

    println!("{:?}", opts);

    match ls_print_input(opts) {
        Ok(_) => return,
        Err(e) => println!("ERROR: {}", e),
    }
}
use std::env;
use std::fs;
use std::io;

mod format;

fn sort_by_basename(entries: &mut Vec<fs::DirEntry>) -> () {
    entries.sort_by_key(|a| a.path().into_os_string().into_string().unwrap());
}

fn is_hidden(entry: &fs::DirEntry) -> bool {
    entry.file_name().into_string().unwrap().starts_with(".")
}

fn ls_print_directory_conents(dir: &str, format: format::Format) -> io::Result<&str> {
    let mut entries: Vec<fs::DirEntry> = fs::read_dir(dir)?.map(|entry| entry.unwrap()).collect();
    entries.retain(|ref x| !is_hidden(&x));
    sort_by_basename(&mut entries);

    for file in entries {
        match format {
            format::Format::Long => println!("{}", format::long_form(&file)),
            format::Format::Short => println!("{}", format::short_form(&file)),
        }
    }

    Ok("got it")
}

fn ls_print_input(path: &str) -> io::Result<&str> {
    let meta = fs::metadata(path)?;
    if meta.is_dir() {
        ls_print_directory_conents(path, format::Format::Short)
    } else if meta.is_file() {
        Ok("")
    } else {
        Ok("errors are successes") 
    }
}

fn main() {
    let path: String;

    if let Some(arg1) = env::args().nth(1) {
        path = arg1;
    } else {
        path = String::from(".");
    }

    match ls_print_input(&path) {
        Ok(_) => return,
        Err(e) => println!("ERROR: {}", e),
    }
}
use std::fs;
use std::io;

fn sort_by_basename(entries: Vec<fs::DirEntry>) -> Vec<String> {
    let mut dest: Vec<String> = Vec::with_capacity(entries.len());

    for entry in entries {
        let file_name = entry.file_name().into_string().unwrap();
        dest.push(file_name);
    }

    dest.sort();
    dest
}

fn ls_print_directory_conents(dir: &str) -> io::Result<&str> {
    let entries = fs::read_dir(dir)?.map(|entry| entry.unwrap()).collect();

    for file in sort_by_basename(entries) {
        println!("{}", file);
    }

    Ok("got it")
}

fn ls_print_input(path: &str) -> io::Result<&str> {
    let meta = fs::metadata(path)?;
    if meta.is_dir() {
        ls_print_directory_conents(path)
    } else if meta.is_file() {
        Ok("")
    } else {
        Ok("errors are successes") 
    }
}

fn main() {
    match ls_print_input(".") {
        Ok(_) => return,
        Err(e) => println!("ERROR: {}", e),
    }
}
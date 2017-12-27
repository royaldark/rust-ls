use std::fs;

#[derive(Debug)]
pub enum Format {
    Short,
    Long
}

pub fn long_form(file: &fs::DirEntry) -> String {
    format!("{} {}",
             file.metadata().unwrap().len(),
             file.file_name().into_string().unwrap())
}

pub fn short_form(file: &fs::DirEntry) -> String {
    format!("{}", file.file_name().into_string().unwrap())
}

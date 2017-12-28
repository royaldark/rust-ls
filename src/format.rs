extern crate users;
extern crate chrono;

use std::fs;
use std::u32;
use std::os::unix::fs::{FileTypeExt, MetadataExt, PermissionsExt};

use self::chrono::{DateTime, NaiveDateTime, Utc};

use cli;


/*
total 20
-rw-r--r-- 1 joe joe 5640 Dec 27 23:56 Cargo.lock
-rw-r--r-- 1 joe joe  115 Dec 27 23:56 Cargo.toml
drwxr-xr-x 2 joe joe 4096 Dec 28 00:09 src
drwxr-xr-x 4 joe joe 4096 Dec 28 00:06 target


n nbit($number, $n) { return ($number >> $n-1) & 1;} 

total int = Sum of (physical_blocks_in_use) * physical_block_size/ls_block_size) for each file.

Where:

    ls_block_size is an arbitrary environment variable (normally 512 or 1024 bytes) which is freely modifiable with the --block-size=<int> flag on ls, the POSIXLY_CORRECT=1 GNU environment variable (to get 512-byte units), or the -k flag to force 1kB units.
    physical_block_size is the OS dependent value of an internal block interface, which may or may not be connected to the underlying hardware. This value is normally 512b or 1k, but is completely dependent on OS. It can be revealed through the %B value on stat or fstat. Note that this value is (almost always) unrelated to the number of physical blocks on a modern storage device.

len = (end_pos - start_pos)


     ‘c’
          character special file
     ‘C’
          high performance (“contiguous data”) file
     ‘d’
          directory
     ‘D’
          door (Solaris 2.5 and up)
     ‘l’
          symbolic link
     ‘M’
          off-line (“migrated”) file (Cray DMF)
     ‘n’
          network special file (HP-UX)
     ‘P’
          port (Solaris 10 and up)
     ‘?’
          some other file type


 In addition to the name of each file, print the file type, file
     mode bits, number of hard links, owner name, group name, size, and
     timestamp (*note Formatting file timestamps::), normally the
     modification time.  Print question marks for information that
     cannot be determined.


*/

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

fn human_size_string(len: u64) -> String {
    if len < 1024 {
        format!("{}", len)
    } else if len < 1024 * 1024 {
        format!("{:.1}K", len as f64 / 1000.0)
    } else {
        format!("{:.1}M", len as f64 / 1000.0 / 1000.0)
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
    let dt: DateTime<Utc> = DateTime::from_utc(ndt, Utc);

    dt.format(format).to_string()
}

pub fn long_form(file: &fs::DirEntry, opts: &cli::LsOptions) -> String {
    let metadata = file.metadata().unwrap();

    format!("{} {} {} {} {} {} {}",
            permissions_string(&metadata),
            metadata.nlink(),
            user_name(&metadata),
            group_name(&metadata),
            size_string(metadata.len(), &opts),
            timestamp(&metadata),
            file.file_name().into_string().unwrap())
}

pub fn short_form(file: &fs::DirEntry, _opts: &cli::LsOptions) -> String {
    format!("{}", file.file_name().into_string().unwrap())
}

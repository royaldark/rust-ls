mod cli;
mod format;
mod fs;

fn main() {
    let opts = cli::parse_cli();

    println!("{:?}", opts);

    match fs::ls_print_input(opts) {
        Ok(_) => return,
        Err(e) => println!("ls: {}", e),
    }
}
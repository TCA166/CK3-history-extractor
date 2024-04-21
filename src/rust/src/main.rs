use minijinja::{Environment, context};
use std::time::SystemTime;
use std::io::prelude::*;
use std::io::{stdout, stdin};
mod save_file;
use save_file::SaveFile;

fn main() {
    //Get the staring time
    let start_time = SystemTime::now();
    //User IO
    stdout().write_all(b"Enter the filename: ").unwrap();
    stdout().flush().unwrap();
    //raw file contents
    let mut filename = String::new();
    stdin().read_line(&mut filename).unwrap();
    let filename = filename.trim();
    //initialize the save file
    let mut save = SaveFile::new(filename);
    let obj = save.next();
    print!("{:#?}\n", obj);
    save.next();
}

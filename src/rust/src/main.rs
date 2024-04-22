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
    let save = SaveFile::new(filename);
    for i in save{
        print!("{}\r", i.get_name());
        //print!("{:#?}\n", i);
    }
    //Get the ending time
    let end_time = SystemTime::now();
    //Print the time taken
    println!("\nTime taken: {}s\n", end_time.duration_since(start_time).unwrap().as_secs());
}

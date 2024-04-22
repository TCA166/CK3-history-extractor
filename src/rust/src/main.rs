use std::collections::HashMap;
use std::hash::Hash;
use std::rc::Rc;
use std::time::SystemTime;
use std::io::prelude::*;
use std::io::{stdout, stdin};

mod game_object;

mod save_file;
use save_file::SaveFile;

mod game_state;
use game_state::GameState;

use crate::structures::{Player, Title};

mod structures;

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
    let save = SaveFile::new(filename); // now we have an iterator we can work with that returns these large objects
    // this is sort of like the first round of filtering where we store the objects we care about
    let mut game_state:GameState = GameState::new();
    for i in save{
        print!("{:?}", i.get_name());
        match i.get_name(){
            "played_character" => {
                let player = i.to::<Player>(&game_state);
                
            }
            _ => {}
        }
    }

    //Get the ending time
    let end_time = SystemTime::now();
    //Print the time taken
    println!("\nTime taken: {}s\n", end_time.duration_since(start_time).unwrap().as_secs());
}

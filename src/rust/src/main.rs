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
    let mut filename:String = String::new();
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2{
        stdout().write_all(b"Enter the filename: ").unwrap();
        stdout().flush().unwrap();
        //raw file contents
        let mut filename = String::new();
        stdin().read_line(&mut filename).unwrap();
        let filename = filename.trim();
    }
    else{
        filename = args[1].clone();
    }
    //initialize the save file
    let save = SaveFile::new(filename.as_str()); // now we have an iterator we can work with that returns these large objects
    // this is sort of like the first round of filtering where we store the objects we care about
    let mut game_state:GameState = GameState::new();
    for i in save{
        print!("{:?}", i.get_name());
        match i.get_name(){ //the order is kept consistent with the order in the save file
            "landed_titles" => {
                let landed = i.get("landed_titles").unwrap().as_object().unwrap();
                for l in landed.get_keys(){
                    game_state.add::<Title>(landed.get(&l).unwrap().as_object().unwrap().as_ref());
                }
            }
            "dynasties" => {
                let dynasties = i.get_keys();
                for d in dynasties{
                    game_state.add::<Title>(i.get(&d).unwrap().as_object().unwrap().as_ref());
                }
            }
            "living" => {
                let living = i.get_keys();
                for l in living{
                    game_state.add::<Player>(i.get(&l).unwrap().as_object().unwrap().as_ref());
                }
            }
            "dead_unprunable" => {
                let dead = i.get_keys();
                for d in dead{
                    game_state.add::<Player>(i.get(&d).unwrap().as_object().unwrap().as_ref());
                }
            }
            "played_character" => {
                game_state.add::<Player>(&i);
            }
            _ => {}
        }
    }

    //Get the ending time
    let end_time = SystemTime::now();
    //Print the time taken
    println!("\nTime taken: {}s\n", end_time.duration_since(start_time).unwrap().as_secs());
}

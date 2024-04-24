use std::cell::RefCell;
use std::time::SystemTime;
use std::io::prelude::*;
use std::io::{stdout, stdin};
use std::env;

mod game_object;

mod save_file;
use save_file::SaveFile;

mod game_state;
use game_state::GameState;

mod structures;

fn main() {
    env::set_var("RUST_BACKTRACE", "1");
    //Get the staring time
    let start_time = SystemTime::now();
    //User IO
    let mut filename = String::new();
    let args: Vec<String> = env::args().collect();
    if args.len() < 2{
        stdout().write_all(b"Enter the filename: ").unwrap();
        stdout().flush().unwrap();
        //raw file contents
        stdin().read_line(&mut filename).unwrap();
        filename = filename.trim().to_string();
    }
    else{
        filename = args[1].clone();
    }
    //initialize the save file
    let save = SaveFile::new(filename.as_str()); // now we have an iterator we can work with that returns these large objects
    // this is sort of like the first round of filtering where we store the objects we care about
    let mut game_state:GameState = GameState::new();
    for mut i in save{
        print!("{:?}\n", i.get_name());
        stdout().flush().unwrap();
        match i.get_name(){ //the order is kept consistent with the order in the save file
            "landed_titles" => {
                let o = i.to_object().unwrap();
                let landed = o.get_object_ref("landed_titles");
                for v in landed.get_obj_iter(){
                    let o = v.1.as_object_ref();
                    if o.is_none(){
                        println!("Landed title {} is none?", v.0);
                        continue;
                    }
                    game_state.add_title(o.unwrap());
                }
            }
            "dynasties" => {
                for d in i.to_object().unwrap().get_obj_iter(){
                    let o = d.1.as_object_ref().unwrap();
                    if o.get_name() == "dynasty_house" || o.get_name() == "dynasties"{
                        for h in o.get_obj_iter(){
                            let house = h.1.as_object_ref();
                            if house.is_none(){
                                println!("House {} is none?", h.0);
                                continue;
                            }
                            game_state.add_dynasty(house.unwrap());
                        }
                    }
                }
            }
            "living" => {
                let o = i.to_object().unwrap();
                let living = o.get_keys();
                for l in living{
                    game_state.add_character(o.get_object_ref(&l));
                }
            }
            "dead_unprunable" => {
                let o = i.to_object().unwrap();
                let dead = o.get_keys();
                for d in dead{
                    game_state.add_character(o.get_object_ref(&d));
                }
            }
            "played_character" => {
                let p = RefCell::from(i.to_object().unwrap());
                game_state.add_player(p.borrow());
            }
            _ => {
                i.skip();
            }
        }
    }

    //Get the ending time
    let end_time = SystemTime::now();
    //Print the time taken
    println!("\nTime taken: {}s\n", end_time.duration_since(start_time).unwrap().as_secs());
}

use csv::ReaderBuilder;
use image::{io::Reader as ImageReader, save_buffer};

use crate::game_object::GameId;

/// Returns a vector of bytes from a png file encoded with rgb8, meaning each pixel is represented by 3 bytes
fn read_png_bytes(path:String) -> Vec<u8>{
    let img = ImageReader::open(path).unwrap().decode().unwrap();
    let buff = img.to_rgb8();
    let bytes = buff.into_raw();
    bytes
}

/// A struct representing a game map, from which we can create [crate::structures::Title] maps
pub struct GameMap{
    height: u32,
    width: u32,
    byte_sz: usize,
    province_map: Vec<u8>,
    id_colors: Vec<[u8; 3]>,
}

const WATER_COLOR:[u8; 3] = [20, 150, 255];
const LAND_COLOR:[u8; 3] = [255, 255, 255];
const BLACK_COLOR:[u8; 3] = [0, 0, 0];

const IMG_WIDTH:u32 = 8192;
const IMG_HEIGHT:u32 = 4096;

const SCALE:u32 = 4;

impl GameMap{
    /// Creates a new GameMap from a province map and a definition.csv file located inside the provided path
    pub fn new(map_path:&str) -> Self{
        let mut provinces_bytes = read_png_bytes(map_path.to_owned() + "/provinces.png");
        let river_bytes = read_png_bytes(map_path.to_owned() + "/rivers.png");
        //apply river bytes as a mask to the provinces bytes so that non white pixels in rivers are black
        let len = provinces_bytes.len();
        let mut x = 0;
        while x < len{
            if river_bytes[x] != 255 || river_bytes[x + 1] != 255 || river_bytes[x + 2] != 255{
                provinces_bytes[x..x + 3].copy_from_slice(&BLACK_COLOR);
            }
            x += 3;
        }
        //save_buffer("provinces.png", &provinces_bytes, IMG_WIDTH, IMG_HEIGHT, image::ExtendedColorType::Rgb8).unwrap();
        //determine the x cutoff point for the provinces bytes - the maximum x coordinate of non black pixels
        let mut x;
        let mut y = 0;
        let mut max_x = 0;
        while y < IMG_HEIGHT{
            x = 0;
            while x < IMG_WIDTH{
                let idx = (y * IMG_WIDTH * 3 + x * 3) as usize;
                if idx < len && provinces_bytes[idx] != 0{
                    if x > max_x{
                        max_x = x;
                    }
                }
                x += 1;
            }
            y += 1;
        }
        //scale the image down to 1/4 of the size
        let mut x = 0;
        let mut y = 0;
        let mut new_bytes = Vec::new();
        while y < IMG_HEIGHT{
            x = 0;
            while x < max_x{
                let mut r:u16 = 0;
                let mut g:u16 = 0;
                let mut b:u16 = 0;
                let mut count = 0;
                let mut water:u8 = 0;
                for i in 0..SCALE{
                    for j in 0..SCALE{
                        let idx = ((y + i) * IMG_WIDTH * 3 + (x + j) * 3) as usize;
                        if provinces_bytes[idx..idx + 3] == BLACK_COLOR{
                            water += 1;
                        }
                        r += provinces_bytes[idx] as u16;
                        g += provinces_bytes[idx + 1] as u16;
                        b += provinces_bytes[idx + 2] as u16;
                        count += 1;
                    }
                }
                x += SCALE;
                if water > 2{
                    new_bytes.push(0);
                    new_bytes.push(0);
                    new_bytes.push(0);
                    continue;
                }
                r /= count;
                g /= count;
                b /= count;
                new_bytes.push(r as u8);
                new_bytes.push(g as u8);
                new_bytes.push(b as u8);
            }
            y += SCALE;
        }
        let width = x / SCALE;
        let height = IMG_HEIGHT / SCALE;
        //save_buffer("provinces.png", &new_bytes, width, height, image::ExtendedColorType::Rgb8).unwrap();
        //ok so now we have a province map with each land province being a set color and we now just need to read definition.csv
        let mut id_colors = Vec::new();
        let mut rdr = ReaderBuilder::new().delimiter(b';').from_path(map_path.to_owned() + "/definition.csv").unwrap();
        for record in rdr.records(){
            if record.is_err(){
                continue;
            }
            let record = record.unwrap();
            if record[0].chars().next().unwrap() == '#'{
                continue;
            }
            let r = record[1].parse::<u8>().unwrap();
            let g = record[2].parse::<u8>().unwrap();
            let b = record[3].parse::<u8>().unwrap();
            id_colors.push([r, g, b]);
        }
        GameMap{
            height: height,
            width: width,
            byte_sz: new_bytes.len(),
            province_map: new_bytes,
            id_colors: id_colors,
        }
    }

    /// Creates a new map from the province map with the colors of the provinces in id_list changed to target_color
    pub fn create_map(&self, id_list:Vec<GameId>, target_color:[u8; 3], output_path:&str) {
        let mut new_map = self.province_map.clone();
        let colors = id_list.iter().map(|id| self.id_colors[*id as usize]).collect::<Vec<[u8; 3]>>();
        let mut x = 0;
        while x < self.byte_sz{
            let pixel = &self.province_map[x..x + 3];
            let clr;
            if pixel[0] == 0 && pixel[1] == 0 && pixel[2] == 0{ //if we find a black pixel = water
                clr = &WATER_COLOR;
            } else if colors.contains(&[pixel[0], pixel[1], pixel[2]]) { //if we find a color in the list of colors we want to change
                clr = &target_color;
            } else{ //if we find something else we set it to white
                clr = &LAND_COLOR;
            }
            //this ending is a loop to minimize the number of times the checks above are done
            while x < self.byte_sz && new_map[x..x + 3] == *pixel{
                new_map[x..x + 3].copy_from_slice(clr);
                x += 3;
            }
        }
        save_buffer(output_path, &new_map, self.width, self.height, image::ExtendedColorType::Rgb8).unwrap();
    }
}

use std::{
    collections::{HashMap, HashSet},
    thread,
};

use csv::ReaderBuilder;

use image::{io::Reader as ImageReader, save_buffer, ImageBuffer, Rgba};

use super::super::{
    game_object::{GameId, GameString},
    save_file::SaveFile,
};

/// Returns a vector of bytes from a png file encoded with rgb8, meaning each pixel is represented by 3 bytes
fn read_png_bytes(path: String) -> Vec<u8> {
    let img = ImageReader::open(&path);
    if img.is_err() {
        let err = img.err().unwrap();
        panic!("Error {} reading image at path: {}, are you sure you are pointing at the right directory?", err, path);
    }
    let img = img.unwrap().decode().unwrap();
    let buff = img.to_rgb8();
    let bytes = buff.into_raw();
    bytes
}

/// Creates a mapping from barony title keys to province ids
fn create_title_province_map(game_path: &str) -> HashMap<String, GameId> {
    let path = game_path.to_owned() + "/common/landed_titles/00_landed_titles.txt";
    let file = SaveFile::open(&path);
    let mut map = HashMap::new();
    for mut title in file {
        let title_object = title.to_object();
        //DFS in the structure
        let mut stack = vec![&title_object];
        while let Some(o) = stack.pop() {
            let pro = o.get("province");
            if pro.is_some() {
                let id = pro.unwrap().as_id();
                map.insert(o.get_name().to_owned(), id);
            }
            for (_key, val) in o.get_obj_iter() {
                if let Some(obj) = val.as_object() {
                    stack.push(obj);
                }
            }
        }
    }
    return map;
}

/// A struct representing a game map, from which we can create [crate::structures::Title] maps
pub struct GameMap {
    height: u32,
    width: u32,
    byte_sz: usize,
    province_map: Vec<u8>,
    title_color_map: HashMap<String, [u8; 3]>,
}

const WATER_COLOR: [u8; 3] = [20, 150, 255];
const LAND_COLOR: [u8; 3] = [255, 255, 255];
const NULL_COLOR: [u8; 3] = [0, 0, 0];

const IMG_WIDTH: u32 = 8192;
const IMG_HEIGHT: u32 = 4096;

const SCALE: u32 = 4;

impl GameMap {
    /// Creates a new GameMap from a province map and a definition.csv file located inside the provided path.
    /// The function expects the path to be a valid CK3 game directory.
    pub fn new(game_path: &str) -> Self {
        let map_path = game_path.to_owned() + "/map_data";
        let mut provinces_bytes = read_png_bytes(map_path.to_owned() + "/provinces.png");
        let river_bytes = read_png_bytes(map_path.to_owned() + "/rivers.png");
        //apply river bytes as a mask to the provinces bytes so that non white pixels in rivers are black
        let len = provinces_bytes.len();
        let mut x = 0;
        while x < len {
            if river_bytes[x] != 255 || river_bytes[x + 1] != 255 || river_bytes[x + 2] != 255 {
                provinces_bytes[x..x + 3].copy_from_slice(&NULL_COLOR);
            }
            x += 3;
        }
        //save_buffer("provinces.png", &provinces_bytes, IMG_WIDTH, IMG_HEIGHT, image::ExtendedColorType::Rgb8).unwrap();
        //determine the x cutoff point for the provinces bytes - the maximum x coordinate of non black pixels
        let mut x;
        let mut y = 0;
        let mut max_x = 0;
        while y < IMG_HEIGHT {
            x = 0;
            while x < IMG_WIDTH {
                let idx = (y * IMG_WIDTH * 3 + x * 3) as usize;
                if idx < len && provinces_bytes[idx] != 0 {
                    if x > max_x {
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
        while y < IMG_HEIGHT {
            x = 0;
            while x < max_x {
                //unique scaling algorithm here, we take the most common color in a SCALE x SCALE square and set all the pixels in that square to that color
                //UNLESS a set amount are water, in which case we set the square to water
                let mut water: u8 = 0;
                let mut occurences = HashMap::new();
                for i in 0..SCALE {
                    for j in 0..SCALE {
                        let idx = ((y + i) * IMG_WIDTH * 3 + (x + j) * 3) as usize;
                        let slc = &provinces_bytes[idx..idx + 3];
                        if slc == &NULL_COLOR {
                            water += 1;
                        } else {
                            if occurences.contains_key(slc) {
                                *occurences.get_mut(slc).unwrap() += 1;
                            } else {
                                occurences.insert(slc, 1);
                            }
                        }
                    }
                }
                x += SCALE;
                if water > 3 {
                    new_bytes.extend_from_slice(&NULL_COLOR);
                    continue;
                }
                //add the most common color in colors to the new bytes
                let mut max = 0;
                let mut max_color: &[u8] = &[0, 0, 0];
                for (color, count) in occurences.iter() {
                    if count > &max {
                        max = *count;
                        max_color = *color;
                    }
                }
                new_bytes.extend_from_slice(max_color);
            }
            y += SCALE;
        }
        let width = x / SCALE;
        let height = IMG_HEIGHT / SCALE;
        //save_buffer("provinces.png", &new_bytes, width, height, image::ExtendedColorType::Rgb8).unwrap();
        //ok so now we have a province map with each land province being a set color and we now just need to read definition.csv
        let mut id_colors = HashMap::new();
        let mut rdr = ReaderBuilder::new()
            .comment(Some(b'#'))
            .flexible(true)
            .delimiter(b';')
            .from_path(map_path.to_owned() + "/definition.csv")
            .unwrap();
        for record in rdr.records() {
            if record.is_err() {
                continue;
            }
            let record = record.unwrap();
            let id = record[0].parse::<GameId>();
            if id.is_err() {
                continue;
            }
            let id = id.unwrap();
            let r = record[1].parse::<u8>().unwrap();
            let g = record[2].parse::<u8>().unwrap();
            let b = record[3].parse::<u8>().unwrap();
            id_colors.insert(id, [r, g, b]);
        }
        let title_province_map = create_title_province_map(game_path);
        GameMap {
            height: height,
            width: width,
            byte_sz: new_bytes.len(),
            province_map: new_bytes,
            title_color_map: title_province_map
                .iter()
                .map(|(k, v)| (k.to_owned(), id_colors.get(v).unwrap().clone()))
                .collect(),
        }
    }

    /// Creates a new map from the province map with the colors of the provinces in id_list changed to target_color
    /// Returns a vector of RGB bytes representing the new map
    fn create_map(&self, key_list: Vec<GameString>, target_color: &[u8; 3]) -> Vec<u8> {
        let mut new_map = Vec::with_capacity(self.province_map.len());
        let colors: HashSet<_> = key_list
            .iter()
            .map(|id| self.title_color_map[id.as_str()])
            .collect();
        let mut x = 0;
        while x < self.byte_sz {
            let mut z = x + 3; // overkill, but saves us an arithmetic operation
            let pixel: &[u8] = &self.province_map[x..z];
            let clr;
            if pixel == NULL_COLOR {
                //if we find a NULL pixel = water
                clr = &WATER_COLOR;
            } else if colors.contains(pixel) {
                //if we find a color in the list of colors we want to change
                clr = &target_color;
            } else {
                //if we find something else we set it to white
                clr = &LAND_COLOR;
            }
            new_map.extend_from_slice(clr);
            x = z;
            z = x + 3;
            //this ending is a loop to minimize the number of times the checks above are done
            while x < self.byte_sz && self.province_map[x..z] == *pixel {
                new_map.extend_from_slice(clr);
                x = z;
                z = x + 3;
            }
        }
        return new_map;
    }

    /// Creates a province map with the colors of the provinces in id_list changed to target_color
    /// Returns a [ImageBuffer] of RGBA bytes representing the new map
    pub fn create_map_buffer(
        &self,
        key_list: Vec<GameString>,
        target_color: &[u8; 3],
    ) -> ImageBuffer<Rgba<u8>, Vec<u8>> {
        //we need to convert the vec of bytes to a vec of rgba bytes
        let new_map = self.create_map(key_list, target_color);
        let mut rgba = Vec::with_capacity(new_map.len() * 4 / 3);
        for i in 0..new_map.len() {
            rgba.push(new_map[i]);
            if (i + 1) % 3 == 0 {
                rgba.push(255);
            }
        }
        ImageBuffer::from_vec(self.width, self.height, rgba).unwrap()
    }

    /// Creates a new map from the province map with the colors of the provinces in id_list changed to target_color
    pub fn create_map_file(
        &self,
        key_list: Vec<GameString>,
        target_color: &[u8; 3],
        output_path: &str,
    ) {
        let new_map = self.create_map(key_list, target_color);
        let width = self.width;
        let height = self.height;
        let output_path = output_path.to_owned();
        //we move the writing process out into a thread because it's an IO heavy operation
        thread::spawn(move || {
            save_buffer(
                output_path,
                &new_map,
                width,
                height,
                image::ExtendedColorType::Rgb8,
            )
            .unwrap();
        });
    }
}

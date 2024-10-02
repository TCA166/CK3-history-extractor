use std::{collections::HashMap, thread};

use csv::ReaderBuilder;

use image::{save_buffer, ImageBuffer, ImageReader, Rgba};

use plotters::{
    backend::BitMapBackend,
    drawing::IntoDrawingArea,
    element::Text,
    prelude::{EmptyElement, Rectangle},
    style::{Color, IntoFont, RGBAColor, ShapeStyle, BLACK},
};

use super::super::parser::{GameId, GameString, SaveFile};

// color stuff

/// The color of the text drawn on the map
const TEXT_COLOR: RGBAColor = RGBAColor(0, 0, 0, 0.5);
/// The color of the water on the map
const WATER_COLOR: [u8; 3] = [20, 150, 255];
/// The color of the land on the map
const LAND_COLOR: [u8; 3] = [255, 255, 255];
/// The color of the null pixels on the map
const NULL_COLOR: [u8; 3] = [0, 0, 0];

// map image stuff

/// The width of the input map image
const IMG_WIDTH: u32 = 8192;
/// The height of the input map image
const IMG_HEIGHT: u32 = 4096;
/// The scale factor for the input map image
const SCALE: u32 = 4;

// File system stuff

const MAP_PATH_SUFFIX: &str = "/map_data";
const PROVINCES_SUFFIX: &str = "/provinces.png";
const RIVERS_SUFFIX: &str = "/rivers.png";
const DEFINITION_SUFFIX: &str = "/definition.csv";

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

/// Draws given text on an image buffer, the text is placed at the bottom left corner and is 5% of the height of the image
fn draw_text(img: &mut [u8], width: u32, height: u32, text: &str) {
    //TODO is this the best way to draw text?
    let back = BitMapBackend::with_buffer(img, (width, height)).into_drawing_area();
    let text_height = height / 20;
    let style = ("sans-serif", text_height).into_font().color(&TEXT_COLOR);
    back.draw(&Text::new(
        text,
        (10, height as i32 - text_height as i32),
        style,
    ))
    .unwrap();
    back.present().unwrap();
}

/// Draws a legend on the given image buffer, the legend is placed at the bottom right corner and consists of a series of colored rectangles with text labels
fn draw_legend(img: &mut [u8], width: u32, height: u32, legend: Vec<(String, [u8; 3])>) {
    let back = BitMapBackend::with_buffer(img, (width, height)).into_drawing_area();
    let text_height = (height / 30) as i32;
    let style = ("sans-serif", text_height).into_font();
    let mut x = (width / 50) as i32;
    for (label, color) in legend {
        let text_size = style.box_size(&label).unwrap();
        let margin = text_height / 3;
        back.draw(
            &(EmptyElement::at((x, height as i32 - (text_height * 2)))
                + Rectangle::new(
                    [(0, 0), (text_height, text_height)],
                    ShapeStyle {
                        color: RGBAColor(color[0], color[1], color[2], 1.0),
                        filled: true,
                        stroke_width: 1,
                    },
                )
                + Rectangle::new(
                    [(0, 0), (text_height, text_height)],
                    ShapeStyle {
                        color: BLACK.to_rgba(),
                        filled: false,
                        stroke_width: 1,
                    },
                )
                + Text::new(
                    label,
                    (text_height + margin, (text_height - text_size.1 as i32)),
                    style.clone(),
                )),
        )
        .unwrap();
        x += text_height + text_size.0 as i32 + (margin * 2);
    }
    back.present().unwrap();
}

/// A struct representing a game map, from which we can create [crate::structures::Title] maps
pub struct GameMap {
    height: u32,
    width: u32,
    byte_sz: usize,
    province_map: Vec<u8>,
    title_color_map: HashMap<String, [u8; 3]>,
}

impl GameMap {
    /// Creates a new GameMap from a province map and a definition.csv file located inside the provided path.
    /// The function expects the path to be a valid CK3 game directory.
    pub fn new(game_path: &str) -> Self {
        let map_path = game_path.to_owned() + MAP_PATH_SUFFIX;
        let mut provinces_bytes = read_png_bytes(map_path.to_owned() + PROVINCES_SUFFIX);
        let river_bytes = read_png_bytes(map_path.to_owned() + RIVERS_SUFFIX);
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
            .from_path(map_path.to_owned() + DEFINITION_SUFFIX)
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

    /// Creates a new map from the province map with the colors of the provinces in id_list changed to a color determined by assoc
    /// Returns a vector of RGB bytes representing the new map
    fn create_map<F>(&self, key_list: Vec<GameString>, assoc: F) -> Vec<u8>
    where
        F: Fn(&String) -> [u8; 3],
    {
        let mut new_map = Vec::with_capacity(self.province_map.len());
        let mut colors: HashMap<&[u8], [u8; 3]> = HashMap::new();
        for k in key_list.iter() {
            let color = self.title_color_map.get(k.as_str());
            if color.is_some() {
                colors.insert(color.unwrap(), assoc(k));
            } else {
                // huh? this is weird
            }
        }
        let mut x = 0;
        while x < self.byte_sz {
            let mut z = x + 3; // overkill, but saves us an arithmetic operation
            let pixel: &[u8] = &self.province_map[x..z];
            let clr;
            if pixel == NULL_COLOR {
                //if we find a NULL pixel = water
                clr = &WATER_COLOR;
            } else {
                let color = colors.get(pixel);
                if color.is_some() {
                    clr = color.unwrap();
                } else {
                    clr = &LAND_COLOR;
                }
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
        label: &str,
    ) -> ImageBuffer<Rgba<u8>, Vec<u8>> {
        //we need to convert the vec of bytes to a vec of rgba bytes
        let mut new_map = self.create_map(key_list, |_: &String| *target_color);
        if !label.is_empty() {
            draw_text(&mut new_map, self.width, self.height, label);
        }
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
        label: &str,
    ) {
        let mut new_map = self.create_map(key_list, |_: &String| *target_color);
        let width = self.width;
        let height = self.height;
        let output_path = output_path.to_owned();
        let label = label.to_owned();
        //we move the writing process out into a thread because it's an IO heavy operation
        thread::spawn(move || {
            if !label.is_empty() {
                draw_text(&mut new_map, width, height, &label);
            }
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

    /// Creates a new map from the province map with the colors of the provinces in id_list changed to a color determined by assoc
    pub fn create_map_graph<F>(&self, assoc: F, output_path: &str, legend: Vec<(String, [u8; 3])>)
    where
        F: Fn(&String) -> [u8; 3],
    {
        let mut new_map = self.create_map(
            self.title_color_map
                .keys()
                .map(|x| GameString::new(x.to_owned()))
                .collect(),
            assoc,
        );
        let width = self.width;
        let height = self.height;
        let output_path = output_path.to_owned();
        //we move the writing process out into a thread because it's an IO heavy operation
        thread::spawn(move || {
            draw_legend(&mut new_map, width, height, legend);
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

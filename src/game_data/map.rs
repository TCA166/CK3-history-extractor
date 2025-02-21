use std::{borrow::Borrow, error, io, num::ParseIntError, path::Path, thread};

use csv::ReaderBuilder;

use derive_more::{Display, From};
use image::{save_buffer, ImageBuffer, ImageReader, Rgba};

use plotters::{
    backend::BitMapBackend,
    drawing::IntoDrawingArea,
    element::Text,
    prelude::{EmptyElement, Rectangle},
    style::{Color, IntoFont, RGBAColor, ShapeStyle, BLACK},
};
use serde::Serialize;

use super::super::{
    parser::{GameId, GameString},
    types::HashMap,
};

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

#[derive(Debug, From, Display)]
pub enum MapError {
    IoError(io::Error),
    ImageError(image::ImageError),
    DefinitionError(csv::Error),
    ParsingError(ParseIntError),
}

impl error::Error for MapError {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        match self {
            MapError::IoError(e) => Some(e),
            MapError::ImageError(e) => Some(e),
            MapError::DefinitionError(e) => Some(e),
            MapError::ParsingError(e) => Some(e),
        }
    }
}

/// Returns a vector of bytes from a png file encoded with rgb8, meaning each pixel is represented by 3 bytes
fn read_png_bytes<P: AsRef<Path>>(path: P) -> Result<Box<[u8]>, MapError> {
    Ok(ImageReader::open(path)?
        .decode()?
        .to_rgb8()
        .into_raw()
        .into_boxed_slice())
}

/// An instance of an image, that depicts a map.
pub struct Map {
    height: u32,
    width: u32,
    bytes: Box<[u8]>,
}

impl Map {
    /// Draws given text on an image buffer, the text is placed at the bottom left corner and is 5% of the height of the image
    pub fn draw_text<T: Borrow<str>>(&mut self, text: T) {
        let back = BitMapBackend::with_buffer(&mut self.bytes, (self.width, self.height))
            .into_drawing_area();
        let text_height = self.height / 20;
        let style = ("sans-serif", text_height).into_font().color(&TEXT_COLOR);
        back.draw(&Text::new(
            text,
            (10, self.height as i32 - text_height as i32),
            style,
        ))
        .unwrap();
        back.present().unwrap();
    }

    /// Draws a legend on the given image buffer, the legend is placed at the bottom right corner and consists of a series of colored rectangles with text labels
    pub fn draw_legend<I: IntoIterator<Item = (String, [u8; 3])>>(&mut self, legend: I) {
        let back = BitMapBackend::with_buffer(&mut self.bytes, (self.width, self.height))
            .into_drawing_area();
        let text_height = (self.height / 30) as i32;
        let style = ("sans-serif", text_height).into_font();
        let mut x = (self.width / 50) as i32;
        for (label, color) in legend {
            let text_size = style.box_size(&label).unwrap();
            let margin = text_height / 3;
            back.draw(
                &(EmptyElement::at((x, self.height as i32 - (text_height * 2)))
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

    pub fn save<P: AsRef<Path>>(self, path: P) {
        let path = path.as_ref().to_owned();
        thread::spawn(move || {
            save_buffer(
                path,
                &self.bytes,
                self.width,
                self.height,
                image::ExtendedColorType::Rgb8,
            )
            .unwrap();
        });
    }
}

impl Into<ImageBuffer<Rgba<u8>, Vec<u8>>> for Map {
    fn into(self) -> ImageBuffer<Rgba<u8>, Vec<u8>> {
        let mut rgba = Vec::with_capacity(self.bytes.len() * 4 / 3);
        for i in 0..self.bytes.len() {
            rgba.push(self.bytes[i]);
            if (i + 1) % 3 == 0 {
                rgba.push(255);
            }
        }
        ImageBuffer::from_vec(self.width, self.height, rgba).unwrap()
    }
}

/// A struct representing a game map, from which we can create [Map] instances
#[derive(Serialize)]
pub struct GameMap {
    height: u32,
    width: u32,
    province_map: Box<[u8]>,
    title_color_map: HashMap<String, [u8; 3]>,
}

impl GameMap {
    /// Creates a new GameMap from a province map and a definition.csv file located inside the provided path.
    /// The function expects the path to be a valid CK3 game directory.
    pub fn new<P: AsRef<Path>>(
        provinces_path: P,
        rivers_path: P,
        definition_path: P,
        province_barony_map: HashMap<GameId, String>,
    ) -> Result<Self, MapError> {
        let mut provinces_bytes = read_png_bytes(provinces_path)?;
        let river_bytes = read_png_bytes(rivers_path)?;
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
        let mut new_bytes = Vec::with_capacity(provinces_bytes.len() / (SCALE * SCALE) as usize);
        while y < IMG_HEIGHT {
            x = 0;
            while x < max_x {
                //unique scaling algorithm here, we take the most common color in a SCALE x SCALE square and set all the pixels in that square to that color
                //UNLESS a set amount are water, in which case we set the square to water
                let mut water: u8 = 0;
                let mut occurences = HashMap::default();
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
        let mut key_colors: HashMap<String, [u8; 3]> = HashMap::default();
        let mut rdr = ReaderBuilder::new()
            .comment(Some(b'#'))
            .flexible(true)
            .delimiter(b';')
            .from_path(definition_path)?;
        for record in rdr.records() {
            let record = match record {
                Ok(record) => record,
                Err(_) => continue,
            };
            let id = match record[0].parse::<GameId>() {
                Ok(id) => id,
                Err(_) => continue,
            };
            let r = record[1].parse::<u8>()?;
            let g = record[2].parse::<u8>()?;
            let b = record[3].parse::<u8>()?;
            if let Some(barony) = province_barony_map.get(&id) {
                key_colors.insert(barony.clone(), [r, g, b]);
            }
        }
        Ok(GameMap {
            height: height,
            width: width,
            province_map: new_bytes.into_boxed_slice(),
            title_color_map: key_colors,
        })
    }
}

pub trait MapGenerator {
    /// Creates a new instance of map, with pixels colored in accordance with assoc function. key_list acts as a whitelist of keys to use assoc on. If it's None then assoc is applied to all keys.
    fn create_map<F: Fn(&str) -> [u8; 3], I: IntoIterator<Item = GameString>>(
        &self,
        assoc: F,
        key_list: Option<I>,
    ) -> Map;

    /// Creates a new instance of map, with all pixels corresponding to keys in the key_list colored same as the target_color
    fn create_map_flat<I: IntoIterator<Item = GameString>>(
        &self,
        key_list: I,
        target_color: [u8; 3],
    ) -> Map {
        self.create_map(|_| target_color, Some(key_list))
    }
}

impl MapGenerator for GameMap {
    fn create_map<F: Fn(&str) -> [u8; 3], I: IntoIterator<Item = GameString>>(
        &self,
        assoc: F,
        key_list: Option<I>,
    ) -> Map {
        let mut new_map = Vec::with_capacity(self.province_map.len());
        let mut colors: HashMap<&[u8], [u8; 3]> = HashMap::default();
        if let Some(key_list) = key_list {
            for k in key_list {
                if let Some(color) = self.title_color_map.get(k.as_ref()) {
                    colors.insert(color, assoc(k.as_ref()));
                } else {
                    // huh? this is weird
                }
            }
        } else {
            for (k, v) in self.title_color_map.iter() {
                colors.insert(v, assoc(k));
            }
        }
        let mut x = 0;
        while x < self.province_map.len() {
            let mut z = x + 3; // overkill, but saves us an arithmetic operation
            let pixel: &[u8] = &self.province_map[x..z];
            let clr;
            if pixel == NULL_COLOR {
                //if we find a NULL pixel = water
                clr = &WATER_COLOR;
            } else {
                if let Some(color) = colors.get(pixel) {
                    clr = color;
                } else {
                    clr = &LAND_COLOR;
                }
            }
            new_map.extend_from_slice(clr);
            x = z;
            z = x + 3;
            //this ending is a loop to minimize the number of times the checks above are done
            while x < self.province_map.len() && self.province_map[x..z] == *pixel {
                new_map.extend_from_slice(clr);
                x = z;
                z = x + 3;
            }
        }
        return Map {
            height: self.height,
            width: self.width,
            bytes: new_map.into_boxed_slice(),
        };
    }
}

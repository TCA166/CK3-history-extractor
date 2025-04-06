use std::{borrow::Borrow, error, io, num::ParseIntError, ops::Deref, path::Path, thread};

use csv::ReaderBuilder;

use derive_more::{Display, From};
use image::{
    imageops::{crop_imm, resize, FilterType},
    ImageReader, Rgb, RgbImage,
};

use plotters::{
    backend::BitMapBackend,
    drawing::IntoDrawingArea,
    element::Text,
    prelude::{EmptyElement, Rectangle},
    style::{Color, IntoFont, RGBAColor, ShapeStyle, BLACK},
};
use serde::Serialize;

use base64::prelude::*;

use super::super::types::{GameId, GameString, HashMap};

// color stuff

/// The color of the text drawn on the map
const TEXT_COLOR: RGBAColor = RGBAColor(0, 0, 0, 0.5);
/// The color of the water on the map
const WATER_COLOR: Rgb<u8> = Rgb([20, 150, 255]);
/// The color of the land on the map
const LAND_COLOR: Rgb<u8> = Rgb([255, 255, 255]);
/// The color of the null pixels on the map
const NULL_COLOR: Rgb<u8> = Rgb([0, 0, 0]);

// map image stuff

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
fn read_png_bytes<P: AsRef<Path>>(path: P) -> Result<RgbImage, MapError> {
    Ok(ImageReader::open(path)?.decode()?.to_rgb8())
}

pub trait MapImage {
    fn draw_text<T: Borrow<str>>(&mut self, text: T);

    fn draw_legend<C: Into<Rgb<u8>>, I: IntoIterator<Item = (String, C)>>(&mut self, legend: I);

    fn save_in_thread<P: AsRef<Path>>(self, path: P);
}

impl MapImage for RgbImage {
    /// Draws given text on an image buffer, the text is placed at the bottom left corner and is 5% of the height of the image
    fn draw_text<T: Borrow<str>>(&mut self, text: T) {
        let dimensions = self.dimensions();
        let text_height = dimensions.1 / 20;
        let back = BitMapBackend::with_buffer(self, dimensions).into_drawing_area();
        let style = ("sans-serif", text_height).into_font().color(&TEXT_COLOR);
        back.draw(&Text::new(
            text,
            (10, dimensions.1 as i32 - text_height as i32),
            style,
        ))
        .unwrap();
        back.present().unwrap();
    }

    /// Draws a legend on the given image buffer, the legend is placed at the bottom right corner and consists of a series of colored rectangles with text labels
    fn draw_legend<C: Into<Rgb<u8>>, I: IntoIterator<Item = (String, C)>>(&mut self, legend: I) {
        let dimensions = self.dimensions();
        let text_height = (dimensions.1 / 30) as i32;
        let back = BitMapBackend::with_buffer(self, dimensions).into_drawing_area();
        let style = ("sans-serif", text_height).into_font();
        let mut x = (dimensions.0 / 50) as i32;
        for (label, color) in legend {
            let text_size = style.box_size(&label).unwrap();
            let margin = text_height / 3;
            let color = color.into();
            back.draw(
                &(EmptyElement::at((x, dimensions.1 as i32 - (text_height * 2)))
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

    fn save_in_thread<P: AsRef<Path>>(self, path: P) {
        let path = path.as_ref().to_owned();
        thread::spawn(move || {
            self.save(path).unwrap();
        });
    }
}

fn serialize_into_b64<S: serde::Serializer>(
    image: &RgbImage,
    serializer: S,
) -> Result<S::Ok, S::Error> {
    serializer.serialize_str(&BASE64_STANDARD.encode(image.as_raw()))
}

fn serialize_title_color_map<S: serde::Serializer>(
    title_color_map: &HashMap<String, Rgb<u8>>,
    serializer: S,
) -> Result<S::Ok, S::Error> {
    let mut map = HashMap::new();
    for (k, v) in title_color_map.iter() {
        map.insert(k.clone(), v.0);
    }
    serializer.serialize_some(&map)
}

/// A struct representing a game map, from which we can create [Map] instances
#[derive(Serialize)]
pub struct GameMap {
    height: u32,
    width: u32,
    #[serde(serialize_with = "serialize_into_b64")]
    province_map: RgbImage,
    #[serde(serialize_with = "serialize_title_color_map")]
    title_color_map: HashMap<String, Rgb<u8>>,
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
        let mut provinces = read_png_bytes(provinces_path)?;
        let river = read_png_bytes(rivers_path)?;
        //apply river bytes as a mask to the provinces bytes so that non white pixels in rivers are black
        for (p_b, r_b) in provinces.pixels_mut().zip(river.pixels()) {
            if r_b[0] != 255 || r_b[1] != 255 || r_b[2] != 255 {
                *p_b = NULL_COLOR;
            }
        }
        let (width, height) = provinces.dimensions();
        // we need to find a bounding box for the terrain
        let mut max_x = 0;
        let mut min_x = width;
        let mut max_y = 0;
        let mut min_y = height;
        for x in 0..width {
            for y in 0..height {
                if provinces.get_pixel(x, y) != &NULL_COLOR {
                    if x > max_x {
                        max_x = x;
                    }
                    if x < min_x {
                        min_x = x;
                    }
                    if y > max_y {
                        max_y = y;
                    }
                    if y < min_y {
                        min_y = y;
                    }
                }
            }
        }
        let width = max_x - min_x;
        let height = max_y - min_y;
        let cropped = crop_imm(&provinces, min_x, min_y, width, height);

        //scale the image down to 1/4 of the size
        provinces = resize(
            cropped.deref(),
            (width / SCALE) as u32,
            (height / SCALE) as u32,
            FilterType::Nearest,
        );
        //provinces.save("test.png").unwrap();
        //ok so now we have a province map with each land province being a set color and we now just need to read definition.csv
        let mut key_colors: HashMap<String, Rgb<u8>> = HashMap::default();
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
                // FIXME this doesn't work for EK2
                key_colors.insert(barony.clone(), Rgb([r, g, b]));
            }
        }
        Ok(GameMap {
            height: height,
            width: width,
            province_map: provinces,
            title_color_map: key_colors,
        })
    }
}

pub trait MapGenerator {
    /// Creates a new instance of map, with pixels colored in accordance with assoc function. key_list acts as a whitelist of keys to use assoc on. If it's None then assoc is applied to all keys.
    fn create_map<C: Into<Rgb<u8>> + Clone, F: Fn(&str) -> C, I: IntoIterator<Item = GameString>>(
        &self,
        assoc: F,
        key_list: Option<I>,
    ) -> RgbImage;

    /// Creates a new instance of map, with all pixels corresponding to keys in the key_list colored same as the target_color
    fn create_map_flat<C: Into<Rgb<u8>> + Clone, I: IntoIterator<Item = GameString>>(
        &self,
        key_list: I,
        target_color: C,
    ) -> RgbImage {
        self.create_map(|_| target_color.clone(), Some(key_list))
    }
}

impl MapGenerator for GameMap {
    // this place is very hot according to the perf profiler
    fn create_map<
        C: Into<Rgb<u8>> + Clone,
        F: Fn(&str) -> C,
        I: IntoIterator<Item = GameString>,
    >(
        &self,
        assoc: F,
        key_list: Option<I>,
    ) -> RgbImage {
        let mut colors = HashMap::default();
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
        let mut new_map = self.province_map.clone();
        for pixel in new_map.pixels_mut() {
            if pixel == &NULL_COLOR {
                *pixel = WATER_COLOR;
            } else {
                if let Some(color) = colors.get(pixel) {
                    *pixel = (*color).clone().into();
                } else {
                    *pixel = LAND_COLOR;
                }
            }
        }
        return new_map;
    }
}

use std::collections::HashMap;

use rand::{thread_rng, Rng};

use plotters::{
    coord::types::{RangedCoordf64, RangedCoordi32, RangedCoordu32},
    prelude::*,
};

// This is a cool little library that provides the TREE LAYOUT ALGORITHM, the rendering is done by plotters
//https://github.com/zxch3n/tidy/tree/master it is sort of tiny so here github link in case it goes down
use tidy_tree::TidyTree;

use super::super::{
    game_object::{GameId, GameString},
    game_state::GameState,
    structures::{Character, Dynasty, GameObjectDerived, Title},
    types::{Shared, Wrapper},
};
use super::timeline::RealmDifference;

const GRAPH_SIZE: (u32, u32) = (1024, 768);

const TREE_SCALE: f64 = 1.5;

const NO_PARENT: usize = usize::MAX;

/// Handles node initialization within the graph.
/// Tree is the tree object we are adding the node to, stack is the stack we are using to traverse the tree, storage is the hashmap we are using to store the node data, fnt is the font we are using to calculate the size of the node, and parent is the parent node id.
fn handle_node<T: TreeNode>(
    node: Shared<T>,
    tree: &mut TidyTree,
    stack: &mut Vec<(usize, Shared<T>)>,
    storage: &mut HashMap<
        usize,
        (
            usize,
            GameString,
            (f64, f64),
            (i32, i32),
            Option<GameString>,
        ),
    >,
    parent: usize,
    sz: &(u32, u32),
) -> usize {
    let ch = node.get_internal();
    let id = ch.get_id() as usize;
    let name = ch.get_name().clone();
    //we use sz, which is the rough size of a character, to calculate the size of the node
    let txt_width = sz.0 * name.len() as u32;
    let node_width = txt_width as f64 * TREE_SCALE;
    let node_height = sz.1 as f64 * TREE_SCALE;
    //we also here calculate the point where the text should be drawn while we have convenient access to both size with margin and without
    let txt_point = (
        -(node_width as i32 - txt_width as i32),
        -(node_height as i32 - sz.1 as i32),
    );
    //add node to tree
    tree.add_node(id, node_width, node_height, parent);
    stack.push((id, node.clone()));
    //add aux data to storage because the nodes only store the id and no additional data
    storage.insert(
        id,
        (
            parent,
            name,
            (node_width, node_height),
            txt_point,
            ch.get_class(),
        ),
    );
    return id;
}

/// A trait for objects that can be used in a tree structure
pub trait TreeNode: GameObjectDerived {
    /// Returns an iterator over the children of the node
    fn get_children(&self) -> &Vec<Shared<Self>>;

    /// Returns an iterator over the parent of the node
    fn get_parent(&self) -> &Vec<Shared<Self>>;

    /// Returns the class of the node
    fn get_class(&self) -> Option<GameString>;
}

fn create_graph(data: &Vec<(u32, u32)>, output_path: &str) {
    let mut min_x: u32 = 0;
    let mut max_x: u32 = 0;
    let mut min_y: u32 = 0;
    let mut max_y: u32 = 0;
    for (x, y) in data {
        if *x < min_x || min_x == 0 {
            min_x = *x;
        }
        if *x > max_x {
            max_x = *x;
        }
        if *y < min_y {
            min_y = *y;
        }
        if *y > max_y {
            max_y = *y;
        }
    }

    let root = SVGBackend::new(output_path, GRAPH_SIZE).into_drawing_area();
    root.fill(&WHITE).unwrap();
    let mut chart = ChartBuilder::on(&root)
        //.caption("Deaths of culture members through time", ("sans-serif", 50).into_font())
        .margin(5)
        .x_label_area_size(30)
        .y_label_area_size(30)
        .build_cartesian_2d(min_x..max_x, min_y..(max_y + 10))
        .unwrap();

    chart.configure_mesh().draw().unwrap();

    chart
        .draw_series(LineSeries::new(data.iter().map(|(x, y)| (*x, *y)), &RED))
        .unwrap();
}

/// An object that can create graphs from the game state
pub struct Grapher {
    /// Stored graph data for all faiths, certainly less memory efficient but the speed is worth it
    faith_graph_complete: HashMap<GameId, Vec<(u32, u32)>>,
    culture_graph_complete: HashMap<GameId, Vec<(u32, u32)>>,
}

impl Grapher {
    pub fn new(game_state: &GameState) -> Self {
        //the idea is to get all the data we may need in one go, chances are all of it is gonna be needed anywho
        Grapher {
            faith_graph_complete: game_state.get_yearly_deaths(|c| {
                let faith = c.get_faith();
                if faith.is_some() {
                    return Some(faith.unwrap().get_internal().get_id());
                }
                None
            }),
            culture_graph_complete: game_state.get_yearly_deaths(|c| {
                let culture = c.get_culture();
                if culture.is_some() {
                    return Some(culture.unwrap().get_internal().get_id());
                }
                None
            }),
        }
    }

    /// Creates a tree graph from a given node
    /// The reverse parameter determines if the tree is drawn from the parent to the children or the other way around
    pub fn create_tree_graph<T: TreeNode>(
        &self,
        start: Shared<T>, // the root node that is a TreeNode
        reverse: bool,
        output_path: &str,
    ) {
        let mut tree = TidyTree::with_tidy_layout(TREE_SCALE * 15.0, TREE_SCALE * 5.0);
        //tree nodes don't have any data attached to them, so we need to store the data separately
        let mut storage = HashMap::new();
        let fnt = ("sans-serif", 6.66 * TREE_SCALE).into_font();
        let sz = fnt.box_size("X").unwrap(); // from this we determine a rough size of a character
        let mut stack = Vec::new(); //BFS stack
        handle_node(start, &mut tree, &mut stack, &mut storage, NO_PARENT, &sz);
        while let Some(current) = stack.pop() {
            let char = current.1.get_internal();
            let iter = if reverse {
                char.get_parent()
            } else {
                char.get_children()
            };
            for child in iter {
                handle_node(
                    child.clone(),
                    &mut tree,
                    &mut stack,
                    &mut storage,
                    current.0,
                    &sz,
                );
            }
        }

        tree.layout(); //this calls the layout algorithm

        let root;

        let mut groups: HashMap<&str, RGBColor> = HashMap::new(); //class groups
        let mut positions = HashMap::new();
        {
            //convert the tree layout to a hashmap and apply a 'scale' to the drawing area
            let layout = tree.get_pos(); //this isn't documented, but this function dumps the layout into a 3xN matrix (id, x, y)

            //we need to find the area that the tree laying algorithm uses to draw the tree
            let mut min_x = 0.0;
            let mut max_x = 0.0;
            let mut min_y = 0.0;
            let mut max_y = 0.0;
            for i in 0..layout.len() / 3 {
                let id = layout[i * 3] as usize;
                let x = layout[i * 3 + 1];
                let y = layout[i * 3 + 2];
                let node_data = storage.get(&id).unwrap();
                let class = &node_data.4;
                if class.is_some() {
                    // group resolving
                    let class = class.as_ref().unwrap();
                    if !groups.contains_key(class.as_str()) {
                        let mut rng = thread_rng();
                        let base: u8 = 85;
                        let mut color = RGBColor(base, base, base);
                        //pick a random color and make it dominant
                        let index = rng.gen_range(0..3);
                        let add = rng.gen_range(160 - base..255 - base);
                        match index {
                            0 => {
                                color.0 += add;
                            }
                            1 => {
                                color.1 += add;
                            }
                            2 => {
                                color.2 += add;
                            }
                            _ => unreachable!(),
                        }
                        groups.insert(class.as_str(), color);
                    }
                }
                // canvas size resolving
                positions.insert(id, (x, y));
                if x < min_x || min_x == 0.0 {
                    min_x = x - node_data.3 .0 as f64;
                }
                if x > max_x {
                    max_x = x + node_data.3 .0 as f64;
                }
                if y < min_y {
                    min_y = y - node_data.3 .1 as f64;
                }
                if y > max_y {
                    max_y = y + node_data.3 .1 as f64;
                }
            }

            min_x *= 1.02;
            max_x *= 1.02;

            let x_size = (max_x - min_x + 10.0) as u32;
            let y_size = (max_y - min_y + 10.0) as u32;

            /* Note on scaling
            I did try, and I mean TRY HARD to get the scaling to work properly, but Plotters doesn't allow me to properly square rectangles.
            Their size is in i32, which means when we try to render a tree 10k units wide the rectangle size of 0.0001 is 0.
            This is a limitation of the library, and I can't do anything about it.
            */

            let root_raw = SVGBackend::new(output_path, (x_size, y_size)).into_drawing_area();

            root_raw.fill(&WHITE).unwrap();

            root = root_raw.apply_coord_spec(Cartesian2d::<RangedCoordf64, RangedCoordf64>::new(
                min_x..max_x,
                min_y..max_y,
                (
                    (x_size / 100) as i32..((x_size / 100) * 99) as i32,
                    (y_size / 25) as i32..(y_size / 25 * 24) as i32,
                ),
            ));
        }
        //we first draw the lines. Lines go from middle points of the nodes to the middle point of the parent nodes
        for (id, (x, y)) in &positions {
            let node_data = storage.get(&id).unwrap();
            if node_data.0 != NO_PARENT {
                //draw the line if applicable
                let (parent_x, parent_y) = positions.get(&node_data.0).unwrap();
                //MAYBE improve the line laying algorithm, but it's not that important
                root.draw(&PathElement::new(
                    vec![
                        (*x, *y - (node_data.2 .1 / 2.0)),
                        (*parent_x, *parent_y + (node_data.2 .1 / 2.0)),
                    ],
                    Into::<ShapeStyle>::into(&BLACK).stroke_width(1),
                ))
                .unwrap();
            }
        }
        //then we draw the nodes so that they lay on top of the lines
        for (id, (x, y)) in &positions {
            let node_data = storage.get(&id).unwrap();
            let class = &node_data.4;
            let color = if class.is_some() {
                groups.get(class.as_ref().unwrap().as_str()).unwrap()
            } else {
                &WHITE
            };
            //draw the element after the line so that the line is behind the element
            root.draw(
                &(EmptyElement::at((*x, *y))
                // the rectangle is defined by two points, the top left and the bottom right. We calculate the top left by subtracting half the size of the node from the center point
                + Rectangle::new(
                    [
                        (-(node_data.2.0 as i32) / 2, -(node_data.2.1 as i32) / 2),
                        (node_data.2.0 as i32 / 2, node_data.2.1 as i32 / 2)
                    ],
                    Into::<ShapeStyle>::into(color.mix(0.9)).filled(),
                //we add the text to the node, the text is drawn at the point we calculated earlier
                ) + Text::new(
                    node_data.1.as_str().to_owned(),
                    node_data.3,
                    fnt.clone(),
            )),
            )
            .unwrap();
        }
        root.present().unwrap();
    }

    /// Creates a dynasty graph, meaning the family tree graph
    pub fn create_dynasty_graph(&self, dynasty: &Dynasty, output_path: &str) {
        //we get the founder and use it as root
        let founder = dynasty.get_founder();
        self.create_tree_graph::<Character>(founder, false, output_path)
    }

    /// Creates a death graph for a culture
    pub fn create_culture_graph(&self, culture_id: GameId, output_path: &str) {
        let data = self.culture_graph_complete.get(&culture_id);
        if data.is_none() {
            return;
        }
        create_graph(data.unwrap(), output_path)
    }

    /// Creates a death graph for a faith
    pub fn create_faith_graph(&self, faith_id: GameId, output_path: &str) {
        let data = self.faith_graph_complete.get(&faith_id);
        if data.is_none() {
            return;
        }
        create_graph(data.unwrap(), output_path)
    }

    pub fn create_timeline_graph(
        timespans: &Vec<(Shared<Title>, Vec<(u32, u32)>)>,
        events: &Vec<(
            u32,
            Shared<Character>,
            Shared<Title>,
            GameString,
            RealmDifference,
        )>,
        max_date: u32,
        output_path: &str,
    ) {
        let root = SVGBackend::new(output_path, GRAPH_SIZE).into_drawing_area();

        root.fill(&WHITE).unwrap();

        let t_len = timespans.len() as i32;
        let fnt = ("sans-serif", 10.0).into_font();
        let lifespan_y = fnt.box_size("L").unwrap().1 as i32;
        const MARGIN: i32 = 3;
        let height = lifespan_y * t_len + MARGIN;

        let root = root.apply_coord_spec(Cartesian2d::<RangedCoordu32, RangedCoordi32>::new(
            0..max_date,
            -height..height,
            (0..GRAPH_SIZE.0 as i32, 0..GRAPH_SIZE.1 as i32),
        ));

        root.draw(&PathElement::new(
            [(0, 0), (max_date, 0)],
            Into::<ShapeStyle>::into(&BLACK).filled(),
        ))
        .unwrap();
        const YEAR_INTERVAL: u32 = 25;
        //draw the tick
        for i in 0..max_date / YEAR_INTERVAL {
            root.draw(&PathElement::new(
                [
                    (i * YEAR_INTERVAL + 1, -height),
                    (i * YEAR_INTERVAL, MARGIN),
                ],
                Into::<ShapeStyle>::into(&BLACK).filled(),
            ))
            .unwrap();
        }
        //draw the century labels
        for i in 1..(max_date / 100) + 1 {
            let txt = (i * 100).to_string();
            let txt_x = fnt.box_size(&txt).unwrap().0 as u32;
            root.draw(&Text::new(
                txt,
                (i * 100 - (txt_x / 2), MARGIN),
                fnt.clone(),
            ))
            .unwrap();
        }
        //draw the empire lifespans
        for (i, (title, data)) in timespans.iter().enumerate() {
            let mut txt_x = 0;
            for (start, end) in data {
                if *start < txt_x || txt_x == 0 {
                    txt_x = *start;
                }
                let real_end;
                if *end == 0 {
                    real_end = max_date;
                } else {
                    real_end = *end;
                }
                root.draw(&Rectangle::new(
                    [
                        (*start, -lifespan_y * i as i32 - MARGIN),
                        (real_end, -lifespan_y * (i + 1) as i32 - MARGIN),
                    ],
                    Into::<ShapeStyle>::into(&GREEN).filled(),
                ))
                .unwrap();
            }
            root.draw(&Text::new(
                title.get_internal().get_name().to_string(),
                (txt_x, -lifespan_y * (i + 1) as i32),
                fnt.clone(),
            ))
            .unwrap();
        }
        let mut lane: Vec<u32> = Vec::new();
        //draw the events
        for (date, char, title, group_desc, difference) in events.iter() {
            let title_name = title.get_internal().get_name();
            let char_name = char.get_internal().get_name();
            let txt = format!(
                "{} conquered {} for the {} {}",
                char_name,
                title_name,
                difference.get_name(),
                group_desc
            );
            let txt_x = date - fnt.box_size(&txt).unwrap().0 as u32;
            let mut y = 0;
            let mut found = false;
            //find the lane that has space for us
            for (j, lane) in lane.iter_mut().enumerate() {
                if *lane < txt_x {
                    y = j as u32;
                    found = true;
                    *lane = *date + txt_x;
                    break;
                }
            }
            //if we havent found one then we create a new lane
            if !found {
                y = lane.len() as u32;
                if y as i32 * lifespan_y > height {
                    //if the lane is out of bounds we skip the event
                    continue;
                }
                lane.push(*date + txt_x);
            }
            root.draw(&Text::new(
                txt,
                (txt_x, lifespan_y * (y + 1) as i32),
                fnt.clone(),
            ))
            .unwrap();
            root.draw(&PathElement::new(
                [(*date, lifespan_y * (y + 1) as i32), (*date, 0)],
                Into::<ShapeStyle>::into(&BLACK).filled(),
            ))
            .unwrap();
            root.draw(&Circle::new(
                (*date, 0),
                2,
                Into::<ShapeStyle>::into(&RED).filled(),
            ))
            .unwrap();
        }
        root.present().unwrap();
    }
}

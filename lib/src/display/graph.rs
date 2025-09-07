use std::collections::HashMap;

use rand::{rng, Rng};

use plotters::{
    coord::types::{RangedCoordf64, RangedCoordi32},
    prelude::*,
};

// This is a cool little library that provides the TREE LAYOUT ALGORITHM, the rendering is done by plotters
//https://github.com/zxch3n/tidy/tree/master it is sort of tiny so here github link in case it goes down
use tidy_tree::TidyTree;

use super::super::save_file::{
    parser::types::{GameId, GameString, Wrapper},
    structures::{Dynasty, FromGameObject, GameObjectDerived, GameRef, Title},
};

use std::{collections::BTreeMap, path::Path};

/// Common graph size in pixels
const GRAPH_SIZE: (u32, u32) = (1024, 768);

/// A value indicating that the node has no parent
const NO_PARENT: usize = usize::MAX;

const GRAPH_MARGIN: u32 = 5;
const GRAPH_LABEL_SPACE: u32 = GRAPH_MARGIN * 10;

const TREE_MARGIN: u32 = 10;
const TREE_NODE_SIZE_MULTIPLIER: f64 = 1.5;
const PARENT_CHILD_MARGIN: f64 = 15.0;
const PEER_MARGIN: f64 = 5.0;

const TIMELINE_MARGIN: u32 = 3;

/// The y label for the death graphs
const Y_LABEL: &str = "Percentage of global deaths";

/// The maximum y value for the death graphs
const MAX_Y: f64 = 100.0;
const MIN_Y: f64 = 0.0;

/// Handles node initialization within the graph.
/// Tree is the tree object we are adding the node to, stack is the stack we are using to traverse the tree, storage is the hashmap we are using to store the node data, fnt is the font we are using to calculate the size of the node, and parent is the parent node id.
fn handle_node<
    I: IntoIterator<Item = GameRef<T>>,
    T: TreeNode<I> + GameObjectDerived + FromGameObject,
>(
    node: GameRef<T>,
    tree: &mut TidyTree,
    stack: &mut Vec<(usize, GameRef<T>)>,
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
    fnt: &FontDesc,
) {
    let obj = node.get_internal();
    let id = obj.get_id() as usize;
    if let Some(ch) = obj.inner() {
        let name = ch.get_name();
        let txt_size = fnt.box_size(&name).unwrap();
        let node_width = txt_size.0 as f64 * TREE_NODE_SIZE_MULTIPLIER;
        let node_height = txt_size.1 as f64 * TREE_NODE_SIZE_MULTIPLIER;
        //we also here calculate the point where the text should be drawn while we have convenient access to both size with margin and without
        let txt_point = (
            -(node_width as i32 - txt_size.0 as i32),
            -(node_height as i32 - txt_size.1 as i32),
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
    }
}

/// A trait for objects that can be used in a tree structure
pub trait TreeNode<I: IntoIterator>: Sized {
    /// Returns an iterator over the children of the node
    fn get_children(&self) -> Option<I>;

    /// Returns an iterator over the parent of the node
    fn get_parent(&self) -> Option<I>;

    /// Returns the class of the node
    fn get_class(&self) -> Option<GameString>;
}

/// Creates a graph from a given data set
/// Assumes that the data is sorted by the x value, and that the y value is a percentage
fn create_graph<P: AsRef<Path>, S: Into<String>>(
    data: &BTreeMap<i16, u32>,
    contast: &BTreeMap<i16, u32>,
    output_path: &P,
    ylabel: Option<S>,
    xlabel: Option<S>,
) {
    let mut min_x: i16 = 0;
    let mut max_x: i16 = 0;
    {
        let mut iter = data.iter();
        if let Some((x, _)) = iter.next() {
            min_x = *x;
        }
        if let Some((x, _)) = iter.next_back() {
            max_x = *x;
        }
    }

    let root = SVGBackend::new(output_path, GRAPH_SIZE).into_drawing_area();
    root.fill(&WHITE).unwrap();
    let mut chart = ChartBuilder::on(&root)
        .margin(GRAPH_MARGIN)
        .x_label_area_size(GRAPH_LABEL_SPACE)
        .y_label_area_size(GRAPH_LABEL_SPACE)
        .build_cartesian_2d((min_x as i32)..(max_x as i32), MIN_Y..MAX_Y)
        .unwrap();

    let mut mesh = chart.configure_mesh();

    if let Some(xlabel) = xlabel {
        mesh.x_desc(xlabel);
    }

    if let Some(ylabel) = ylabel {
        mesh.y_desc(ylabel);
    }

    mesh.draw().unwrap();

    chart
        .draw_series(LineSeries::new(
            (min_x..max_x).map(|year| {
                (
                    year as i32,
                    *data.get(&year).unwrap_or(&0) as f64 / *contast.get(&year).unwrap() as f64
                        * MAX_Y,
                )
            }),
            &RED,
        ))
        .unwrap();
}

/// An object that can create graphs from the game state
pub struct Grapher {
    /// Stored graph data for all faiths, certainly less memory efficient but the speed is worth it
    faith_graph_complete: HashMap<GameId, BTreeMap<i16, u32>>,
    culture_graph_complete: HashMap<GameId, BTreeMap<i16, u32>>,
    total_deaths: BTreeMap<i16, u32>,
}

impl Grapher {
    pub fn new(
        faith_death_data: HashMap<GameId, BTreeMap<i16, u32>>,
        culture_death_data: HashMap<GameId, BTreeMap<i16, u32>>,
        total_deaths: BTreeMap<i16, u32>,
    ) -> Self {
        Grapher {
            faith_graph_complete: faith_death_data,
            culture_graph_complete: culture_death_data,
            total_deaths,
        }
    }

    /// Creates a tree graph from a given node
    /// The reverse parameter determines if the tree is drawn from the parent to the children or the other way around
    pub fn create_tree_graph<
        I: IntoIterator<Item = GameRef<T>>,
        T: TreeNode<I> + GameObjectDerived + FromGameObject,
        P: AsRef<Path>,
    >(
        &self,
        start: GameRef<T>, // the root node that is a TreeNode
        reverse: bool,
        output_path: &P,
    ) {
        let mut tree = TidyTree::with_tidy_layout(PARENT_CHILD_MARGIN, PEER_MARGIN);
        //tree nodes don't have any data attached to them, so we need to store the data separately
        let mut storage = HashMap::default();
        let fnt = ("sans-serif", 6.66 * TREE_NODE_SIZE_MULTIPLIER).into_font();
        let mut stack = Vec::new(); //BFS stack
        handle_node(start, &mut tree, &mut stack, &mut storage, NO_PARENT, &fnt);
        while let Some(current) = stack.pop() {
            if let Some(char) = current.1.get_internal().inner() {
                let iter = if reverse {
                    if let Some(parent) = char.get_parent() {
                        parent
                    } else {
                        continue;
                    }
                } else {
                    if let Some(children) = char.get_children() {
                        children
                    } else {
                        continue;
                    }
                };
                for el in iter {
                    handle_node(el, &mut tree, &mut stack, &mut storage, current.0, &fnt);
                }
            }
        }

        tree.layout(); //this calls the layout algorithm

        let root;

        let mut groups: HashMap<&str, RGBColor> = HashMap::default(); //class groups
        let mut positions: HashMap<usize, (f64, f64)> = HashMap::default();
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
                let (_, _, (node_width, node_height), _, class) = storage.get(&id).unwrap();
                if let Some(class) = class {
                    // group resolving
                    if !groups.contains_key(class.as_ref()) {
                        let mut rng = rng();
                        let base: u8 = 85;
                        let mut color = RGBColor(base, base, base);
                        //pick a random color and make it dominant
                        let index = rng.random_range(0..3);
                        let add = rng.random_range(160 - base..255 - base);
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
                        groups.insert(class.as_ref(), color);
                    }
                }
                // canvas size resolving
                positions.insert(id, (x, y));
                let candidate_x = x - node_width;
                if candidate_x < min_x || min_x == 0.0 {
                    min_x = candidate_x;
                }
                let candidate_x = x + node_width;
                if candidate_x > max_x {
                    max_x = candidate_x;
                }
                let candidate_y = y - node_height;
                if candidate_y < min_y {
                    min_y = candidate_y;
                }
                let candidate_y = y + node_height;
                if candidate_y > max_y {
                    max_y = candidate_y;
                }
            }

            let x_size = (max_x - min_x + (TREE_MARGIN as f64 * 2.0)) as u32;
            let y_size = (max_y - min_y + (TREE_MARGIN as f64 * 2.0)) as u32;

            /* NOTE on scaling
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
                    TREE_MARGIN as i32..(x_size - TREE_MARGIN) as i32,
                    (y_size / 25) as i32..(y_size / 25 * 24) as i32,
                ),
            ));
        }
        //we first draw the lines. Lines go from middle points of the nodes to the middle point of the parent nodes
        for (id, (x, y)) in &positions {
            let (parent, _, (_, node_height), _, _) = storage.get(id).unwrap();
            if *parent != NO_PARENT {
                //draw the line if applicable
                let (parent_x, parent_y) = positions.get(parent).unwrap();
                //MAYBE improve the line laying algorithm, but it's not that important
                root.draw(&PathElement::new(
                    vec![
                        (*x, *y - (node_height / 2.0)),
                        (*parent_x, *parent_y + (node_height / 2.0)),
                    ],
                    Into::<ShapeStyle>::into(&BLACK).stroke_width(1),
                ))
                .unwrap();
            }
        }
        //then we draw the nodes so that they lay on top of the lines
        for (id, (x, y)) in &positions {
            let (_, node_name, (node_width, node_height), txt_point, class) =
                storage.get(id).unwrap();
            let color = if let Some(class) = class {
                groups.get(class.as_ref()).unwrap()
            } else {
                &WHITE
            };
            //draw the element after the line so that the line is behind the element
            root.draw(
                &(EmptyElement::at((*x, *y))
                // the rectangle is defined by two points, the top left and the bottom right. We calculate the top left by subtracting half the size of the node from the center point
                + Rectangle::new(
                    [
                        (-(*node_width as i32) / 2, -(*node_height as i32) / 2),
                        (*node_width as i32 / 2, *node_height as i32 / 2)
                    ],
                    Into::<ShapeStyle>::into(color.mix(0.9)).filled(),
                //we add the text to the node, the text is drawn at the point we calculated earlier
                ) + Text::new(
                    node_name.clone(),
                    *txt_point,
                    fnt.clone(),
            )),
            )
            .unwrap();
        }
        root.present().unwrap();
    }

    /// Creates a dynasty graph, meaning the family tree graph
    pub fn create_dynasty_graph<P: AsRef<Path>>(&self, dynasty: &Dynasty, output_path: &P) {
        //we get the founder and use it as root
        self.create_tree_graph(dynasty.get_founder(), false, output_path)
    }

    pub fn create_culture_graph<P: AsRef<Path>>(&self, culture_id: GameId, output_path: &P) {
        if let Some(data) = self.culture_graph_complete.get(&culture_id) {
            create_graph(data, &self.total_deaths, output_path, Some(Y_LABEL), None)
        }
    }

    pub fn create_faith_graph<P: AsRef<Path>>(&self, faith_id: GameId, output_path: &P) {
        if let Some(data) = self.faith_graph_complete.get(&faith_id) {
            create_graph(data, &self.total_deaths, output_path, Some(Y_LABEL), None)
        }
    }
}

pub fn create_timeline_graph<P: AsRef<Path>>(
    timespans: &Vec<(GameRef<Title>, Vec<(i16, i16)>)>,
    max_date: i16,
    output_path: P,
) {
    let root = SVGBackend::new(&output_path, GRAPH_SIZE).into_drawing_area();

    root.fill(&WHITE).unwrap();

    let t_len = timespans.len() as i32;
    let fnt = ("sans-serif", 10.0).into_font();
    let lifespan_y = fnt.box_size("L").unwrap().1 as i32;
    let height = lifespan_y * t_len + TIMELINE_MARGIN as i32;

    let root = root.apply_coord_spec(Cartesian2d::<RangedCoordi32, RangedCoordi32>::new(
        0..max_date as i32,
        -height..TIMELINE_MARGIN as i32 * 3,
        (0..GRAPH_SIZE.0 as i32, 0..GRAPH_SIZE.1 as i32),
    ));

    root.draw(&PathElement::new(
        [(0, 0), (max_date as i32, 0)],
        Into::<ShapeStyle>::into(&BLACK).filled(),
    ))
    .unwrap();
    const YEAR_INTERVAL: i32 = 25;
    //draw the tick
    for i in 0..max_date as i32 / YEAR_INTERVAL {
        root.draw(&PathElement::new(
            [
                (i * YEAR_INTERVAL + 1, -height),
                (i * YEAR_INTERVAL, TIMELINE_MARGIN as i32),
            ],
            Into::<ShapeStyle>::into(&BLACK).filled(),
        ))
        .unwrap();
    }
    //draw the century labels
    for i in 1..(max_date as i32 / 100) + 1 {
        let txt = (i * 100).to_string();
        let txt_x = fnt.box_size(&txt).unwrap().0 as i32;
        root.draw(&Text::new(
            txt,
            (i * 100 - (txt_x / 2), TIMELINE_MARGIN as i32),
            fnt.clone(),
        ))
        .unwrap();
    }
    //draw the empire lifespans
    for (i, (title, data)) in timespans.iter().enumerate() {
        if let Some(title) = title.get_internal().inner() {
            let mut txt_x = 0;
            for (start, end) in data {
                if *start < txt_x || txt_x == 0 {
                    txt_x = *start;
                }
                let real_end;
                if *end == 0 {
                    real_end = max_date as i32;
                } else {
                    real_end = *end as i32;
                }
                root.draw(&Rectangle::new(
                    [
                        (
                            *start as i32,
                            -lifespan_y * i as i32 - TIMELINE_MARGIN as i32,
                        ),
                        (
                            real_end,
                            -lifespan_y * (i + 1) as i32 - TIMELINE_MARGIN as i32,
                        ),
                    ],
                    Into::<ShapeStyle>::into(&GREEN).filled(),
                ))
                .unwrap();
            }
            root.draw(&Text::new(
                title.get_name(),
                (txt_x as i32, -lifespan_y * (i + 1) as i32),
                fnt.clone(),
            ))
            .unwrap();
        }
    }
    root.present().unwrap();
}

use std::collections::HashMap;

use crate::{game_object::{GameId, GameString}, game_state::GameState, structures::{Dynasty, GameObjectDerived}, types::{Shared, Wrapper}};
use plotters::{coord::types::RangedCoordf64, prelude::*};

// This is a cool little library that provides the TREE LAYOUT ALGORITHM, the rendering is done by plotters
//https://github.com/zxch3n/tidy/tree/master it is sort of tiny so here github link in case it goes down
use tidy_tree::TidyTree;

const GRAPH_SIZE:(u32, u32) = (1024, 768);

const TREE_SCALE:f64 = 1.5;

const NO_PARENT:usize = usize::MAX;

/// Handles node initialization within the graph.
/// Tree is the tree object we are adding the node to, stack is the stack we are using to traverse the tree, storage is the hashmap we are using to store the node data, fnt is the font we are using to calculate the size of the node, and parent is the parent node id.
fn handle_node<T:GameObjectDerived>(node:Shared<T>, tree:&mut TidyTree, stack:&mut Vec<(usize, Shared<T>)>, storage:&mut HashMap<usize, (usize, GameString, (f64, f64), (i32, i32))>, fnt:FontDesc, parent:usize) -> usize{
    let ch = node.get_internal();
    let id = ch.get_id() as usize;
    let name = ch.get_name().clone();
    //box_size returns the size the text will take up
    let sz = fnt.box_size(&name).unwrap();
    //we use this to determine the size of the node adding a margin
    let node_width = sz.0 as f64 * TREE_SCALE;
    let node_height = sz.1 as f64 * TREE_SCALE;
    //we also here calculate the point where the text should be drawn while we have convenient access to both size with margin and without
    let txt_point = (-(node_width as i32 - sz.0 as i32), -(node_height as i32 - sz.1 as i32));
    //add node to tree
    tree.add_node(id, node_width, node_height, parent);
    stack.push((id, node.clone()));
    //add aux data to storage because the nodes only store the id and no additional data
    storage.insert(id, (parent, name, (node_width, node_height), txt_point));
    return id;
}

/// An object that can create graphs from the game state
pub struct Grapher {
    /// Stored graph data for all faiths, certainly less memory efficient but the speed is worth it
    faith_graph_complete: HashMap<GameId, Vec<(u32, u32)>>,
    culture_graph_complete: HashMap<GameId, Vec<(u32, u32)>>,
}

impl Grapher{
    pub fn new(game_state:GameState) -> Self{
        Grapher{
            faith_graph_complete: game_state.get_faiths_graph_data(),
            culture_graph_complete: game_state.get_culture_graph_data(),
        }
    }

    /// Creates a dynasty graph, meaning the family tree graph
    pub fn create_dynasty_graph(&self, dynasty:&Dynasty, output_path:&str){
        let mut tree = TidyTree::with_tidy_layout(TREE_SCALE * 4.0, TREE_SCALE * 4.0);
        //tree nodes don't have any data attached to them, so we need to store the data separately
        let mut storage = HashMap::new();
        let fnt = ("sans-serif", 10.0).into_font();
        //BFS stack
        let mut stack = Vec::new();
        //we get the founder and use it as root
        let founder = dynasty.get_founder();
        handle_node(founder, &mut tree, &mut stack, &mut storage, fnt.clone(), NO_PARENT);
        while let Some(current) = stack.pop(){
            let char = current.1.get_internal();
            let children_iter = char.get_children_iter();
            for child in children_iter{
                handle_node(child.clone(), &mut tree, &mut stack, &mut storage, fnt.clone(), current.0);
            }
        }
        
        tree.layout(); //this calls the layout algorithm

        let root;

        let mut positions = HashMap::new();
        { //convert the tree layout to a hashmap and apply a 'scale' to the drawing area
            let layout = tree.get_pos(); //this isn't documented, but this function dumps the layout into a 3xN matrix (id, x, y)

            //we need to find the area that the tree laying algorithm uses to draw the tree
            let mut min_x = 0.0;
            let mut max_x = 0.0;
            let mut min_y = 0.0;
            let mut max_y = 0.0;
            for i in 0..layout.len() / 3{
                let id = layout[i * 3] as usize;
                let x = layout[i * 3 + 1];
                let y = layout[i * 3 + 2];
                positions.insert(id, (x, y));
                if x < min_x || min_x == 0.0{
                    min_x = x;
                }
                if x > max_x {
                    max_x = x;
                }
                if y < min_y {
                    min_y = y;
                }
                if y > max_y {
                    max_y = y;
                }
            }

            let x_size = ((max_x - min_x) * 1.005) as u32;
            let y_size = ((max_y - min_y) * 1.02) as u32;

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
                ((x_size / 100) as i32 .. ((x_size / 100) * 99) as i32, (y_size / 25) as i32 .. (y_size / 25 * 24) as i32),
            ));
        }
        //we first draw the lines. Lines go from middle points of the nodes to the middle point of the parent nodes
        for (id, (x, y)) in &positions{
            let node_data = storage.get(&id).unwrap();
            if node_data.0 != NO_PARENT{ //draw the line if applicable
                let (parent_x, parent_y) = positions.get(&node_data.0).unwrap();
                //MAYBE improve the line laying algorithm, but it's not that important
                root.draw(&PathElement::new(
                    vec![(*x, *y), (*parent_x, *parent_y)],
                    Into::<ShapeStyle>::into(&BLACK).stroke_width(1),
                )).unwrap();
            }
        }
        //then we draw the nodes so that they lay on top of the lines
        for (id, (x, y)) in &positions{
            let node_data = storage.get(&id).unwrap();
            //draw the element after the line so that the line is behind the element
            root.draw(
        &(EmptyElement::at((*x, *y)) 
                // the rectangle is defined by two points, the top left and the bottom right. We calculate the top left by subtracting half the size of the node from the center point
                + Rectangle::new(
                    [
                        (-(node_data.2.0 as i32) / 2, -(node_data.2.1 as i32) / 2), 
                        (node_data.2.0 as i32 / 2, node_data.2.1 as i32 / 2)
                    ], 
                    Into::<ShapeStyle>::into(&GREEN).filled(),
                //we add the text to the node, the text is drawn at the point we calculated earlier
                ) + Text::new(
                    format!("{}", node_data.1),
                    node_data.3,
                    fnt.clone(),
            ))).unwrap();
        }
        root.present().unwrap();
    }

    /// Creates a death graph for a culture
    pub fn create_culture_graph(&self, culture_id:GameId, output_path:&str){
        let data = self.culture_graph_complete.get(&culture_id).unwrap();

        let mut min_x:u32 = 0;
        let mut max_x:u32 = 0;
        let mut min_y:u32 = 0;
        let mut max_y:u32 = 0;
        for (x, y) in data {
            if *x < min_x || min_x == 0{
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
            .y_label_area_size(30).build_cartesian_2d(min_x..max_x, min_y..(max_y + 10)).unwrap();

        chart.configure_mesh().draw().unwrap();
        
        chart.draw_series(LineSeries::new(
            data.iter().map(|(x, y)| (*x, *y)),
            &RED,
        )).unwrap();
    }

    /// Creates a death graph for a faith
    pub fn create_faith_graph(&self, faith_id:GameId, output_path:&str){
        let data = self.faith_graph_complete.get(&faith_id).unwrap();

        let mut min_x:i32 = 0;
        let mut max_x:i32 = 0;
        let mut min_y = 0;
        let mut max_y = 0;
        for (x, y) in data {
            if (*x as i32) < min_x || min_x == 0{
                min_x = *x as i32;
            }
            if (*x as i32) > max_x {
                max_x = *x as i32;
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
            //.caption("Deaths of adherents through time", ("sans-serif", 50).into_font())
            .margin(5)
            .x_label_area_size(30)
            .y_label_area_size(30).build_cartesian_2d(min_x..max_x, min_y..max_y).unwrap();

        chart.configure_mesh().draw().unwrap();
        
        chart.draw_series(LineSeries::new(
            data.iter().map(|(x, y)| (*x as i32, *y)),
            &RED,
        )).unwrap();

        root.present().unwrap();
    }
}

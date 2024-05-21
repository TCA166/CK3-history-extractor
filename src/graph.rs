use std::collections::HashMap;

use crate::{game_object::GameId, game_state::GameState, structures::{Dynasty, GameObjectDerived}, types::Wrapper};
use plotters::{coord::types::RangedCoordf64, prelude::*};

// This is a cool little library that provides the TREE LAYOUT ALGORITHM, the rendering is done by plotters
//https://github.com/zxch3n/tidy/tree/master it is sort of tiny so here github link in case it goes down
use tidy_tree::TidyTree;

const GRAPH_SIZE:(u32, u32) = (1024, 768);

const TREE_SCALE:f64 = 1.5;

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

    pub fn create_dynasty_graph(&self, dynasty:&Dynasty, output_path:&str){
        let mut tree = TidyTree::with_layered_tidy(TREE_SCALE * 4.0, TREE_SCALE * 4.0);
        let founder = dynasty.get_founder();
        let mut storage = HashMap::new();
        let fnt = ("sans-serif", 10.0).into_font();
        let handle;
        {
            let nd = founder.get_internal();
            handle = nd.get_id() as usize;
            let name = nd.get_name().clone();
            let sz = fnt.box_size(&name).unwrap();
            let node_width = sz.0 as f64 * TREE_SCALE;
            let node_height = sz.1 as f64 * TREE_SCALE;
            let txt_point = (-(node_width as i32 - sz.0 as i32), -(node_height as i32 - sz.1 as i32));
            tree.add_node(handle, node_width, node_height, usize::MAX);
            storage.insert(handle, (usize::MAX, name, (node_width, node_height), txt_point));
        }
        let mut stack = vec![(handle, founder)];
        while let Some(current) = stack.pop(){
            let char = current.1.get_internal();
            let children_iter = char.get_children_iter();
            for child in children_iter{
                let ch = child.get_internal();
                let id = ch.get_id() as usize;
                let name = ch.get_name().clone();
                let sz = fnt.box_size(&name).unwrap();
                let node_width = sz.0 as f64 * TREE_SCALE;
                let node_height = sz.1 as f64 * TREE_SCALE;
                let txt_point = (-(node_width as i32 - sz.0 as i32), -(node_height as i32 - sz.1 as i32));
                tree.add_node(id, node_width, node_height, current.0);
                stack.push((id, child.clone()));
                storage.insert(id, (current.0, name, (node_width, node_height), txt_point));
            }
        }
        
        tree.layout(); //this calls the layout algorithm
        //the layout is complete, get the compressed layout where pos is a matrix 3xN (id, x, y)
        let layout = tree.get_pos();

        let mut min_x = 0.0;
        let mut max_x = 0.0;
        let mut min_y = 0.0;
        let mut max_y = 0.0;
        for i in 0..layout.len() / 3{
            let x = layout[i * 3 + 1];
            let y = layout[i * 3 + 2];
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

        let root = SVGBackend::new(output_path, (x_size, y_size)).into_drawing_area();

        root.fill(&WHITE).unwrap();

        let root = root.apply_coord_spec(Cartesian2d::<RangedCoordf64, RangedCoordf64>::new(
            min_x..max_x,
            min_y..max_y,
            ((x_size / 100) as i32 .. ((x_size / 100) * 99) as i32, (y_size / 25) as i32 .. (y_size / 25 * 24) as i32),
        ));
        
        //TODO add lines

        //foreach set of 3 values in the layout
        for i in 0..layout.len() / 3{
            let node = &layout[i * 3..i * 3 + 3];
            let node_data = storage.get(&(node[0] as usize)).unwrap();
            root.draw(&(EmptyElement::at((node[1], node[2])) + Rectangle::new(
        [
                    (-(node_data.2.0 as i32) / 2, -(node_data.2.1 as i32) / 2), 
                    (node_data.2.0 as i32 / 2, node_data.2.1 as i32 / 2)
                ],
                Into::<ShapeStyle>::into(&GREEN).filled(),
            ) + Text::new(
                format!("{}", node_data.1),
                node_data.3,
                fnt.clone(),
            ))).unwrap();
        }
        root.present().unwrap();
    }

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
            .caption("Deaths of culture members through time", ("sans-serif", 50).into_font())
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
            .caption("Deaths of adherents through time", ("sans-serif", 50).into_font())
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

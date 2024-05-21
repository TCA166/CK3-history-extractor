use std::collections::HashMap;

use crate::{game_object::GameId, game_state::GameState, structures::{Dynasty, GameObjectDerived}, types::Wrapper};

use layout::{backends::svg::SVGWriter, core::{base::Orientation, geometry::Point, style::StyleAttr, utils::save_to_file}, std_shapes::shapes::{Arrow, Element, ShapeKind}, topo::layout::VisualGraph};
use plotters::prelude::*;

const GRAPH_SIZE:(u32, u32) = (1024, 768);

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
        let mut vg = VisualGraph::new(Orientation::TopToBottom);
        let sz = Point::new(100., 100.);
        let edge = Arrow::simple("");
        let look = StyleAttr::simple();
        let founder = dynasty.get_founder();
        let handle;
        {
            let shape = ShapeKind::new_box(&founder.get_internal().get_name());
            let node = Element::create(shape, look.clone(), Orientation::LeftToRight, sz);
            handle = vg.add_node(node);
        }
        let mut stack = vec![(handle, founder)];
        while let Some(current) = stack.pop(){
            let char = current.1.get_internal();
            let children_iter = char.get_children_iter();
            for child in children_iter{
                let shape = ShapeKind::new_box(&child.get_internal().get_name());
                let node = Element::create(shape, look.clone(), Orientation::LeftToRight, sz);
                let handle = vg.add_node(node);
                vg.add_edge(edge.clone(), current.0, handle);
                stack.push((handle, child.clone()));
            }
        }
        //FIXME this is too slow
        let mut svg = SVGWriter::new();
        vg.do_it(false, false, false, &mut svg);
        save_to_file(output_path, &svg.finalize()).unwrap();
        
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

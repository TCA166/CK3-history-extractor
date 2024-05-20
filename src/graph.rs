use crate::{game_object::GameId, game_state::GameState};

use plotters::prelude::*;

/// An object that can create graphs from the game state
pub struct Grapher {
    game_state: GameState
}

impl Grapher{
    pub fn new(game_state:GameState) -> Self{
        Grapher{
            game_state
        }
    }

    /// Creates a death graph for a faith
    pub fn create_faith_graph(&self, faith_id:GameId, output_path:&str){
        let data = self.game_state.get_faith_graph_data(faith_id);

        let mut min_x:i32 = 0;
        let mut max_x:i32 = 0;
        let mut min_y = 0;
        let mut max_y = 0;
        for (x, y) in &data {
            if (*x as i32) < min_x {
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

        let root = SVGBackend::new(output_path, (1024, 768)).into_drawing_area();
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


        chart
            .configure_series_labels()
            .background_style(&WHITE.mix(0.8))
            .border_style(&BLACK)
            .draw().unwrap();

        root.present().unwrap();
    }
}

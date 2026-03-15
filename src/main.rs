mod colormap;
mod heatmap;
mod simhandler;
mod mpscsingle;

use colormap::BWColormap;
use heatmap::HeatmapPlot;

use core::f64;

use eframe::egui::{self, Id};
use ndarray::Array2;

use crate::{colormap::RainbowColormap, simhandler::SimulationHandler};

#[derive(Default,Clone)]
struct TestData {
    data: Array2<f64>
}

impl TestData {
    fn new(dimx: usize, dimy: usize) -> Self {
        Self { data: Array2::from_elem((dimx,dimy), 0.0) }
    }
}

impl simhandler::SimulationData for TestData {
    type SimRes = Array2<f64>;
    
    fn update(&mut self, ctx: &simhandler::SimulationContext) -> () {
        let t = ctx.get_step();
        let (rows, cols) = self.data.dim();
    
    // Define wave parameters
        let center_x = rows as f64 / 2.0;
        let center_y = cols as f64 / 2.0;
        let frequency = 0.2; // How tightly packed the rings are
        let speed = 0.1;     // How fast it expands per step

    // We use indexed_iter_mut to get coordinates and a mutable reference to each cell
        self.data.indexed_iter_mut().for_each(|((i, j), val)| {
            let dx = i as f64 - center_x;
            let dy = j as f64 - center_y;
        
            // Calculate Euclidean distance from center
            let r = (dx * dx + dy * dy).sqrt();
        
            // Radial sine wave formula: sin(distance - time)
            *val = (r * frequency - (t as f64) * speed).sin();
        });

    }

    fn send_result(&self, ctx: &simhandler::SimulationContext) -> Self::SimRes {
        self.data.clone()
    }
}

struct App {
    heatmap: HeatmapPlot,
    sim: SimulationHandler<TestData>
}

impl App {
    fn new(ctx: &eframe::CreationContext) -> Self {
        let mut hmap = HeatmapPlot::default();
        hmap.update_data(&generate_test_dataB(500, 500));
        let handle = SimulationHandler::new(TestData::new(800, 800))
            .send_frequency(1);
        App { heatmap: hmap, sim: handle }
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        
        self.sim.try_receive().inspect(|rec| {self.heatmap.update_data(rec);});

        egui::SidePanel::new(egui::panel::Side::Left, Id::new("side_panel")).show(ctx, |ui| {
            ui.label("Hello");
            ui.label("world");
            ui.label("success");
            if ui.button("Click!").clicked() {
                //self.heatmap.update_data(&generate_test_dataB(100, 100));
                self.sim.run();
            }
            if ui.button("Update!").clicked() {
                self.sim.try_receive().inspect(|rec| {self.heatmap.update_data(rec);});
            }
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            self.heatmap.plot_with_cmap(ui, &RainbowColormap::new(-1.0, 1.0))
        });

        ctx.request_repaint();
    }
}

fn generate_test_data(dimx: usize, dimy: usize) -> Array2<f64> {
    let center_x = dimx as f64 / 2.0;
    let center_y = dimy as f64 / 2.0;
    
    // Adjust the frequency to change how "tight" the rings are.
    // A smaller value means wider rings.
    let frequency = 0.2;

    Array2::from_shape_fn((dimx, dimy), |(i, j)| {
        let dx = i as f64 - center_x;
        let dy = j as f64 - center_y;
        
        // Calculate Euclidean distance from the center:
        // $d = \sqrt{(x - c_x)^2 + (y - c_y)^2}$
        let distance = (dx * dx + dy * dy).sqrt();

        // Calculate the sine and shift from [-1, 1] to [0, 1]
        ( (distance * frequency).sin() + 1.0 ) / 2.0
    })
}

fn generate_test_dataB(dimx: usize, dimy: usize) -> Array2<f64> {
    let scale = 0.1; // Controls the "tightness" of the waves

    Array2::from_shape_fn((dimx, dimy), |(i, j)| {
        let x = i as f64 * scale;
        let y = j as f64 * scale;

        // Combine various wave components:
        // 1. Horizontal wave
        let v1 = x.sin();
        // 2. Vertical wave
        let v2 = y.sin();
        // 3. Diagonal wave
        let v3 = (x + y).sin();
        // 4. A circular component for complexity
        let v4 = ((x * x + y * y).sqrt()).sin();

        // The raw sum ranges roughly from -4.0 to 4.0
        let combined = (v1 + v2 + v3 + v4) / 4.0;

        // Shift and scale to ensure the output is strictly [0.0, 1.0]
        (combined + 1.0) / 2.0
    })
}

fn main() {
    let native_options = eframe::NativeOptions::default();

    eframe::run_native("Heatmap test!", native_options, 
        Box::new(|cc| Ok(Box::new(App::new(cc))))
    );
}
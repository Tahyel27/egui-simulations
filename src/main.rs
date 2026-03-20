mod colormap;
mod heatmap;
mod simhandler;
mod mpscsingle;

use heatmap::{HeatmapPlot, HeatmapState};

use core::f64;

use eframe::egui::{self, Id, Pos2};
use ndarray::Array2;

use crate::colormap::RainbowColormap; 
use crate::simhandler::SimulationHandler;

#[derive(Default,Clone)]
struct TestData {
    data: Array2<f64>
}

#[derive(Default,Clone)]
struct TestParams {
    x: f64,
    y: f64,
}

impl TestData {
    fn new(dimx: usize, dimy: usize) -> Self {
        Self { data: Array2::from_elem((dimx,dimy), 0.0) }
    }
}

impl simhandler::SimulationData for TestData {
    type SimRes = Array2<f64>;

    type SimParams = TestParams;
    
    fn update(&mut self, ctx: &mut simhandler::SimulationContext<Self::SimParams>) -> () {
        let t = ctx.get_step();
        let (rows, cols) = self.data.dim();

        let center = ctx.get_params();
    
    // Define wave parameters
        let center_x = rows as f64 / 2.0 + center.x;
        let center_y = cols as f64 / 2.0 - center.y;
        let frequency = 0.2; // How tightly packed the rings are
        let speed = 0.1;   // How fast it expands per step

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

    fn send_result(&self, ctx: &simhandler::SimulationContext<Self::SimParams>) -> Self::SimRes {
        self.data.clone()
    }
}

struct App {
    sim: SimulationHandler<TestData>,
    test_pos: Pos2,
    heatmap: HeatmapState
}

impl App {
    fn new(ctx: &eframe::CreationContext) -> Self {
        let mut heatmap = HeatmapState::default();
        heatmap.render_data(&generate_test_data(500, 500), &ctx.egui_ctx, RainbowColormap::new(0.0, 1.0));
        let handle = SimulationHandler::new(TestData::new(500, 400), TestParams::default())
            .send_frequency(1);
        App { sim: handle, test_pos: Pos2::new(0., 0.), heatmap }
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        
        self.sim.try_receive().inspect(|rec| {self.heatmap.render_data(rec, ctx, RainbowColormap::new(-1.0, 1.0));});

        egui::SidePanel::new(egui::panel::Side::Left, Id::new("side_panel")).show(ctx, |ui| {
            ui.label("Hello");
            ui.label("world");
            ui.label(format!("Plot position: {} {}", self.test_pos.x, self.test_pos.y));
            if ui.button("Run!").clicked() {
                self.sim.run();
            }
            if ui.button("Pause!").clicked() {
                self.sim.pause();
            }
            if ui.button("Resume!").clicked() {
                self.sim.resume();
            }
            if ui.button("Modify!").clicked() {
                self.sim.modify_data( |data| *data = TestData { data: Array2::<f64>::zeros((200,200)) });
            }
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            let res = HeatmapPlot::new(&self.heatmap)
                .show_axes(true)
                .show_grid(true)
                .show(ui);
            if res.response.clicked() {
                if let Some(pos) = res.pos {
                    self.test_pos = pos.to_pos2();
                    self.sim.update_params(|p| {
                        p.x = pos.x;
                        p.y = pos.y;
                    });
                }
            }
            res.response
        });

        ctx.request_repaint();
    }
}

fn generate_test_data(dimx: usize, dimy: usize) -> Array2<f64> {
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
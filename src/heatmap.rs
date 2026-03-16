use crate::colormap::Colormap;

use eframe::egui::{self, Response, Ui, Vec2};
use egui_plot::{Plot, PlotImage, PlotPoint};
use ndarray::Array2;

#[derive(Default)]
struct DataGeneration {
    current_index: usize,
    old_index: usize
}

impl DataGeneration {
    fn increment(&mut self) {
        self.current_index = self.current_index + 1;
    }

    fn increment_and_validate(&mut self) {
        self.increment();
        self.validate();
    }

    fn is_valid(&self) -> bool {
        self.current_index == self.old_index
    }

    fn validate(&mut self) {
        self.old_index = self.current_index;
    }

    fn run_and_validate<F>(&mut self, f: F)
        where F: FnOnce() -> ()
    {
        if !self.is_valid() {
            f();
            self.validate();
        }
    }
}

pub struct HeatmapResponse {
    pub response: Response,
    pub pos: Option<PlotPoint>
}

pub struct HeatmapPlot {
    data: Array2<f64>,
    data_age: DataGeneration,
    texture: Option<egui::TextureHandle>,
    texture_age: DataGeneration,
    data_dims: (f32, f32),
    data_scale: f32
}

impl Default for HeatmapPlot {
    fn default() -> Self {
        Self { data: Default::default(),
            data_age: Default::default(), 
            texture: Default::default(), 
            texture_age: Default::default(), 
            data_dims: (1.0, 1.0),
            data_scale: 1.0
        }
    }
}

impl HeatmapPlot {
    pub fn update_texture<Cmap: Colormap>(&mut self, ctx: &egui::Context, cmap: &Cmap)
    {
        self.data_age.run_and_validate(|| {
            let (rows, cols) = self.data.dim();
            let mut pixels = Vec::with_capacity(rows * cols);
            /*self.data.indexed_iter().for_each(|((i,j),v)| {
                pixels.push(cmap.egui_color(*v));
            });*/
            //let pixels = self.data.iter().map(|v| cmap.egui_color(*v)).collect();

            /*for xaxis in self.data.outer_iter() {
                let xaxis: ArrayView1<f64> = xaxis;
                pixels.extend(xaxis.iter().map(|v| cmap.egui_color(*v)));
            }*/

            pixels.extend(self.data.t().iter().map(|v| cmap.egui_color(*v)));

            let image = egui::ColorImage::new([rows, cols], pixels);

            let handle = ctx.load_texture("heatmap_texture", image,  Default::default());

            self.texture = Some(handle);

            self.texture_age.increment_and_validate();
        });

    }

    pub fn update_data(&mut self, data: &Array2<f64>) {
        self.data = data.clone();
        self.data_age.increment();
        let (dimx, dimy) = data.dim();
        self.data_dims.0 = (dimx as f32) * self.data_scale;
        self.data_dims.1 = (dimy as f32) * self.data_scale;
    }

    pub fn plot(&self, ui: &mut Ui) -> HeatmapResponse {

        let response = match &self.texture {
            Some(texture) => {
                Plot::new("heatmap_plot")
                    .show_axes(true)
                    .data_aspect(1.0)
                    .show(ui, |plt_ut| {
                        plt_ut.image(PlotImage::new("heatmap_image", texture.id(),
                        PlotPoint::new(0.0, 0.0),  Vec2::new(self.data_dims.0, self.data_dims.1)));
                    })
            }
            None => {
                Plot::new("empty_heatmap_plot")
                    .show_axes(true)
                    .data_aspect(1.0)
                    .show(ui, |_| {})
            }
        };

        let pointer_pos = response.response.hover_pos();

        let pos = pointer_pos.map(|p_pos| response.transform.value_from_position(p_pos));

        HeatmapResponse { response: response.response, pos }
    }

    pub fn plot_with_cmap<Cmap: Colormap>(&mut self, ui: &mut egui::Ui, cmap: &Cmap) -> HeatmapResponse {
        self.update_texture(ui.ctx(), cmap);
        self.plot(ui)
    }

    pub fn set_scale(&mut self, scale: f64) {
        self.data_scale = scale as f32;
    }
}
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

#[derive(Default)]
pub struct HeatmapPlot {
    data: Array2<f64>,
    data_age: DataGeneration,
    texture: Option<egui::TextureHandle>,
    texture_age: DataGeneration
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

            pixels.extend(self.data.iter().map(|v| cmap.egui_color(*v)));

            let image = egui::ColorImage::new([rows, cols], pixels);

            let handle = ctx.load_texture("heatmap_texture", image,  Default::default());

            self.texture = Some(handle);

            self.texture_age.increment_and_validate();
        });

    }

    pub fn update_data(&mut self, data: &Array2<f64>) {
        self.data = data.clone();
        self.data_age.increment();
    }

    pub fn plot(&self, ui: &mut Ui) -> Response {

        match &self.texture {
            Some(texture) => {
                Plot::new("heatmap_plot")
                    .show_axes(true)
                    .data_aspect(1.0)
                    .show(ui, |plt_ut| {
                        plt_ut.image(PlotImage::new("heatmap_image", texture.id(),
                        PlotPoint::new(0.0, 0.0),  Vec2::new(10.0, 10.0)));
                    }).response
            }
            None => {
                Plot::new("empty_heatmap_plot")
                    .show_axes(true)
                    .data_aspect(1.0)
                    .show(ui, |_| {}).response
            }
        }
    }

    pub fn plot_with_cmap<Cmap: Colormap>(&mut self, ui: &mut egui::Ui, cmap: &Cmap) -> Response {
        self.update_texture(ui.ctx(), cmap);
        self.plot(ui)
    }
}
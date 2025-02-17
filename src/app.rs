use std::path::PathBuf;

use clap::Parser;
use eframe::CreationContext;
use egui_graphs::{DefaultGraphView, GraphView, SettingsInteraction, SettingsNavigation, SettingsStyle};
use glob::Pattern;

use crate::{js::find_project_root, Scanner, ScannerOptions};

#[derive(Parser, Debug)]
pub struct AppArgs {
    pub entry: PathBuf,

    #[arg(short, long)]
    pub cwd: Option<PathBuf>,
}

pub struct GuiApp {
    scanner: Scanner,
    includes: Vec<String>,
    filters: Vec<String>,
    args: AppArgs,
    graph: Option<egui_graphs::Graph<String>>,
}

impl GuiApp {
    pub fn new(cc: &CreationContext, args: AppArgs) -> Self {
        let opts = ScannerOptions::default();
        Self {
            scanner: Scanner::new(
                args.cwd.clone().unwrap_or(
                    find_project_root(&args.entry).expect("Could not find root directory"),
                ),
                opts.clone(),
            ),
            includes: opts
                .include
                .unwrap_or(vec![])
                .iter()
                .map(|p| p.to_string())
                .collect(),
            filters: opts.filter.iter().map(|p| p.to_string()).collect(),
            graph: None,
            args,
        }
    }
}

impl eframe::App for GuiApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui_extras::install_image_loaders(ctx);

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.horizontal_top(|ui| {
                ui.label("Filters");

                for text in self.filters.iter_mut() {
                    ui.text_edit_singleline(text);
                }

                if ui.button("Add").clicked() {
                    self.filters.push("".to_string());
                };
            });

            ui.horizontal_top(|ui| {
                ui.label("Includes");

                for text in self.includes.iter_mut() {
                    ui.text_edit_singleline(text);
                }

                if ui.button("Add").clicked() {
                    self.includes.push("".to_string());
                };
            });

            ui.horizontal_top(|ui| {
                if ui.button("Scan").clicked() {
                    self.scanner.set_filters(
                        self.filters
                            .iter()
                            .map(|s| Pattern::new(s).unwrap())
                            .collect(),
                    );
                    self.scanner.set_includes(
                        (self.includes.len() > 0)
                            .then_some(
                                self.includes
                                    .iter()
                                    .map(|s| Pattern::new(s).unwrap())
                                    .collect(),
                            )
                    );
                    self.graph = Some(egui_graphs::Graph::from(
                        &self.scanner.scan(&self.args.entry).build_petgraph(),
                    ));
                };
            });

            ui.separator();

            match &mut self.graph {
                Some(graph) => {
                    ui.add(
                        &mut GraphView::<String>::new(graph)
                            .with_styles(&SettingsStyle::new().with_labels_always(true))
                            .with_interactions(&SettingsInteraction::new().with_dragging_enabled(true))
                            .with_navigations(&SettingsNavigation::new().with_zoom_and_pan_enabled(true).with_fit_to_screen_enabled(true))
                    );
                }
                None => {
                    ui.label("No graph to display");
                }
            }
        });
    }
}

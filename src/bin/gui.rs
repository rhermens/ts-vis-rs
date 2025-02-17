use clap::Parser;
use eframe::NativeOptions;
use ts_vis_rs::app::{AppArgs, GuiApp};

fn main() {
    let args = AppArgs::parse();

    let _ = eframe::run_native("ts-vis", NativeOptions::default(), Box::new(|cc| {
        Ok(Box::new(GuiApp::new(cc, args)))
    }));
}

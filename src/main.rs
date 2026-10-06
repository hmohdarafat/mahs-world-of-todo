
mod combat;
mod config;
mod data;
mod hero;
mod items;
mod model;
mod persistence;
mod quests;
mod utils;
mod world;
mod ui;

use gtk::{glib, prelude::*};

fn main() -> glib::ExitCode {
    let app = gtk::Application::builder().application_id("dev.example.WowTodo").build();
    app.connect_activate(|app| {
        let ui = ui::Ui::build(app);
        ui.start();
    });
    app.run()
}


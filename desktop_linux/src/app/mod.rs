pub mod dialog;
pub mod handlers;
pub mod render;
pub mod state;
pub mod utils;
pub mod widgets;

use gtk::glib;
use gtk::prelude::*;

use self::handlers::connect_handlers;
use self::state::load_initial_state;
use self::utils::install_css;
use self::widgets::build_widgets;
use crate::core::DesktopCore;

const APP_ID: &str = "io.synap.desktop_linux";

pub fn run_app(core: std::rc::Rc<dyn DesktopCore>) -> glib::ExitCode {
    let app = gtk::Application::builder().application_id(APP_ID).build();

    app.connect_activate(move |app| {
        if let Some(window) = app.active_window() {
            window.present();
            return;
        }

        build_main_window(app, core.clone());
    });

    app.run()
}

fn build_main_window(app: &gtk::Application, core: std::rc::Rc<dyn DesktopCore>) {
    install_css();

    if let Some(settings) = gtk::Settings::default() {
        settings.set_gtk_interface_color_scheme(gtk::InterfaceColorScheme::Light);
    }

    let state = std::rc::Rc::new(std::cell::RefCell::new(load_initial_state(core.as_ref())));
    let widgets = build_widgets(app);

    render::render_from_state(&state, &widgets);
    connect_handlers(&widgets, state.clone(), core);

    widgets.window.present();
}

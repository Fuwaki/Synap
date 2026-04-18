mod app;
mod core;
mod domain;
mod usecase;

use std::rc::Rc;

use crate::core::{DesktopCore, SynapCoreAdapter};

fn main() -> gtk::glib::ExitCode {
    if std::env::var_os("GSK_RENDERER").is_none() {
        // Default to cairo for lower memory usage, but allow explicit overrides.
        unsafe {
            std::env::set_var("GSK_RENDERER", "cairo");
        }
    }

    let core = match SynapCoreAdapter::new_from_env() {
        Ok(core) => Rc::new(core) as Rc<dyn DesktopCore>,
        Err(error) => {
            eprintln!("failed to initialize desktop core: {error}");
            return gtk::glib::ExitCode::FAILURE;
        }
    };

    app::run_app(core)
}

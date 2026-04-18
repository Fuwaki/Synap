use gtk::prelude::*;

use crate::domain::NoteLayout;

pub fn clear_list_box(list_box: &gtk::ListBox) {
    while let Some(child) = list_box.first_child() {
        list_box.remove(&child);
    }
}

pub fn apply_layout_class(list_box: &gtk::ListBox, layout: NoteLayout) {
    list_box.remove_css_class("layout-waterfall");
    list_box.remove_css_class("layout-list");

    match layout {
        NoteLayout::Waterfall => list_box.add_css_class("layout-waterfall"),
        NoteLayout::List => list_box.add_css_class("layout-list"),
    }
}

pub fn set_button_active(button: &gtk::Button, active: bool) {
    if active {
        button.add_css_class("active");
    } else {
        button.remove_css_class("active");
    }
}

pub fn install_css() {
    let provider = gtk::CssProvider::new();
    provider.set_prefers_color_scheme(gtk::InterfaceColorScheme::Light);
    provider.load_from_string(include_str!("../style.css"));

    if let Some(display) = gtk::gdk::Display::default() {
        gtk::style_context_add_provider_for_display(
            &display,
            &provider,
            gtk::STYLE_PROVIDER_PRIORITY_APPLICATION,
        );
    }
}

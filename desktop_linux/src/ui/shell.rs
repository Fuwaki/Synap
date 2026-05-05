use adw::prelude::*;
use relm4::prelude::*;

use crate::{
    app::{App, message::AppMsg},
    domain::{AppState, ContentView, Theme},
};

pub struct ContentPages {
    pub content_stack: gtk::Stack,
    pub list_box: gtk::ListBox,
    pub empty_page: adw::StatusPage,
    pub detail_content_row: adw::ActionRow,
    pub detail_tags_row: adw::ActionRow,
    pub detail_meta_row: adw::ActionRow,
    pub detail_origins_box: gtk::Box,
    pub detail_replies_box: gtk::Box,
    pub detail_versions_box: gtk::Box,
    pub theme_dropdown: gtk::DropDown,
    pub tags_flow_box: gtk::FlowBox,
    pub timeline_container: gtk::Box,
    pub layout_stack: gtk::Stack,
    pub flow_box: gtk::FlowBox,
}

pub fn build_content_pages(state: &AppState, sender: &ComponentSender<App>) -> ContentPages {
    let content_stack = gtk::Stack::new();
    content_stack.set_hexpand(true);
    content_stack.set_vexpand(true);
    content_stack.set_margin_start(12);
    content_stack.set_margin_end(12);
    content_stack.set_margin_bottom(12);

    let list_box = gtk::ListBox::new();
    list_box.set_css_classes(&["boxed-list"]);
    list_box.set_selection_mode(gtk::SelectionMode::Single);
    list_box.set_vexpand(true);

    let (layout_stack, flow_box, notes_page) = build_notes_page(&list_box, sender);
    content_stack.add_named(&notes_page, Some("notes"));

    let empty_page = adw::StatusPage::new();
    empty_page.set_icon_name(Some("document-edit-symbolic"));
    content_stack.add_named(&empty_page, Some("empty"));

    let detail_page = build_detail_page(sender);
    content_stack.add_named(&detail_page.root, Some("detail"));

    let settings_page = build_settings_page(state.theme, sender);
    content_stack.add_named(&settings_page.root, Some("settings"));

    let tags_page = build_tags_page();
    content_stack.add_named(&tags_page.root, Some("tags"));

    let timeline_page = build_timeline_page();
    content_stack.add_named(&timeline_page.root, Some("timeline"));

    let initial_child = initial_content_child(state);
    content_stack.set_visible_child_name(initial_child);

    ContentPages {
        content_stack,
        list_box,
        empty_page,
        detail_content_row: detail_page.content_row,
        detail_tags_row: detail_page.tags_row,
        detail_meta_row: detail_page.meta_row,
        detail_origins_box: detail_page.origins_box,
        detail_replies_box: detail_page.replies_box,
        detail_versions_box: detail_page.versions_box,
        theme_dropdown: settings_page.theme_dropdown,
        tags_flow_box: tags_page.tags_flow_box,
        timeline_container: timeline_page.timeline_container,
        layout_stack,
        flow_box,
    }
}

pub fn install_css() {
    let provider = gtk::CssProvider::new();
    provider.load_from_string(include_str!("../style.css"));
    if let Some(display) = gtk::gdk::Display::default() {
        gtk::style_context_add_provider_for_display(
            &display,
            &provider,
            gtk::STYLE_PROVIDER_PRIORITY_APPLICATION,
        );
    }
}

fn build_notes_page(
    list_box: &gtk::ListBox,
    sender: &ComponentSender<App>,
) -> (gtk::Stack, gtk::FlowBox, gtk::ScrolledWindow) {
    let layout_stack = gtk::Stack::new();

    let list_scroller = gtk::ScrolledWindow::new();
    list_scroller.set_child(Some(list_box));
    layout_stack.add_named(&list_scroller, Some("list"));

    let waterfall_scroller = gtk::ScrolledWindow::new();
    waterfall_scroller.set_hscrollbar_policy(gtk::PolicyType::Never);
    let flow_box = gtk::FlowBox::new();
    flow_box.set_selection_mode(gtk::SelectionMode::Single);
    flow_box.set_homogeneous(false);
    flow_box.set_max_children_per_line(2);
    flow_box.set_column_spacing(12);
    flow_box.set_row_spacing(12);
    flow_box.set_margin_start(12);
    flow_box.set_margin_end(12);
    flow_box.set_margin_top(12);
    flow_box.set_margin_bottom(12);
    flow_box.set_hexpand(true);
    waterfall_scroller.set_child(Some(&flow_box));
    layout_stack.add_named(&waterfall_scroller, Some("waterfall"));

    let notes_scroller = gtk::ScrolledWindow::new();
    notes_scroller.set_child(Some(&layout_stack));

    let sender_scroll = sender.input_sender().clone();
    notes_scroller
        .vadjustment()
        .connect_value_changed(move |adj| {
            let upper = adj.upper();
            let page_size = adj.page_size();
            let value = adj.value();

            if upper > page_size && value >= upper - page_size - 100.0 {
                let _ = sender_scroll.send(AppMsg::LoadMoreNotes);
            }
        });

    (layout_stack, flow_box, notes_scroller)
}

struct DetailPage {
    root: gtk::ScrolledWindow,
    content_row: adw::ActionRow,
    tags_row: adw::ActionRow,
    meta_row: adw::ActionRow,
    origins_box: gtk::Box,
    replies_box: gtk::Box,
    versions_box: gtk::Box,
}

fn build_detail_page(sender: &ComponentSender<App>) -> DetailPage {
    let detail_clamp = adw::Clamp::builder()
        .maximum_size(800)
        .margin_top(24)
        .margin_bottom(24)
        .margin_start(24)
        .margin_end(24)
        .build();

    let detail_box = gtk::Box::new(gtk::Orientation::Vertical, 24);
    let detail_group = adw::PreferencesGroup::builder().title("笔记内容").build();

    let content_row = adw::ActionRow::builder()
        .title("内容")
        .subtitle_selectable(true)
        .build();
    let tags_row = adw::ActionRow::builder().title("标签").build();
    let meta_row = adw::ActionRow::builder().title("创建时间").build();

    detail_group.add(&content_row);
    detail_group.add(&tags_row);
    detail_group.add(&meta_row);

    let detail_buttons = gtk::Box::new(gtk::Orientation::Horizontal, 12);
    detail_buttons.set_halign(gtk::Align::End);

    let reply_button = gtk::Button::with_label("回复笔记");
    let sender_reply = sender.input_sender().clone();
    reply_button.connect_clicked(move |_| {
        let _ = sender_reply.send(AppMsg::ReplyToNote);
    });

    let edit_button = gtk::Button::with_label("编辑笔记");
    let sender_edit = sender.input_sender().clone();
    edit_button.connect_clicked(move |_| {
        let _ = sender_edit.send(AppMsg::EditNote);
    });

    let delete_button = gtk::Button::with_label("删除笔记");
    delete_button.add_css_class("destructive-action");
    let sender_delete = sender.input_sender().clone();
    delete_button.connect_clicked(move |_| {
        let _ = sender_delete.send(AppMsg::DeleteNote);
    });

    detail_buttons.append(&reply_button);
    detail_buttons.append(&edit_button);
    detail_buttons.append(&delete_button);

    let origins_box = build_detail_section("溯源链");
    let replies_box = build_detail_section("回复");
    let versions_box = build_detail_section("其他版本");

    detail_box.append(&detail_group);
    detail_box.append(&origins_box);
    detail_box.append(&replies_box);
    detail_box.append(&versions_box);
    detail_box.append(&detail_buttons);
    detail_clamp.set_child(Some(&detail_box));

    let root = gtk::ScrolledWindow::new();
    root.set_child(Some(&detail_clamp));

    DetailPage {
        root,
        content_row,
        tags_row,
        meta_row,
        origins_box,
        replies_box,
        versions_box,
    }
}

struct SettingsPage {
    root: adw::PreferencesPage,
    theme_dropdown: gtk::DropDown,
}

fn build_settings_page(theme: Theme, sender: &ComponentSender<App>) -> SettingsPage {
    let root = adw::PreferencesPage::new();
    let settings_group = adw::PreferencesGroup::builder().title("外观").build();

    let theme_row = adw::ActionRow::builder()
        .title("主题")
        .subtitle("选择应用的颜色主题")
        .build();

    let theme_dropdown = gtk::DropDown::from_strings(&["跟随系统", "浅色", "深色"]);
    theme_dropdown.set_selected(theme.index());
    let sender_theme = sender.input_sender().clone();
    theme_dropdown.connect_selected_notify(move |dropdown| {
        let _ = sender_theme.send(AppMsg::ThemeChanged(Theme::from_index(dropdown.selected())));
    });
    theme_row.add_suffix(&theme_dropdown);
    settings_group.add(&theme_row);
    root.add(&settings_group);

    SettingsPage {
        root,
        theme_dropdown,
    }
}

struct TagsPage {
    root: gtk::ScrolledWindow,
    tags_flow_box: gtk::FlowBox,
}

fn build_tags_page() -> TagsPage {
    let root = gtk::ScrolledWindow::new();
    let tags_box = gtk::Box::new(gtk::Orientation::Vertical, 12);
    tags_box.set_margin_top(24);
    tags_box.set_margin_bottom(24);
    tags_box.set_margin_start(24);
    tags_box.set_margin_end(24);

    let tags_label = gtk::Label::new(Some("所有标签"));
    tags_label.add_css_class("heading");
    tags_label.set_halign(gtk::Align::Start);
    tags_box.append(&tags_label);

    let tags_flow_box = gtk::FlowBox::new();
    tags_flow_box.set_selection_mode(gtk::SelectionMode::None);
    tags_flow_box.set_homogeneous(true);
    tags_flow_box.set_max_children_per_line(3);
    tags_flow_box.set_min_children_per_line(1);
    tags_flow_box.set_column_spacing(12);
    tags_flow_box.set_row_spacing(12);

    tags_box.append(&tags_flow_box);
    root.set_child(Some(&tags_box));

    TagsPage {
        root,
        tags_flow_box,
    }
}

struct TimelinePage {
    root: gtk::ScrolledWindow,
    timeline_container: gtk::Box,
}

fn build_timeline_page() -> TimelinePage {
    let root = gtk::ScrolledWindow::new();
    let timeline_box = gtk::Box::new(gtk::Orientation::Vertical, 24);
    timeline_box.set_margin_top(24);
    timeline_box.set_margin_bottom(24);
    timeline_box.set_margin_start(24);
    timeline_box.set_margin_end(24);

    let timeline_container = gtk::Box::new(gtk::Orientation::Vertical, 24);
    timeline_box.append(&timeline_container);

    root.set_child(Some(&timeline_box));

    TimelinePage {
        root,
        timeline_container,
    }
}

fn build_detail_section(title: &str) -> gtk::Box {
    let section = gtk::Box::new(gtk::Orientation::Vertical, 6);
    let label = gtk::Label::new(Some(title));
    label.add_css_class("heading");
    label.set_halign(gtk::Align::Start);
    section.append(&label);
    section
}

fn initial_content_child(state: &AppState) -> &'static str {
    let is_empty = state.visible_notes().is_empty();
    match state.content_view {
        ContentView::NoteDetail => "detail",
        ContentView::Settings => "settings",
        ContentView::Tags => "tags",
        ContentView::Timeline => "timeline",
        ContentView::TagNotes | ContentView::Notes | ContentView::Trash => {
            if is_empty {
                "empty"
            } else {
                "notes"
            }
        }
    }
}

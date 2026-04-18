use gtk::prelude::*;

use crate::domain::{NoteLayout, NoteListItemViewModel};

pub const SIDEBAR_WIDTH: i32 = 280;
pub const SIDEBAR_BUTTON_WIDTH: i32 = SIDEBAR_WIDTH * 4 / 5;

#[derive(Clone)]
pub struct AppWidgets {
    pub window: gtk::ApplicationWindow,
    pub page_title: gtk::Label,
    pub status_label: gtk::Label,
    pub toolbar_controls: gtk::Box,
    pub search_entry: gtk::SearchEntry,
    pub layout_dropdown: gtk::DropDown,
    pub content_stack: gtk::Stack,
    pub list_stack: gtk::Stack,
    pub list_box: gtk::ListBox,
    pub empty_title_label: gtk::Label,
    pub empty_body_label: gtk::Label,
    pub detail_meta_label: gtk::Label,
    pub detail_content_label: gtk::Label,
    pub detail_tags_label: gtk::Label,
    pub detail_edit_button: gtk::Button,
    pub detail_delete_button: gtk::Button,
    pub create_note_button: gtk::Button,
    pub trash_button: gtk::Button,
    pub settings_button: gtk::Button,
    pub back_to_list_button: gtk::Button,
}

pub fn build_widgets(app: &gtk::Application) -> AppWidgets {
    let window = gtk::ApplicationWindow::builder()
        .application(app)
        .title("Synap")
        .default_width(1360)
        .default_height(920)
        .build();
    window.add_css_class("app-shell");

    let root = gtk::Box::new(gtk::Orientation::Horizontal, 0);

    let sidebar = gtk::Box::new(gtk::Orientation::Vertical, 18);
    sidebar.add_css_class("sidebar");
    sidebar.set_width_request(SIDEBAR_WIDTH);

    let brand = gtk::Label::new(Some("Synap"));
    brand.add_css_class("brand");
    brand.set_xalign(0.0);

    let create_note_button = build_nav_button("新建笔记", "create-note-button");
    let trash_button = build_nav_button("回收站", "trash-button");
    let settings_button = build_nav_button("设置", "settings-button");
    let back_to_list_button = build_nav_button("返回笔记列表", "back-to-list-button");
    back_to_list_button.set_visible(false);

    let sidebar_spacer = gtk::Box::new(gtk::Orientation::Vertical, 0);
    sidebar_spacer.set_vexpand(true);

    sidebar.append(&brand);
    sidebar.append(&create_note_button);
    sidebar.append(&trash_button);
    sidebar.append(&settings_button);
    sidebar.append(&sidebar_spacer);
    sidebar.append(&back_to_list_button);

    let divider = gtk::Separator::new(gtk::Orientation::Vertical);
    divider.set_widget_name("main-divider");

    let content_pane = gtk::Box::new(gtk::Orientation::Vertical, 18);
    content_pane.add_css_class("content-pane");
    content_pane.set_hexpand(true);
    content_pane.set_vexpand(true);

    let toolbar = gtk::Box::new(gtk::Orientation::Horizontal, 16);

    let page_title = gtk::Label::new(Some("笔记列表"));
    page_title.add_css_class("section-title");
    page_title.set_xalign(0.0);

    let toolbar_spacer = gtk::Box::new(gtk::Orientation::Horizontal, 0);
    toolbar_spacer.set_hexpand(true);

    let toolbar_controls = gtk::Box::new(gtk::Orientation::Horizontal, 12);
    toolbar_controls.add_css_class("toolbar-controls");
    toolbar_controls.set_halign(gtk::Align::End);

    let search_entry = gtk::SearchEntry::new();
    search_entry.add_css_class("search-entry");
    search_entry.set_placeholder_text(Some("搜索内容或标签"));
    search_entry.set_width_chars(24);

    let layout_dropdown =
        gtk::DropDown::from_strings(&[NoteLayout::Waterfall.label(), NoteLayout::List.label()]);
    layout_dropdown.add_css_class("layout-selector-dropdown");
    layout_dropdown.set_width_request(172);
    layout_dropdown.set_selected(NoteLayout::Waterfall.index());
    install_layout_dropdown_factories(&layout_dropdown);

    toolbar_controls.append(&search_entry);
    toolbar_controls.append(&layout_dropdown);

    toolbar.append(&page_title);
    toolbar.append(&toolbar_spacer);
    toolbar.append(&toolbar_controls);

    let status_label = gtk::Label::new(None);
    status_label.add_css_class("status-label");
    status_label.set_xalign(0.0);
    status_label.set_visible(false);

    let list_box = gtk::ListBox::new();
    list_box.add_css_class("note-list");
    list_box.set_selection_mode(gtk::SelectionMode::Single);
    list_box.set_activate_on_single_click(true);
    list_box.set_vexpand(true);

    let list_scroller = gtk::ScrolledWindow::new();
    list_scroller.set_hexpand(true);
    list_scroller.set_vexpand(true);
    list_scroller.set_policy(gtk::PolicyType::Never, gtk::PolicyType::Automatic);
    list_scroller.set_overlay_scrolling(false);
    list_scroller.set_child(Some(&list_box));

    let list_stack = gtk::Stack::new();
    list_stack.set_hexpand(true);
    list_stack.set_vexpand(true);

    let (empty_placeholder, empty_title_label, empty_body_label) =
        build_placeholder("还没有笔记", "从左侧点击新建笔记，开始记录你的第一条内容。");

    list_stack.add_named(&list_scroller, Some("list"));
    list_stack.add_named(&empty_placeholder, Some("empty"));

    let content_stack = gtk::Stack::new();
    content_stack.set_hexpand(true);
    content_stack.set_vexpand(true);

    let detail_shell = gtk::Box::new(gtk::Orientation::Vertical, 18);
    detail_shell.add_css_class("detail-shell");
    detail_shell.set_hexpand(true);
    detail_shell.set_vexpand(true);

    let detail_actions = gtk::Box::new(gtk::Orientation::Horizontal, 10);
    detail_actions.set_halign(gtk::Align::End);

    let detail_edit_button = gtk::Button::with_label("编辑笔记");
    detail_edit_button.add_css_class("editor-secondary-action");

    let detail_delete_button = gtk::Button::with_label("删除笔记");
    detail_delete_button.add_css_class("detail-delete-button");

    detail_actions.append(&detail_edit_button);
    detail_actions.append(&detail_delete_button);

    let detail_meta_label = gtk::Label::new(None);
    detail_meta_label.add_css_class("detail-meta");
    detail_meta_label.set_xalign(0.0);

    let detail_content_label = gtk::Label::new(None);
    detail_content_label.add_css_class("detail-content");
    detail_content_label.set_xalign(0.0);
    detail_content_label.set_wrap(true);
    detail_content_label.set_selectable(true);

    let detail_tags_label = gtk::Label::new(None);
    detail_tags_label.add_css_class("detail-tags");
    detail_tags_label.set_xalign(0.0);
    detail_tags_label.set_wrap(true);
    detail_tags_label.set_selectable(true);

    detail_shell.append(&detail_actions);
    detail_shell.append(&detail_meta_label);
    detail_shell.append(&detail_content_label);
    detail_shell.append(&detail_tags_label);

    let (settings_placeholder, _, _) = build_placeholder(
        "设置正在整理中",
        "桌面 Linux 端的设置页会在后续阶段补齐，目前先保留占位。",
    );

    content_stack.add_named(&list_stack, Some("notes"));
    content_stack.add_named(&detail_shell, Some("detail"));
    content_stack.add_named(&settings_placeholder, Some("settings"));

    content_pane.append(&toolbar);
    content_pane.append(&status_label);
    content_pane.append(&content_stack);

    root.append(&sidebar);
    root.append(&divider);
    root.append(&content_pane);
    window.set_child(Some(&root));

    AppWidgets {
        window,
        page_title,
        status_label,
        toolbar_controls,
        search_entry,
        layout_dropdown,
        content_stack,
        list_stack,
        list_box,
        empty_title_label,
        empty_body_label,
        detail_meta_label,
        detail_content_label,
        detail_tags_label,
        detail_edit_button,
        detail_delete_button,
        create_note_button,
        trash_button,
        settings_button,
        back_to_list_button,
    }
}

pub fn build_nav_button(label: &str, widget_name: &str) -> gtk::Button {
    let button = gtk::Button::new();
    button.add_css_class("nav-button");
    button.set_widget_name(widget_name);
    button.set_halign(gtk::Align::Start);
    button.set_width_request(SIDEBAR_BUTTON_WIDTH);

    let title = gtk::Label::new(Some(label));
    title.add_css_class("nav-button-label");
    title.set_xalign(0.0);
    button.set_child(Some(&title));

    button
}

fn install_layout_dropdown_factories(dropdown: &gtk::DropDown) {
    let selected_factory = gtk::SignalListItemFactory::new();
    selected_factory.connect_setup(|_, object| {
        let Some(list_item) = object.downcast_ref::<gtk::ListItem>() else {
            return;
        };
        list_item.set_selectable(false);
        list_item.set_activatable(false);

        let content = gtk::Box::new(gtk::Orientation::Horizontal, 0);
        content.add_css_class("layout-selector-value-surface");
        content.set_hexpand(true);
        content.set_halign(gtk::Align::Fill);

        let label = gtk::Label::new(None);
        label.add_css_class("layout-selector-value-label");
        label.set_xalign(0.0);
        label.set_hexpand(true);
        content.append(&label);
        list_item.set_child(Some(&content));
    });
    selected_factory.connect_bind(|_, object| {
        let Some(list_item) = object.downcast_ref::<gtk::ListItem>() else {
            return;
        };
        let Some(content) = list_item.child().and_downcast::<gtk::Box>() else {
            return;
        };
        let Some(label) = content.first_child().and_downcast::<gtk::Label>() else {
            return;
        };
        let Some(item) = list_item.item().and_downcast::<gtk::StringObject>() else {
            return;
        };
        label.set_text(item.string().as_ref());
    });
    dropdown.set_factory(Some(&selected_factory));

    let list_factory = gtk::SignalListItemFactory::new();
    list_factory.connect_setup(|_, object| {
        let Some(list_item) = object.downcast_ref::<gtk::ListItem>() else {
            return;
        };
        let content = gtk::Box::new(gtk::Orientation::Horizontal, 0);
        content.add_css_class("layout-selector-item-surface");
        content.set_hexpand(true);
        content.set_halign(gtk::Align::Fill);

        let label = gtk::Label::new(None);
        label.add_css_class("layout-selector-item-label");
        label.set_xalign(0.0);
        label.set_hexpand(true);
        content.append(&label);
        list_item.set_child(Some(&content));
    });
    list_factory.connect_bind(|_, object| {
        let Some(list_item) = object.downcast_ref::<gtk::ListItem>() else {
            return;
        };
        let Some(content) = list_item.child().and_downcast::<gtk::Box>() else {
            return;
        };
        let Some(label) = content.first_child().and_downcast::<gtk::Label>() else {
            return;
        };
        let Some(item) = list_item.item().and_downcast::<gtk::StringObject>() else {
            return;
        };
        label.set_text(item.string().as_ref());
    });
    dropdown.set_list_factory(Some(&list_factory));
}

pub fn build_placeholder(title: &str, body: &str) -> (gtk::Box, gtk::Label, gtk::Label) {
    let outer = gtk::Box::new(gtk::Orientation::Vertical, 0);
    outer.set_hexpand(true);
    outer.set_vexpand(true);
    outer.set_halign(gtk::Align::Center);
    outer.set_valign(gtk::Align::Center);

    let card = gtk::Box::new(gtk::Orientation::Vertical, 8);
    card.add_css_class("placeholder-card");
    card.set_width_request(420);

    let title_label = gtk::Label::new(Some(title));
    title_label.add_css_class("placeholder-title");
    title_label.set_xalign(0.0);

    let body_label = gtk::Label::new(Some(body));
    body_label.add_css_class("placeholder-body");
    body_label.set_xalign(0.0);
    body_label.set_wrap(true);

    card.append(&title_label);
    card.append(&body_label);
    outer.append(&card);

    (outer, title_label, body_label)
}

pub fn build_note_row(note: &synap_core::dto::NoteDTO) -> gtk::ListBoxRow {
    let row = gtk::ListBoxRow::new();
    row.add_css_class("note-row");

    let card = gtk::Box::new(gtk::Orientation::Vertical, 10);
    card.add_css_class("note-card");

    let note_view = NoteListItemViewModel::from(note);
    let preview = gtk::Label::new(Some(&note_view.preview));
    preview.add_css_class("note-content");
    preview.set_xalign(0.0);
    preview.set_wrap(true);

    card.append(&preview);

    if !note.tags.is_empty() {
        let tags = note
            .tags
            .iter()
            .map(|tag| format!("#{tag}"))
            .collect::<Vec<_>>()
            .join("  ");
        let tags_label = gtk::Label::new(Some(&tags));
        tags_label.add_css_class("note-tags");
        tags_label.set_xalign(0.0);
        tags_label.set_wrap(true);
        card.append(&tags_label);
    }

    row.set_child(Some(&card));
    row
}

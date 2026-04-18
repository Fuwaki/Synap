use gtk::prelude::*;

use super::state::refresh_home;
use super::widgets::AppWidgets;
use crate::core::DesktopCore;
use crate::domain::{AppState, ContentView};

pub fn open_create_note_window(
    parent: &gtk::ApplicationWindow,
    state: std::rc::Rc<std::cell::RefCell<AppState>>,
    widgets: AppWidgets,
    core: std::rc::Rc<dyn DesktopCore>,
) {
    open_note_editor_window(
        parent,
        state,
        widgets,
        core,
        NoteEditorMode::Create,
        "新建笔记",
        "直接记录内容；标签可选，使用逗号分隔。",
        String::new(),
        Vec::new(),
    );
}

pub fn open_edit_note_window(
    parent: &gtk::ApplicationWindow,
    state: std::rc::Rc<std::cell::RefCell<AppState>>,
    widgets: AppWidgets,
    core: std::rc::Rc<dyn DesktopCore>,
    note_id: String,
    content: String,
    tags: Vec<String>,
) {
    open_note_editor_window(
        parent,
        state,
        widgets,
        core,
        NoteEditorMode::Edit { note_id },
        "编辑笔记",
        "修改正文或标签，保存后会立即更新详情。",
        content,
        tags,
    );
}

enum NoteEditorMode {
    Create,
    Edit { note_id: String },
}

impl NoteEditorMode {
    fn status_message(&self) -> &'static str {
        match self {
            Self::Create => "已创建笔记",
            Self::Edit { .. } => "已更新笔记",
        }
    }

    fn target_content_view(&self) -> ContentView {
        match self {
            Self::Create => ContentView::Notes,
            Self::Edit { .. } => ContentView::NoteDetail,
        }
    }

    fn is_create(&self) -> bool {
        matches!(self, Self::Create)
    }
}

fn open_note_editor_window(
    parent: &gtk::ApplicationWindow,
    state: std::rc::Rc<std::cell::RefCell<AppState>>,
    widgets: AppWidgets,
    core: std::rc::Rc<dyn DesktopCore>,
    mode: NoteEditorMode,
    title_text: &str,
    hint_text: &str,
    initial_content: String,
    initial_tags: Vec<String>,
) {
    let dialog = gtk::Window::builder()
        .title(title_text)
        .default_width(720)
        .default_height(560)
        .modal(true)
        .destroy_with_parent(true)
        .resizable(true)
        .transient_for(parent)
        .build();

    if let Some(app) = parent.application() {
        dialog.set_application(Some(&app));
    }

    dialog.add_css_class("editor-window");

    let shell = gtk::Box::new(gtk::Orientation::Vertical, 16);
    shell.add_css_class("editor-shell");
    shell.set_margin_top(24);
    shell.set_margin_bottom(24);
    shell.set_margin_start(24);
    shell.set_margin_end(24);
    shell.set_hexpand(true);
    shell.set_vexpand(true);

    let title = gtk::Label::new(Some(title_text));
    title.add_css_class("editor-title");
    title.set_xalign(0.0);

    let hint = gtk::Label::new(Some(hint_text));
    hint.add_css_class("editor-hint");
    hint.set_xalign(0.0);

    let content_buffer = gtk::TextBuffer::new(None);
    content_buffer.set_text(&initial_content);
    let content_view = gtk::TextView::with_buffer(&content_buffer);
    content_view.add_css_class("editor-text-view");
    content_view.set_widget_name("editor-text-view");
    content_view.set_wrap_mode(gtk::WrapMode::WordChar);
    content_view.set_top_margin(8);
    content_view.set_bottom_margin(8);
    content_view.set_left_margin(8);
    content_view.set_right_margin(8);
    content_view.set_vexpand(true);

    let content_scroller = gtk::ScrolledWindow::new();
    content_scroller.add_css_class("editor-scroll");
    content_scroller.set_hexpand(true);
    content_scroller.set_vexpand(true);
    content_scroller.set_min_content_height(320);
    content_scroller.set_policy(gtk::PolicyType::Never, gtk::PolicyType::Automatic);
    content_scroller.set_overlay_scrolling(false);
    content_scroller.set_child(Some(&content_view));

    let tags_entry = gtk::Entry::new();
    tags_entry.add_css_class("editor-tags-entry");
    tags_entry.set_placeholder_text(Some("标签，例如 rust, idea, diary"));
    tags_entry.set_text(&initial_tags.join(", "));

    let dialog_status = gtk::Label::new(None);
    dialog_status.add_css_class("status-label");
    dialog_status.set_xalign(0.0);
    dialog_status.set_visible(false);

    let actions = gtk::Box::new(gtk::Orientation::Horizontal, 10);
    actions.set_halign(gtk::Align::End);

    let cancel_button = gtk::Button::with_label("取消");
    cancel_button.add_css_class("editor-secondary-action");

    let save_button = gtk::Button::with_label("保存");
    save_button.add_css_class("editor-primary-action");

    actions.append(&cancel_button);
    actions.append(&save_button);

    shell.append(&title);
    shell.append(&hint);
    shell.append(&content_scroller);
    shell.append(&tags_entry);
    shell.append(&dialog_status);
    shell.append(&actions);

    dialog.set_child(Some(&shell));

    let dialog_for_cancel = dialog.clone();
    cancel_button.connect_clicked(move |_| {
        dialog_for_cancel.close();
    });

    let dialog_for_save = dialog.clone();
    let state_for_save = state.clone();
    let widgets_for_save = widgets.clone();
    let core_for_save = core.clone();
    let tags_entry_for_save = tags_entry.clone();
    let dialog_status_for_save = dialog_status.clone();
    let target_content_view = mode.target_content_view();
    let success_status = mode.status_message().to_string();
    let is_create = mode.is_create();
    save_button.connect_clicked(move |_| {
        let (start, end) = content_buffer.bounds();
        let content = content_buffer.text(&start, &end, false).to_string();
        let trimmed = content.trim().to_string();

        if trimmed.is_empty() {
            dialog_status_for_save.set_text("请输入笔记内容");
            dialog_status_for_save.set_visible(true);
            return;
        }

        let tags = parse_tags(&tags_entry_for_save.text());
        let result = match &mode {
            NoteEditorMode::Create => core_for_save.create_note(trimmed, tags),
            NoteEditorMode::Edit { note_id } => core_for_save.edit_note(note_id, trimmed, tags),
        };

        match result {
            Ok(note) => {
                {
                    let mut app_state = state_for_save.borrow_mut();
                    app_state.content_view = target_content_view;
                    app_state.search_query.clear();
                }

                if is_create && !widgets_for_save.search_entry.text().is_empty() {
                    widgets_for_save.search_entry.set_text("");
                }

                refresh_home(
                    &state_for_save,
                    &widgets_for_save,
                    &core_for_save,
                    Some(note.id),
                    Some(success_status.clone()),
                );
                dialog_for_save.close();
            }
            Err(error) => {
                dialog_status_for_save.set_text(&format!("保存失败: {error}"));
                dialog_status_for_save.set_visible(true);
            }
        }
    });

    dialog.present();
    content_view.grab_focus();
}

pub fn parse_tags(raw: &str) -> Vec<String> {
    let mut tags = Vec::new();

    for tag in raw.split([',', '，']) {
        let trimmed = tag.trim();
        if trimmed.is_empty() {
            continue;
        }
        if tags.iter().any(|existing| existing == trimmed) {
            continue;
        }
        tags.push(trimmed.to_string());
    }

    tags
}

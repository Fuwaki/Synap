use gtk::prelude::*;

use super::utils::{apply_layout_class, clear_list_box, set_button_active};
use super::widgets::{build_note_row, AppWidgets};
use crate::domain::AppState;

pub fn render_from_state(state: &std::cell::RefCell<AppState>, widgets: &AppWidgets) {
    let snapshot = state.borrow().clone();
    render(&snapshot, widgets);
}

pub fn render(state: &AppState, widgets: &AppWidgets) {
    widgets.page_title.set_text(state.content_view.title());
    widgets
        .back_to_list_button
        .set_visible(state.content_view != crate::domain::ContentView::Notes);
    widgets
        .toolbar_controls
        .set_visible(state.content_view != crate::domain::ContentView::Settings);

    if widgets.layout_dropdown.selected() != state.layout.index() {
        widgets.layout_dropdown.set_selected(state.layout.index());
    }

    set_button_active(
        &widgets.trash_button,
        state.content_view == crate::domain::ContentView::Trash,
    );
    set_button_active(
        &widgets.settings_button,
        state.content_view == crate::domain::ContentView::Settings,
    );

    if let Some(status) = &state.status {
        widgets.status_label.set_text(status);
        widgets.status_label.set_visible(true);
    } else {
        widgets.status_label.set_text("");
        widgets.status_label.set_visible(false);
    }

    match state.content_view {
        crate::domain::ContentView::NoteDetail => {
            widgets.content_stack.set_visible_child_name("detail");
            render_note_detail(state, widgets);
        }
        crate::domain::ContentView::Settings => {
            widgets.content_stack.set_visible_child_name("settings");
        }
        crate::domain::ContentView::Notes | crate::domain::ContentView::Trash => {
            widgets.content_stack.set_visible_child_name("notes");
            rebuild_note_list(state, widgets);
        }
    }
}

fn render_note_detail(state: &AppState, widgets: &AppWidgets) {
    let Some(detail) = &state.selected_note_detail else {
        widgets.detail_meta_label.set_text("未选择笔记");
        widgets
            .detail_content_label
            .set_text("请先从列表中打开一条笔记。 ");
        widgets.detail_tags_label.set_text("");
        widgets.detail_edit_button.set_sensitive(false);
        widgets.detail_delete_button.set_sensitive(false);
        return;
    };

    widgets.detail_meta_label.set_text(&format!(
        "创建于 {}{}",
        detail.created_at_label,
        if detail.deleted { " · 已删除" } else { "" }
    ));
    widgets.detail_content_label.set_text(&detail.content);

    if detail.tags.is_empty() {
        widgets.detail_tags_label.set_text("暂无标签");
    } else {
        widgets.detail_tags_label.set_text(
            &detail
                .tags
                .iter()
                .map(|tag| format!("#{tag}"))
                .collect::<Vec<_>>()
                .join("  "),
        );
    }

    widgets.detail_edit_button.set_sensitive(!detail.deleted);
    widgets.detail_delete_button.set_sensitive(!detail.deleted);
}

fn rebuild_note_list(state: &AppState, widgets: &AppWidgets) {
    clear_list_box(&widgets.list_box);
    apply_layout_class(&widgets.list_box, state.layout);

    let visible_notes = state.visible_notes();

    if visible_notes.is_empty() {
        let (title, body) = empty_copy(state);
        widgets.empty_title_label.set_text(&title);
        widgets.empty_body_label.set_text(&body);
        widgets.list_stack.set_visible_child_name("empty");
        widgets.list_box.unselect_all();
        return;
    }

    for note in &visible_notes {
        widgets.list_box.append(&build_note_row(note));
    }

    widgets.list_stack.set_visible_child_name("list");

    if let Some(selected_index) = state.selected_index_in(&visible_notes) {
        if let Some(row) = widgets.list_box.row_at_index(selected_index as i32) {
            widgets.list_box.select_row(Some(&row));
            return;
        }
    }

    widgets.list_box.unselect_all();
}

fn empty_copy(state: &AppState) -> (String, String) {
    let query = state.search_query.trim();

    match state.content_view {
        crate::domain::ContentView::Notes if query.is_empty() => (
            "还没有笔记".to_string(),
            "从左侧点击新建笔记，开始记录你的第一条内容。".to_string(),
        ),
        crate::domain::ContentView::Notes => (
            "没有找到匹配笔记".to_string(),
            format!("未检索到与\"{}\"相关的笔记，换个关键词再试试。", query),
        ),
        crate::domain::ContentView::Trash if query.is_empty() => (
            "回收站是空的".to_string(),
            "当前没有已删除笔记。".to_string(),
        ),
        crate::domain::ContentView::Trash => (
            "回收站中没有匹配项".to_string(),
            format!("回收站里没有与\"{}\"相关的内容。", query),
        ),
        crate::domain::ContentView::NoteDetail => (
            "没有可展示的笔记".to_string(),
            "请先从笔记列表中打开一条笔记。".to_string(),
        ),
        crate::domain::ContentView::Settings => (
            "设置正在整理中".to_string(),
            "桌面 Linux 端的设置页会在后续阶段补齐，目前先保留占位。".to_string(),
        ),
    }
}

use std::{cell::RefCell, rc::Rc};

use gtk::prelude::*;

use super::render::render_from_state;
use super::state::refresh_home;
use super::widgets::AppWidgets;
use crate::core::DesktopCore;
use crate::domain::{AppState, ContentView, NoteLayout};

pub fn connect_handlers(
    widgets: &AppWidgets,
    state: Rc<RefCell<AppState>>,
    core: Rc<dyn DesktopCore>,
) {
    let widgets_for_create = widgets.clone();
    let state_for_create = state.clone();
    let core_for_create = core.clone();
    widgets.create_note_button.connect_clicked(move |_| {
        super::dialog::open_create_note_window(
            &widgets_for_create.window,
            state_for_create.clone(),
            widgets_for_create.clone(),
            core_for_create.clone(),
        );
    });

    let widgets_for_notes = widgets.clone();
    let state_for_notes = state.clone();
    widgets.back_to_list_button.connect_clicked(move |_| {
        super::state::switch_content_view(&state_for_notes, &widgets_for_notes, ContentView::Notes);
    });

    let widgets_for_trash = widgets.clone();
    let state_for_trash = state.clone();
    widgets.trash_button.connect_clicked(move |_| {
        super::state::switch_content_view(&state_for_trash, &widgets_for_trash, ContentView::Trash);
    });

    let widgets_for_settings = widgets.clone();
    let state_for_settings = state.clone();
    widgets.settings_button.connect_clicked(move |_| {
        super::state::switch_content_view(
            &state_for_settings,
            &widgets_for_settings,
            ContentView::Settings,
        );
    });

    let widgets_for_search = widgets.clone();
    let state_for_search = state.clone();
    let core_for_search = core.clone();
    widgets.search_entry.connect_search_changed(move |entry| {
        {
            let mut app_state = state_for_search.borrow_mut();
            app_state.search_query = entry.text().to_string();
        }

        refresh_home(
            &state_for_search,
            &widgets_for_search,
            &core_for_search,
            None,
            None,
        );
    });

    let widgets_for_layout = widgets.clone();
    let state_for_layout = state.clone();
    widgets
        .layout_dropdown
        .connect_selected_notify(move |dropdown| {
            {
                let mut app_state = state_for_layout.borrow_mut();
                app_state.layout = NoteLayout::from_index(dropdown.selected());
            }
            render_from_state(&state_for_layout, &widgets_for_layout);
        });

    let widgets_for_selection = widgets.clone();
    let state_for_selection = state.clone();
    widgets.list_box.connect_row_selected(move |_, row| {
        let Some(row) = row else {
            return;
        };

        let index = row.index();
        if index < 0 {
            return;
        }

        let selected_note = {
            let app_state = state_for_selection.borrow();
            app_state.visible_notes().get(index as usize).cloned()
        };

        let Some(note) = selected_note else {
            return;
        };

        {
            let mut app_state = state_for_selection.borrow_mut();
            if app_state.selected_note_id.as_deref() == Some(note.id.as_str()) {
                return;
            }
            app_state.selected_note_id = Some(note.id);
        }

        render_from_state(&state_for_selection, &widgets_for_selection);
    });

    let widgets_for_open = widgets.clone();
    let state_for_open = state.clone();
    widgets.list_box.connect_row_activated(move |_, row| {
        let index = row.index();
        if index < 0 {
            return;
        }

        let note_id = {
            let app_state = state_for_open.borrow();
            app_state
                .visible_notes()
                .get(index as usize)
                .map(|note| note.id.clone())
        };

        let Some(note_id) = note_id else {
            return;
        };

        super::state::open_note_detail(&state_for_open, &widgets_for_open, note_id);
    });

    let widgets_for_edit = widgets.clone();
    let state_for_edit = state.clone();
    let core_for_edit = core.clone();
    widgets.detail_edit_button.connect_clicked(move |_| {
        let note = { state_for_edit.borrow().selected_note_detail.clone() };
        let Some(note) = note else {
            return;
        };

        super::dialog::open_edit_note_window(
            &widgets_for_edit.window,
            state_for_edit.clone(),
            widgets_for_edit.clone(),
            core_for_edit.clone(),
            note.id,
            note.content,
            note.tags,
        );
    });

    let widgets_for_delete = widgets.clone();
    let state_for_delete = state.clone();
    let core_for_delete = core.clone();
    widgets.detail_delete_button.connect_clicked(move |_| {
        let note_id = { state_for_delete.borrow().selected_note_id.clone() };
        let Some(note_id) = note_id else {
            return;
        };

        match core_for_delete.delete_note(&note_id) {
            Ok(()) => {
                {
                    let mut app_state = state_for_delete.borrow_mut();
                    app_state.content_view = ContentView::Notes;
                }
                refresh_home(
                    &state_for_delete,
                    &widgets_for_delete,
                    &core_for_delete,
                    None,
                    Some("已删除笔记".to_string()),
                );
            }
            Err(error) => {
                {
                    let mut app_state = state_for_delete.borrow_mut();
                    app_state.status = Some(format!("删除失败: {error}"));
                }
                render_from_state(&state_for_delete, &widgets_for_delete);
            }
        }
    });
}

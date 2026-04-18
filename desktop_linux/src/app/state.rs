use std::{cell::RefCell, rc::Rc};

use super::render::render_from_state;
use super::widgets::AppWidgets;
use crate::core::DesktopCore;
use crate::domain::{AppState, ContentView};
use crate::usecase::load_home;

pub fn load_initial_state(core: &dyn DesktopCore) -> AppState {
    let mut state = AppState::default();

    match load_home(core, "") {
        Ok(home) => {
            state.home = home;
            state.sync_selection();
        }
        Err(error) => {
            state.status = Some(format!("初始化失败: {error}"));
        }
    }

    state
}

pub fn switch_content_view(
    state: &Rc<RefCell<AppState>>,
    widgets: &AppWidgets,
    content_view: ContentView,
) {
    {
        let mut app_state = state.borrow_mut();
        app_state.content_view = content_view;
        app_state.sync_selection();
    }

    render_from_state(state, widgets);
}

pub fn refresh_home(
    state: &Rc<RefCell<AppState>>,
    widgets: &AppWidgets,
    core: &Rc<dyn DesktopCore>,
    selected_note_id: Option<String>,
    success_status: Option<String>,
) {
    let query = state.borrow().search_query.clone();

    match load_home(core.as_ref(), &query) {
        Ok(home) => {
            {
                let mut app_state = state.borrow_mut();
                app_state.home = home;
                if let Some(selected_note_id) = selected_note_id {
                    app_state.selected_note_id = Some(selected_note_id);
                }
                app_state.status = success_status;
                app_state.sync_selection();
            }

            render_from_state(state, widgets);
        }
        Err(error) => {
            {
                let mut app_state = state.borrow_mut();
                app_state.status = Some(format!("加载失败: {error}"));
            }

            render_from_state(state, widgets);
        }
    }
}

pub mod message;

use std::rc::Rc;

use adw::prelude::*;
use relm4::prelude::*;

use crate::{
    app::message::AppMsg,
    core::DesktopCore,
    domain::{AppState, ContentView, NoteLayout},
    ui::{
        editor::present_note_editor,
        note_widgets::{build_clickable_note_row, build_note_row, build_waterfall_card},
        shell::{build_content_pages, install_css},
        theme::apply_theme,
    },
    usecase::load_home,
};

pub struct App {
    core: Rc<dyn DesktopCore>,
    state: AppState,
    toast_overlay: adw::ToastOverlay,
    list_box: gtk::ListBox,
    content_stack: gtk::Stack,
    empty_page: adw::StatusPage,
    detail_content_row: adw::ActionRow,
    detail_tags_row: adw::ActionRow,
    detail_meta_row: adw::ActionRow,
    detail_origins_box: gtk::Box,
    detail_replies_box: gtk::Box,
    detail_versions_box: gtk::Box,
    theme_dropdown: gtk::DropDown,
    tags_flow_box: gtk::FlowBox,
    timeline_container: gtk::Box,
    layout_stack: gtk::Stack,
    flow_box: gtk::FlowBox,
}

#[relm4::component(pub)]
impl SimpleComponent for App {
    type Init = Rc<dyn DesktopCore>;
    type Input = AppMsg;
    type Output = ();

    view! {
        #[root]
        adw::ApplicationWindow {
            set_title: Some("Synap"),
            set_default_size: (816, 552),

            #[local_ref]
            toast_overlay -> adw::ToastOverlay {
                adw::OverlaySplitView {
                    set_sidebar_width_fraction: 0.22,
                    set_min_sidebar_width: 220.0,
                    set_max_sidebar_width: 320.0,

                    #[wrap(Some)]
                    set_sidebar = &gtk::Box {
                        set_orientation: gtk::Orientation::Vertical,

                        adw::HeaderBar {
                            set_show_end_title_buttons: false,

                            #[wrap(Some)]
                            set_title_widget = &gtk::Label {
                                set_label: "Synap",
                                add_css_class: "title-1"
                            }
                        },

                        gtk::Box {
                            set_orientation: gtk::Orientation::Vertical,
                            set_spacing: 6,
                            set_margin_top: 12,
                            set_margin_start: 12,
                            set_margin_end: 12,
                            set_margin_bottom: 12,

                            gtk::Button {
                                set_label: "新建笔记",
                                add_css_class: "pill",
                                add_css_class: "suggested-action",
                                connect_clicked[sender] => move |_| {
                                    sender.input(AppMsg::CreateNote);
                                }
                            },

                            gtk::Button {
                                set_label: "笔记列表",
                                #[watch]
                                set_css_classes: if model.state.content_view == ContentView::Notes {
                                    &["flat", "active"]
                                } else {
                                    &["flat"]
                                },
                                connect_clicked[sender] => move |_| {
                                    sender.input(AppMsg::Navigate(ContentView::Notes));
                                }
                            },

                            gtk::Button {
                                set_label: "回收站",
                                #[watch]
                                set_css_classes: if model.state.content_view == ContentView::Trash {
                                    &["flat", "active"]
                                } else {
                                    &["flat"]
                                },
                                connect_clicked[sender] => move |_| {
                                    sender.input(AppMsg::Navigate(ContentView::Trash));
                                }
                            },

                            gtk::Button {
                                set_label: "标签",
                                #[watch]
                                set_css_classes: if matches!(model.state.content_view, ContentView::Tags | ContentView::TagNotes) {
                                    &["flat", "active"]
                                } else {
                                    &["flat"]
                                },
                                connect_clicked[sender] => move |_| {
                                    sender.input(AppMsg::Navigate(ContentView::Tags));
                                }
                            },

                            gtk::Button {
                                set_label: "时间线",
                                #[watch]
                                set_css_classes: if model.state.content_view == ContentView::Timeline {
                                    &["flat", "active"]
                                } else {
                                    &["flat"]
                                },
                                connect_clicked[sender] => move |_| {
                                    sender.input(AppMsg::Navigate(ContentView::Timeline));
                                }
                            },

                            gtk::Box {
                                set_vexpand: true
                            },

                            gtk::Button {
                                set_label: "设置",
                                #[watch]
                                set_css_classes: if model.state.content_view == ContentView::Settings {
                                    &["flat", "active"]
                                } else {
                                    &["flat"]
                                },
                                connect_clicked[sender] => move |_| {
                                    sender.input(AppMsg::Navigate(ContentView::Settings));
                                }
                            }
                        }
                    },

                    #[wrap(Some)]
                    set_content = &gtk::Box {
                        set_orientation: gtk::Orientation::Vertical,

                        adw::HeaderBar {
                            #[wrap(Some)]
                            set_title_widget = &adw::WindowTitle {
                                set_title: "Synap",
                                #[watch]
                                set_subtitle: model.state.content_view.title()
                            }
                        },

                        gtk::Box {
                            set_orientation: gtk::Orientation::Horizontal,
                            set_spacing: 12,
                            set_margin_top: 12,
                            set_margin_bottom: 6,
                            set_margin_start: 18,
                            set_margin_end: 18,
                            #[watch]
                            set_visible: model.state.content_view != ContentView::Settings,

                            gtk::SearchEntry {
                                set_placeholder_text: Some("搜索内容或标签"),
                                set_hexpand: true,
                                set_max_width_chars: 40,
                                connect_search_changed[sender] => move |entry| {
                                    sender.input(AppMsg::SearchChanged(entry.text().to_string()));
                                }
                            },

                            gtk::Button {
                                set_label: "清除筛选",
                                add_css_class: "suggested-action",
                                #[watch]
                                set_visible: model.state.content_view == ContentView::TagNotes || !model.state.search_query.is_empty(),
                                connect_clicked[sender] => move |_| {
                                    sender.input(AppMsg::ClearFilters);
                                }
                            },

                            gtk::DropDown {
                                set_model: Some(&gtk::StringList::new(&[
                                    NoteLayout::Waterfall.label(),
                                    NoteLayout::List.label()
                                ])),
                                #[watch]
                                set_selected: model.state.layout.index(),
                                connect_selected_notify[sender] => move |dropdown| {
                                    sender.input(AppMsg::LayoutChanged(NoteLayout::from_index(dropdown.selected())));
                                }
                            }
                        },

                        gtk::Label {
                            #[watch]
                            set_visible: model.state.status.is_some(),
                            #[watch]
                            set_text: model.state.status.as_deref().unwrap_or(""),
                            add_css_class: "error",
                            set_margin_start: 18,
                            set_margin_end: 18,
                        },

                        #[local_ref]
                        content_stack -> gtk::Stack {}
                    }
                }
            }
        }
    }

    fn init(
        init: Self::Init,
        _root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let core = init;
        let mut state = AppState::default();

        match load_home(core.as_ref(), "") {
            Ok(home) => {
                state.home = home;
                state.sync_selection();
            }
            Err(error) => {
                state.status = Some(format!("初始化失败: {error}"));
            }
        }

        apply_theme(state.theme);
        let toast_overlay = adw::ToastOverlay::new();
        let pages = build_content_pages(&state, &sender);
        let content_stack = pages.content_stack.clone();

        let model = App {
            core: core.clone(),
            state,
            toast_overlay: toast_overlay.clone(),
            list_box: pages.list_box.clone(),
            content_stack: pages.content_stack,
            empty_page: pages.empty_page,
            detail_content_row: pages.detail_content_row,
            detail_tags_row: pages.detail_tags_row,
            detail_meta_row: pages.detail_meta_row,
            detail_origins_box: pages.detail_origins_box,
            detail_replies_box: pages.detail_replies_box,
            detail_versions_box: pages.detail_versions_box,
            theme_dropdown: pages.theme_dropdown,
            tags_flow_box: pages.tags_flow_box,
            timeline_container: pages.timeline_container,
            layout_stack: pages.layout_stack,
            flow_box: pages.flow_box,
        };

        let widgets = view_output!();
        install_css();
        model.connect_note_list(&sender);
        model.rebuild_list(&sender);
        model.sync_ui(&sender);

        ComponentParts { model, widgets }
    }

    fn update(&mut self, msg: Self::Input, sender: ComponentSender<Self>) {
        match msg {
            AppMsg::Navigate(view) => self.navigate(view, &sender),
            AppMsg::SearchChanged(query) => {
                self.state.search_query = query;
                self.refresh_home(&sender);
            }
            AppMsg::LayoutChanged(layout) => {
                self.state.layout = layout;
                self.rebuild_list(&sender);
            }
            AppMsg::DeleteNote => self.delete_selected_note(&sender),
            AppMsg::SaveNote { id, content, tags } => self.save_note(id, content, tags, &sender),
            AppMsg::SaveReply {
                parent_id,
                content,
                tags,
            } => self.save_reply(parent_id, content, tags, &sender),
            AppMsg::CreateNote => self.open_editor(&sender, None, false),
            AppMsg::EditNote => {
                self.open_editor(&sender, self.state.selected_note_id.clone(), false)
            }
            AppMsg::ReplyToNote => {
                self.open_editor(&sender, self.state.selected_note_id.clone(), true)
            }
            AppMsg::ThemeChanged(theme) => {
                self.state.theme = theme;
                apply_theme(theme);
            }
            AppMsg::NoteSelected(index) => self.select_note(index),
            AppMsg::NoteActivated(index) => self.activate_note(index, &sender),
            AppMsg::NoteDetailLoaded(result) => match result {
                Ok(data) => {
                    self.state.selected_note_full = Some(data);
                    self.state.status = None;
                }
                Err(error) => {
                    self.state.status = Some(format!("加载详情失败: {error}"));
                }
            },
            AppMsg::OpenNoteDetail(note_id) => self.open_note_detail(note_id, &sender),
            AppMsg::LoadMoreNotes => self.load_more_notes(&sender),
            AppMsg::MoreNotesLoaded(result) => self.finish_loading_more(result, &sender),
            AppMsg::TagSelected(tag) => self.open_tag_notes(tag, &sender),
            AppMsg::TagsLoaded(result) => match result {
                Ok(tags) => {
                    self.state.all_tags = tags;
                    self.state.status = None;
                }
                Err(error) => self.state.status = Some(format!("加载标签失败: {error}")),
            },
            AppMsg::TagNotesLoaded(result) => match result {
                Ok(notes) => {
                    self.state.tag_notes = notes;
                    self.state.sync_selection();
                    self.rebuild_list(&sender);
                    self.state.status = None;
                }
                Err(error) => self.state.status = Some(format!("加载标签笔记失败: {error}")),
            },
            AppMsg::TagSuggestionsLoaded(result) => {
                if let Ok(suggestions) = result {
                    self.state.tag_suggestions = suggestions;
                }
            }
            AppMsg::TimelineLoaded(result) => match result {
                Ok(sessions) => {
                    self.state.timeline_sessions = sessions;
                    self.state.status = None;
                }
                Err(error) => self.state.status = Some(format!("加载时间线失败: {error}")),
            },
            AppMsg::ClearFilters => self.clear_filters(&sender),
        }
        self.sync_ui(&sender);
    }
}

impl App {
    fn connect_note_list(&self, sender: &ComponentSender<Self>) {
        let sender_for_select = sender.input_sender().clone();
        self.list_box.connect_row_selected(move |_, row| {
            if let Some(row) = row {
                let _ = sender_for_select.send(AppMsg::NoteSelected(row.index() as u32));
            }
        });

        let sender_for_activate = sender.input_sender().clone();
        self.list_box.connect_row_activated(move |_, row| {
            let _ = sender_for_activate.send(AppMsg::NoteActivated(row.index() as u32));
        });
    }

    fn navigate(&mut self, view: ContentView, sender: &ComponentSender<Self>) {
        if self.state.content_view == view {
            return;
        }

        self.state.content_view = view;
        self.state.sync_selection();
        self.rebuild_list(sender);

        if view == ContentView::Tags {
            let core = self.core.clone();
            let sender = sender.clone();
            gtk::glib::spawn_future_local(async move {
                let result = core.get_all_tags();
                let _ = sender.input_sender().send(AppMsg::TagsLoaded(result));
            });
        }

        if view == ContentView::Timeline {
            let core = self.core.clone();
            let sender = sender.clone();
            gtk::glib::spawn_future_local(async move {
                let result = core.get_recent_sessions(None, Some(20));
                let _ = sender
                    .input_sender()
                    .send(AppMsg::TimelineLoaded(result.map(|page| page.sessions)));
            });
        }
    }

    fn delete_selected_note(&mut self, sender: &ComponentSender<Self>) {
        let Some(id) = self.state.selected_note_id.clone() else {
            return;
        };

        match self.core.delete_note(&id) {
            Ok(()) => {
                self.state.content_view = ContentView::Notes;
                self.refresh_home(sender);
                self.toast_overlay.add_toast(adw::Toast::new("已删除笔记"));
            }
            Err(error) => {
                self.state.status = Some(format!("删除失败: {error}"));
            }
        }
    }

    fn save_note(
        &mut self,
        id: Option<String>,
        content: String,
        tags: Vec<String>,
        sender: &ComponentSender<Self>,
    ) {
        let is_edit = id.is_some();
        let result = match id {
            Some(note_id) => self.core.edit_note(&note_id, content, tags),
            None => self.core.create_note(content, tags),
        };

        match result {
            Ok(note) => {
                self.state.content_view = if is_edit {
                    ContentView::NoteDetail
                } else {
                    ContentView::Notes
                };
                self.state.search_query.clear();
                self.refresh_home_with_selection(
                    sender,
                    note.id,
                    if is_edit {
                        "已更新笔记"
                    } else {
                        "已创建笔记"
                    },
                );
            }
            Err(error) => self.state.status = Some(format!("保存失败: {error}")),
        }
    }

    fn save_reply(
        &mut self,
        parent_id: String,
        content: String,
        tags: Vec<String>,
        sender: &ComponentSender<Self>,
    ) {
        match self.core.reply_note(&parent_id, content, tags) {
            Ok(_) => {
                self.state.content_view = ContentView::NoteDetail;
                self.state.search_query.clear();
                self.state.selected_note_id = Some(parent_id.clone());
                self.state.sync_selection();
                self.refresh_home(sender);
                self.load_note_detail(parent_id, sender);
                self.toast_overlay.add_toast(adw::Toast::new("已发送回复"));
            }
            Err(error) => self.state.status = Some(format!("回复失败: {error}")),
        }
    }

    fn select_note(&mut self, index: u32) {
        let visible = self.state.visible_notes();
        if let Some(note) = visible.get(index as usize) {
            if self.state.selected_note_id.as_deref() != Some(&note.id) {
                self.state.selected_note_id = Some(note.id.clone());
                self.state.sync_selection();
            }
        }
    }

    fn activate_note(&mut self, index: u32, sender: &ComponentSender<Self>) {
        let visible = self.state.visible_notes();
        if let Some(note) = visible.get(index as usize) {
            self.open_note_detail(note.id.clone(), sender);
        }
    }

    fn open_note_detail(&mut self, note_id: String, sender: &ComponentSender<Self>) {
        self.state.selected_note_id = Some(note_id.clone());
        self.state.content_view = ContentView::NoteDetail;
        self.state.sync_selection();
        self.rebuild_list(sender);
        self.load_note_detail(note_id, sender);
    }

    fn load_note_detail(&self, note_id: String, sender: &ComponentSender<Self>) {
        let core = self.core.clone();
        let sender = sender.clone();
        gtk::glib::spawn_future_local(async move {
            let result = crate::usecase::load_note_detail(core.as_ref(), &note_id);
            let _ = sender.input_sender().send(AppMsg::NoteDetailLoaded(result));
        });
    }

    fn load_more_notes(&mut self, sender: &ComponentSender<Self>) {
        let cursor = match self.state.content_view {
            ContentView::Notes => self.state.home.notes_cursor.clone(),
            ContentView::Trash => self.state.home.deleted_notes_cursor.clone(),
            _ => None,
        };

        if let Some(cursor) = cursor {
            self.state.is_loading_more = true;
            self.sync_ui(sender);

            let core = self.core.clone();
            let sender = sender.clone();
            let is_trash = self.state.content_view == ContentView::Trash;
            gtk::glib::spawn_future_local(async move {
                let result = if is_trash {
                    crate::usecase::load_more_deleted_notes(core.as_ref(), &cursor)
                } else {
                    crate::usecase::load_more_notes(core.as_ref(), &cursor)
                };
                let _ = sender.input_sender().send(AppMsg::MoreNotesLoaded(result));
            });
        }
    }

    fn finish_loading_more(
        &mut self,
        result: Result<
            (Vec<synap_core::dto::NoteDTO>, Option<String>, bool),
            synap_core::error::ServiceError,
        >,
        sender: &ComponentSender<Self>,
    ) {
        self.state.is_loading_more = false;
        match result {
            Ok((notes, next_cursor, has_more)) => {
                match self.state.content_view {
                    ContentView::Notes => {
                        self.state.home.notes.extend(notes);
                        self.state.home.notes_cursor = next_cursor;
                        self.state.home.has_more_notes = has_more;
                    }
                    ContentView::Trash => {
                        self.state.home.deleted_notes.extend(notes);
                        self.state.home.deleted_notes_cursor = next_cursor;
                        self.state.home.has_more_deleted_notes = has_more;
                    }
                    _ => {}
                }
                self.rebuild_list(sender);
            }
            Err(error) => self.state.status = Some(format!("加载更多失败: {error}")),
        }
    }

    fn open_tag_notes(&mut self, tag: String, sender: &ComponentSender<Self>) {
        self.state.selected_tag = Some(tag.clone());
        self.state.content_view = ContentView::TagNotes;
        self.state.sync_selection();
        self.rebuild_list(sender);

        let core = self.core.clone();
        let sender = sender.clone();
        gtk::glib::spawn_future_local(async move {
            let result = core.get_notes_by_tag(&tag, 50);
            let _ = sender.input_sender().send(AppMsg::TagNotesLoaded(result));
        });
    }

    fn clear_filters(&mut self, sender: &ComponentSender<Self>) {
        self.state.selected_tag = None;
        self.state.search_query.clear();
        self.state.content_view = ContentView::Notes;
        self.state.tag_notes.clear();
        self.state.sync_selection();
        self.refresh_home(sender);
    }

    fn rebuild_list(&self, sender: &ComponentSender<Self>) {
        while let Some(child) = self.list_box.first_child() {
            self.list_box.remove(&child);
        }

        while let Some(child) = self.flow_box.first_child() {
            self.flow_box.remove(&child);
        }

        let visible = self.state.visible_notes();
        for note in &visible {
            self.list_box.append(&build_note_row(note));
            self.flow_box.append(&build_waterfall_card(note, sender));
        }

        if self.state.is_loading_more {
            let loading_row = gtk::ListBoxRow::new();
            let spinner = gtk::Spinner::new();
            spinner.set_margin_top(12);
            spinner.set_margin_bottom(12);
            spinner.start();
            loading_row.set_child(Some(&spinner));
            loading_row.set_activatable(false);
            self.list_box.append(&loading_row);
        }

        if let Some(index) = self.state.selected_index_in(&visible) {
            if let Some(row) = self.list_box.row_at_index(index as i32) {
                self.list_box.select_row(Some(&row));
            }
        }
    }

    fn sync_ui(&self, sender: &ComponentSender<Self>) {
        self.sync_content_stack();
        self.sync_empty_page();
        self.sync_detail_rows();
        self.sync_detail_sections(sender);
        self.sync_theme_dropdown();
        self.sync_tags(sender);
        self.sync_timeline(sender);
    }

    fn sync_content_stack(&self) {
        let is_empty = self.state.visible_notes().is_empty();
        let child_name = match self.state.content_view {
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
        };
        self.content_stack.set_visible_child_name(child_name);

        if matches!(
            self.state.content_view,
            ContentView::Notes | ContentView::Trash | ContentView::TagNotes
        ) {
            let layout_name = match self.state.layout {
                NoteLayout::List => "list",
                NoteLayout::Waterfall => "waterfall",
            };
            self.layout_stack.set_visible_child_name(layout_name);
        }
    }

    fn sync_empty_page(&self) {
        let (title, desc) = match self.state.content_view {
            ContentView::Notes if self.state.search_query.is_empty() => (
                "还没有笔记",
                "从左侧点击新建笔记，开始记录你的第一条内容。".to_string(),
            ),
            ContentView::Notes => (
                "没有找到匹配笔记",
                format!(
                    "未检索到与\"{}\"相关的笔记，换个关键词再试试。",
                    self.state.search_query
                ),
            ),
            ContentView::Trash if self.state.search_query.is_empty() => {
                ("回收站是空的", "当前没有已删除笔记。".to_string())
            }
            ContentView::Trash => (
                "回收站中没有匹配项",
                format!("回收站里没有与\"{}\"相关的内容。", self.state.search_query),
            ),
            _ => ("", String::new()),
        };
        self.empty_page.set_title(title);
        self.empty_page.set_description(Some(&desc));
    }

    fn sync_detail_rows(&self) {
        self.detail_content_row.set_subtitle(&self.detail_content());
        self.detail_tags_row.set_subtitle(&self.detail_tags());
        self.detail_meta_row.set_subtitle(&self.detail_meta());
    }

    fn sync_detail_sections(&self, sender: &ComponentSender<Self>) {
        self.sync_note_section(
            &self.detail_origins_box,
            "无溯源",
            |full| &full.origins,
            sender,
        );
        self.sync_note_section(
            &self.detail_replies_box,
            "无回复",
            |full| &full.replies,
            sender,
        );

        while self.detail_versions_box.observe_children().n_items() > 1 {
            if let Some(child) = self.detail_versions_box.last_child() {
                self.detail_versions_box.remove(&child);
            }
        }
        if let Some(full) = &self.state.selected_note_full {
            if full.other_versions.is_empty() {
                self.detail_versions_box
                    .append(&adw::ActionRow::builder().title("无其他版本").build());
            } else {
                for version in &full.other_versions {
                    let note = &version.note;
                    self.detail_versions_box.append(&build_clickable_note_row(
                        note,
                        sender,
                        note.id.clone(),
                    ));
                }
            }
        }
    }

    fn sync_note_section<'a>(
        &'a self,
        container: &gtk::Box,
        empty_title: &str,
        notes: impl Fn(&'a crate::domain::NoteDetailData) -> &'a [synap_core::dto::NoteDTO],
        sender: &ComponentSender<Self>,
    ) {
        while container.observe_children().n_items() > 1 {
            if let Some(child) = container.last_child() {
                container.remove(&child);
            }
        }

        if let Some(full) = &self.state.selected_note_full {
            let notes = notes(full);
            if notes.is_empty() {
                container.append(&adw::ActionRow::builder().title(empty_title).build());
            } else {
                for note in notes {
                    container.append(&build_clickable_note_row(note, sender, note.id.clone()));
                }
            }
        }
    }

    fn sync_theme_dropdown(&self) {
        let theme_idx = self.state.theme.index();
        if self.theme_dropdown.selected() != theme_idx {
            self.theme_dropdown.set_selected(theme_idx);
        }
    }

    fn sync_tags(&self, sender: &ComponentSender<Self>) {
        while let Some(child) = self.tags_flow_box.first_child() {
            self.tags_flow_box.remove(&child);
        }
        for tag in &self.state.all_tags {
            let button = gtk::Button::with_label(&format!("#{tag}"));
            button.add_css_class("pill");
            let tag_clone = tag.clone();
            let sender_clone = sender.input_sender().clone();
            button.connect_clicked(move |_| {
                let _ = sender_clone.send(AppMsg::TagSelected(tag_clone.clone()));
            });
            self.tags_flow_box.append(&button);
        }
    }

    fn sync_timeline(&self, sender: &ComponentSender<Self>) {
        while let Some(child) = self.timeline_container.first_child() {
            self.timeline_container.remove(&child);
        }
        for session in &self.state.timeline_sessions {
            let session_box = gtk::Box::new(gtk::Orientation::Vertical, 12);

            let start_time = crate::domain::format_timestamp(session.started_at);
            let end_time = crate::domain::format_timestamp(session.ended_at);
            let header = gtk::Label::new(Some(&format!(
                "{} - {} · {} 条笔记",
                start_time, end_time, session.note_count
            )));
            header.add_css_class("heading");
            header.set_halign(gtk::Align::Start);
            session_box.append(&header);

            let notes_box = gtk::Box::new(gtk::Orientation::Vertical, 6);
            for note in &session.notes {
                notes_box.append(&build_clickable_note_row(note, sender, note.id.clone()));
            }
            session_box.append(&notes_box);

            let separator = gtk::Separator::new(gtk::Orientation::Horizontal);
            separator.set_margin_top(12);
            separator.set_margin_bottom(12);
            session_box.append(&separator);

            self.timeline_container.append(&session_box);
        }
    }

    fn refresh_home(&mut self, sender: &ComponentSender<Self>) {
        let query = self.state.search_query.clone();
        match load_home(self.core.as_ref(), &query) {
            Ok(home) => {
                self.state.home = home;
                self.state.sync_selection();
                self.state.status = None;
            }
            Err(error) => self.state.status = Some(format!("加载失败: {error}")),
        }
        self.rebuild_list(sender);
    }

    fn refresh_home_with_selection(
        &mut self,
        sender: &ComponentSender<Self>,
        note_id: String,
        toast_msg: &str,
    ) {
        let query = self.state.search_query.clone();
        match load_home(self.core.as_ref(), &query) {
            Ok(home) => {
                self.state.home = home;
                self.state.selected_note_id = Some(note_id);
                self.state.sync_selection();
                self.state.status = None;
            }
            Err(error) => self.state.status = Some(format!("加载失败: {error}")),
        }
        self.rebuild_list(sender);
        self.toast_overlay.add_toast(adw::Toast::new(toast_msg));
    }

    fn open_editor(&self, sender: &ComponentSender<Self>, note_id: Option<String>, is_reply: bool) {
        present_note_editor(
            sender,
            note_id,
            is_reply,
            self.state.selected_note_detail.clone(),
        );
    }

    fn detail_content(&self) -> String {
        self.state
            .selected_note_detail
            .as_ref()
            .map(|d| d.content.clone())
            .unwrap_or_else(|| "请先从列表中打开一条笔记。".to_string())
    }

    fn detail_tags(&self) -> String {
        self.state
            .selected_note_detail
            .as_ref()
            .map(|d| {
                if d.tags.is_empty() {
                    "暂无标签".to_string()
                } else {
                    d.tags
                        .iter()
                        .map(|t| format!("#{t}"))
                        .collect::<Vec<_>>()
                        .join("  ")
                }
            })
            .unwrap_or_default()
    }

    fn detail_meta(&self) -> String {
        self.state
            .selected_note_detail
            .as_ref()
            .map(|d| {
                format!(
                    "创建于 {}{}",
                    d.created_at_label,
                    if d.deleted { " · 已删除" } else { "" }
                )
            })
            .unwrap_or_default()
    }
}

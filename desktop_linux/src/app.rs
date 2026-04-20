use std::rc::Rc;

use adw::prelude::*;
use relm4::prelude::*;

use crate::core::DesktopCore;
use crate::domain::{AppState, ContentView, NoteLayout, Theme};
use crate::usecase::load_home;

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
    theme_dropdown: gtk::DropDown,
}

#[derive(Debug)]
pub enum AppMsg {
    Navigate(ContentView),
    SearchChanged(String),
    LayoutChanged(NoteLayout),
    DeleteNote,
    SaveNote {
        id: Option<String>,
        content: String,
        tags: Vec<String>,
    },
    CreateNote,
    EditNote,
    ThemeChanged(Theme),
    NoteSelected(u32),
    NoteActivated(u32),
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
                                    connect_search_changed[sender] => move |entry| {
                                        sender.input(AppMsg::SearchChanged(entry.text().to_string()));
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
                            content_stack -> gtk::Stack {
                                set_hexpand: true,
                                set_vexpand: true,
                                set_margin_start: 12,
                                set_margin_end: 12,
                                set_margin_bottom: 12,
                            }
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

        let toast_overlay = adw::ToastOverlay::new();
        let list_box = gtk::ListBox::new();
        list_box.set_css_classes(&["boxed-list"]);
        list_box.set_selection_mode(gtk::SelectionMode::Single);
        list_box.set_vexpand(true);

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

        // Build Stack and its children imperatively
        let content_stack = gtk::Stack::new();
        content_stack.set_hexpand(true);
        content_stack.set_vexpand(true);
        content_stack.set_margin_start(12);
        content_stack.set_margin_end(12);
        content_stack.set_margin_bottom(12);

        // Notes page
        let notes_scroller = gtk::ScrolledWindow::new();
        notes_scroller.set_child(Some(&list_box));
        content_stack.add_named(&notes_scroller, Some("notes"));

        // Empty page
        let empty_page = adw::StatusPage::new();
        empty_page.set_icon_name(Some("document-edit-symbolic"));
        content_stack.add_named(&empty_page, Some("empty"));

        // Detail page
        let detail_clamp = adw::Clamp::builder()
            .maximum_size(800)
            .margin_top(24)
            .margin_bottom(24)
            .margin_start(24)
            .margin_end(24)
            .build();

        let detail_box = gtk::Box::new(gtk::Orientation::Vertical, 24);

        let detail_group = adw::PreferencesGroup::builder().title("笔记内容").build();

        let detail_content_row = adw::ActionRow::builder()
            .title("内容")
            .subtitle_selectable(true)
            .build();
        let detail_tags_row = adw::ActionRow::builder().title("标签").build();
        let detail_meta_row = adw::ActionRow::builder().title("创建时间").build();

        detail_group.add(&detail_content_row);
        detail_group.add(&detail_tags_row);
        detail_group.add(&detail_meta_row);

        let detail_buttons = gtk::Box::new(gtk::Orientation::Horizontal, 12);
        detail_buttons.set_halign(gtk::Align::End);

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

        detail_buttons.append(&edit_button);
        detail_buttons.append(&delete_button);

        detail_box.append(&detail_group);
        detail_box.append(&detail_buttons);
        detail_clamp.set_child(Some(&detail_box));
        content_stack.add_named(&detail_clamp, Some("detail"));

        // Settings page
        let settings_page = adw::PreferencesPage::new();
        let settings_group = adw::PreferencesGroup::builder().title("外观").build();

        let theme_row = adw::ActionRow::builder()
            .title("主题")
            .subtitle("选择应用的颜色主题")
            .build();

        let theme_dropdown = gtk::DropDown::from_strings(&["跟随系统", "浅色", "深色"]);
        theme_dropdown.set_selected(state.theme.index());
        let sender_theme = sender.input_sender().clone();
        theme_dropdown.connect_selected_notify(move |dropdown| {
            let _ = sender_theme.send(AppMsg::ThemeChanged(Theme::from_index(dropdown.selected())));
        });
        theme_row.add_suffix(&theme_dropdown);
        settings_group.add(&theme_row);
        settings_page.add(&settings_group);
        content_stack.add_named(&settings_page, Some("settings"));

        // Set initial visible child
        let is_empty = state.visible_notes().is_empty();
        let initial_child = match state.content_view {
            ContentView::NoteDetail => "detail",
            ContentView::Settings => "settings",
            _ => {
                if is_empty {
                    "empty"
                } else {
                    "notes"
                }
            }
        };
        content_stack.set_visible_child_name(initial_child);

        let model = App {
            core: core.clone(),
            state,
            toast_overlay: toast_overlay.clone(),
            list_box: list_box.clone(),
            content_stack: content_stack.clone(),
            empty_page: empty_page.clone(),
            detail_content_row: detail_content_row.clone(),
            detail_tags_row: detail_tags_row.clone(),
            detail_meta_row: detail_meta_row.clone(),
            theme_dropdown: theme_dropdown.clone(),
        };

        let widgets = view_output!();

        let provider = gtk::CssProvider::new();
        provider.load_from_string(include_str!("style.css"));
        if let Some(display) = gtk::gdk::Display::default() {
            gtk::style_context_add_provider_for_display(
                &display,
                &provider,
                gtk::STYLE_PROVIDER_PRIORITY_APPLICATION,
            );
        }

        let sender_for_select = sender.input_sender().clone();
        list_box.connect_row_selected(move |_, row| {
            if let Some(row) = row {
                let _ = sender_for_select.send(AppMsg::NoteSelected(row.index() as u32));
            }
        });

        let sender_for_activate = sender.input_sender().clone();
        list_box.connect_row_activated(move |_, row| {
            let _ = sender_for_activate.send(AppMsg::NoteActivated(row.index() as u32));
        });

        model.rebuild_list();
        model.sync_ui();

        ComponentParts { model, widgets }
    }

    fn update(&mut self, msg: Self::Input, sender: ComponentSender<Self>) {
        match msg {
            AppMsg::Navigate(view) => {
                if self.state.content_view != view {
                    self.state.content_view = view;
                    self.state.sync_selection();
                    self.rebuild_list();
                }
            }

            AppMsg::SearchChanged(query) => {
                self.state.search_query = query;
                self.refresh_home();
            }

            AppMsg::LayoutChanged(layout) => {
                self.state.layout = layout;
                self.rebuild_list();
            }

            AppMsg::DeleteNote => {
                if let Some(id) = self.state.selected_note_id.clone() {
                    match self.core.delete_note(&id) {
                        Ok(()) => {
                            self.state.content_view = ContentView::Notes;
                            self.refresh_home();
                            self.state.status = Some("已删除笔记".to_string());
                            self.toast_overlay.add_toast(adw::Toast::new("已删除笔记"));
                        }
                        Err(error) => {
                            self.state.status = Some(format!("删除失败: {error}"));
                        }
                    }
                }
            }

            AppMsg::SaveNote { id, content, tags } => {
                let is_edit = id.is_some();
                let result = match id {
                    Some(note_id) => self.core.edit_note(&note_id, content, tags),
                    None => self.core.create_note(content, tags),
                };

                match result {
                    Ok(note) => {
                        let view = if is_edit {
                            ContentView::NoteDetail
                        } else {
                            ContentView::Notes
                        };
                        self.state.content_view = view;
                        self.state.search_query.clear();
                        self.refresh_home_with_selection(
                            note.id,
                            if is_edit {
                                "已更新笔记"
                            } else {
                                "已创建笔记"
                            },
                        );
                    }
                    Err(error) => {
                        self.state.status = Some(format!("保存失败: {error}"));
                    }
                }
            }

            AppMsg::CreateNote => {
                self.open_editor(&sender, None);
            }

            AppMsg::EditNote => {
                let note_id = self.state.selected_note_id.clone();
                self.open_editor(&sender, note_id);
            }

            AppMsg::ThemeChanged(theme) => {
                self.state.theme = theme;
                apply_theme(theme);
            }

            AppMsg::NoteSelected(index) => {
                let visible = self.state.visible_notes();
                if let Some(note) = visible.get(index as usize) {
                    if self.state.selected_note_id.as_deref() != Some(&note.id) {
                        self.state.selected_note_id = Some(note.id.clone());
                        self.state.sync_selection();
                    }
                }
            }

            AppMsg::NoteActivated(index) => {
                let visible = self.state.visible_notes();
                if let Some(note) = visible.get(index as usize) {
                    self.state.selected_note_id = Some(note.id.clone());
                    self.state.content_view = ContentView::NoteDetail;
                    self.state.sync_selection();
                    self.rebuild_list();
                }
            }
        }
        self.sync_ui();
    }
}

impl App {
    fn rebuild_list(&self) {
        while let Some(child) = self.list_box.first_child() {
            self.list_box.remove(&child);
        }

        let visible = self.state.visible_notes();

        for note in &visible {
            let row = build_note_row(note);
            self.list_box.append(&row);
        }

        if let Some(index) = self.state.selected_index_in(&visible) {
            if let Some(row) = self.list_box.row_at_index(index as i32) {
                self.list_box.select_row(Some(&row));
            }
        }
    }

    fn sync_ui(&self) {
        let is_empty = self.state.visible_notes().is_empty();
        let child_name = match self.state.content_view {
            ContentView::NoteDetail => "detail",
            ContentView::Settings => "settings",
            _ => {
                if is_empty {
                    "empty"
                } else {
                    "notes"
                }
            }
        };
        self.content_stack.set_visible_child_name(child_name);

        // Update empty page
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

        // Update detail rows
        self.detail_content_row.set_subtitle(&self.detail_content());
        self.detail_tags_row.set_subtitle(&self.detail_tags());
        self.detail_meta_row.set_subtitle(&self.detail_meta());

        // Update theme dropdown without triggering signal
        let theme_idx = self.state.theme.index();
        if self.theme_dropdown.selected() != theme_idx {
            self.theme_dropdown.set_selected(theme_idx);
        }
    }

    fn refresh_home(&mut self) {
        let query = self.state.search_query.clone();
        match load_home(self.core.as_ref(), &query) {
            Ok(home) => {
                self.state.home = home;
                self.state.sync_selection();
                self.state.status = None;
            }
            Err(error) => {
                self.state.status = Some(format!("加载失败: {error}"));
            }
        }
        self.rebuild_list();
    }

    fn refresh_home_with_selection(&mut self, note_id: String, status: &str) {
        let query = self.state.search_query.clone();
        match load_home(self.core.as_ref(), &query) {
            Ok(home) => {
                self.state.home = home;
                self.state.selected_note_id = Some(note_id);
                self.state.sync_selection();
                self.state.status = Some(status.to_string());
            }
            Err(error) => {
                self.state.status = Some(format!("加载失败: {error}"));
            }
        }
        self.rebuild_list();
        self.toast_overlay.add_toast(adw::Toast::new(status));
    }

    fn open_editor(&self, sender: &ComponentSender<Self>, note_id: Option<String>) {
        let (title, _hint, initial_content, initial_tags) = if let Some(_id) = &note_id {
            let detail = self.state.selected_note_detail.clone();
            if let Some(d) = detail {
                (
                    "编辑笔记",
                    "修改正文或标签，保存后会立即更新详情。",
                    d.content,
                    d.tags,
                )
            } else {
                return;
            }
        } else {
            (
                "新建笔记",
                "直接记录内容；标签可选，使用逗号分隔。",
                String::new(),
                Vec::new(),
            )
        };

        let dialog = adw::Dialog::builder()
            .title(title)
            .content_width(720)
            .content_height(560)
            .build();

        let toolbar_view = adw::ToolbarView::new();
        let header = adw::HeaderBar::new();

        let cancel_button = gtk::Button::with_label("取消");
        let save_button = gtk::Button::with_label("保存");
        save_button.add_css_class("suggested-action");

        header.pack_start(&cancel_button);
        header.pack_end(&save_button);

        let shell = gtk::Box::new(gtk::Orientation::Vertical, 16);
        shell.set_margin_top(24);
        shell.set_margin_bottom(24);
        shell.set_margin_start(24);
        shell.set_margin_end(24);

        let content_buffer = gtk::TextBuffer::new(None);
        content_buffer.set_text(&initial_content);
        let content_view = gtk::TextView::with_buffer(&content_buffer);
        content_view.set_vexpand(true);
        content_view.set_wrap_mode(gtk::WrapMode::WordChar);
        content_view.set_top_margin(8);
        content_view.set_bottom_margin(8);
        content_view.set_left_margin(8);
        content_view.set_right_margin(8);

        let content_scroller = gtk::ScrolledWindow::new();
        content_scroller.set_vexpand(true);
        content_scroller.set_min_content_height(320);
        content_scroller.set_child(Some(&content_view));

        let tags_entry = gtk::Entry::new();
        tags_entry.set_placeholder_text(Some("标签，例如 rust, idea, diary"));
        tags_entry.set_text(&initial_tags.join(", "));

        let status_label = gtk::Label::new(None);
        status_label.add_css_class("error");
        status_label.set_visible(false);

        shell.append(&content_scroller);
        shell.append(&tags_entry);
        shell.append(&status_label);

        toolbar_view.add_top_bar(&header);
        toolbar_view.set_content(Some(&shell));

        dialog.set_child(Some(&toolbar_view));

        let dialog_for_cancel = dialog.clone();
        cancel_button.connect_clicked(move |_| {
            dialog_for_cancel.close();
        });

        let dialog_for_save = dialog.clone();
        let input_sender = sender.input_sender().clone();
        let note_id_for_save = note_id.clone();
        save_button.connect_clicked(move |_| {
            let (start, end) = content_buffer.bounds();
            let text = content_buffer.text(&start, &end, false).to_string();
            let trimmed = text.trim().to_string();

            if trimmed.is_empty() {
                status_label.set_text("请输入笔记内容");
                status_label.set_visible(true);
                return;
            }

            let tags = parse_tags(&tags_entry.text());
            let _ = input_sender.send(AppMsg::SaveNote {
                id: note_id_for_save.clone(),
                content: trimmed,
                tags,
            });

            dialog_for_save.close();
        });

        dialog.present(None::<&gtk::Widget>);
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

fn build_note_row(note: &synap_core::dto::NoteDTO) -> gtk::ListBoxRow {
    let preview = crate::domain::NoteListItemViewModel::from(note).preview;

    let action_row = adw::ActionRow::new();
    action_row.set_title(&preview);
    action_row.set_subtitle(
        &note
            .tags
            .iter()
            .map(|t| format!("#{t}"))
            .collect::<Vec<_>>()
            .join("  "),
    );
    action_row.set_activatable(true);

    let row = gtk::ListBoxRow::new();
    row.set_child(Some(&action_row));
    row
}

fn parse_tags(raw: &str) -> Vec<String> {
    let mut tags = Vec::new();

    for tag in raw.split([',', '，']) {
        let trimmed = tag.trim();
        if trimmed.is_empty() {
            continue;
        }
        if tags.iter().any(|existing: &String| existing == trimmed) {
            continue;
        }
        tags.push(trimmed.to_string());
    }

    tags
}

fn apply_theme(theme: Theme) {
    let manager = adw::StyleManager::default();
    match theme {
        Theme::Auto => manager.set_color_scheme(adw::ColorScheme::Default),
        Theme::Light => manager.set_color_scheme(adw::ColorScheme::ForceLight),
        Theme::Dark => manager.set_color_scheme(adw::ColorScheme::ForceDark),
    }
}

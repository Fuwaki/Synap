use leptos::{prelude::*, task::spawn_local};
use synap_core::NoteDTO;

use crate::app::server::{
    create_note_server, delete_note_server, edit_note_server, list_notes, recommend_tags_server,
};
use crate::app::utils::{format_timestamp, join_tags, parse_tags, preview, remove_tag, short_note_id};

#[component]
pub fn HomePage() -> impl IntoView {
    let (search_input, set_search_input) = signal(String::new());
    let (active_query, set_active_query) = signal(String::new());
    let (refresh_key, set_refresh_key) = signal(0_u64);
    let (focused_note, set_focused_note) = signal(None::<NoteDTO>);
    let (is_creating_new, set_is_creating_new) = signal(false);
    let (editor_initialized, set_editor_initialized) = signal(false);
    let (editor_content, set_editor_content) = signal(String::new());
    let (editor_tags, set_editor_tags) = signal(String::new());
    let (status, set_status) = signal(String::from("Synap Web 已连接到 synap-core"));
    let (is_saving, set_is_saving) = signal(false);
    let (is_deleting, set_is_deleting) = signal(false);

    let notes = Resource::new(
        move || (active_query.get(), refresh_key.get()),
        |(query, _)| async move { list_notes(query).await },
    );

    let active_note = Memo::new(move |_| {
        if is_creating_new.get() {
            return None;
        }

        if let Some(note) = focused_note.get() {
            return Some(note);
        }

        match notes.get() {
            Some(Ok(items)) => items.into_iter().next(),
            _ => None,
        }
    });

    let effective_content = Memo::new(move |_| {
        if editor_initialized.get() {
            editor_content.get()
        } else {
            active_note
                .get()
                .map(|note| note.content)
                .unwrap_or_default()
        }
    });

    let effective_tags_input = Memo::new(move |_| {
        if editor_initialized.get() {
            editor_tags.get()
        } else {
            active_note
                .get()
                .map(|note| join_tags(&note.tags))
                .unwrap_or_default()
        }
    });

    let effective_tags = Memo::new(move |_| parse_tags(&effective_tags_input.get()));

    let recommended_tags = Resource::new(
        move || (effective_content.get(), refresh_key.get()),
        |(content, _)| async move {
            let normalized = content.trim().to_string();
            if normalized.is_empty() {
                Ok(Vec::new())
            } else {
                recommend_tags_server(normalized).await
            }
        },
    );

    let available_recommended_tags = Memo::new(move |_| {
        let existing = effective_tags.get();
        match recommended_tags.get() {
            Some(Ok(tags)) => tags
                .into_iter()
                .filter(|tag| !existing.iter().any(|current| current == tag))
                .collect::<Vec<_>>(),
            _ => Vec::new(),
        }
    });

    let editor_title = Memo::new(move |_| {
        if is_creating_new.get() || active_note.get().is_none() {
            "新建笔记".to_string()
        } else {
            "编辑笔记".to_string()
        }
    });

    let editor_subtitle = Memo::new(move |_| {
        if is_creating_new.get() {
            "内容会通过 Leptos server fn 持久化到 synap-core。".to_string()
        } else if let Some(note) = active_note.get() {
            format!(
                "{} · {}",
                short_note_id(&note.id),
                format_timestamp(note.created_at)
            )
        } else {
            "还没有笔记，写下第一条吧。".to_string()
        }
    });

    let save_label = Memo::new(move |_| {
        if active_note.get().is_some() && !is_creating_new.get() {
            "保存为新版本".to_string()
        } else {
            "创建笔记".to_string()
        }
    });

    view! {
        <Transition
            fallback=move || view! { <div class="app-fallback">"正在从 synap-core 载入数据..."</div> }
        >
            <div class="synap-app">
                <div class="sidebar" id="sidebar">
                    <div class="sidebar-header">
                        <span>"Synap Web"</span>
                        <div class="header-actions">
                            <button
                                class="icon-btn"
                                type="button"
                                title="刷新"
                                on:click=move |_| {
                                    set_refresh_key.update(|value| *value += 1);
                                    set_status.set("已刷新笔记列表".to_string());
                                }
                            >
                                "↻"
                            </button>
                            <button
                                class="icon-btn"
                                type="button"
                                title="新建笔记"
                                on:click=move |_| {
                                    set_is_creating_new.set(true);
                                    set_focused_note.set(None);
                                    set_editor_initialized.set(true);
                                    set_editor_content.set(String::new());
                                    set_editor_tags.set(String::new());
                                    set_status.set("正在创建新笔记".to_string());
                                }
                            >
                                "+"
                            </button>
                        </div>
                    </div>

                    <div class="search-container">
                        <div class="search-box">
                            <input
                                type="text"
                                class="search-input"
                                placeholder="搜索笔记或标签..."
                                prop:value=move || search_input.get()
                                on:input=move |ev| set_search_input.set(event_target_value(&ev))
                            />
                            <button
                                class="search-btn"
                                type="button"
                                title="搜索"
                                on:click=move |_| {
                                    let normalized = search_input.get_untracked().trim().to_string();
                                    if active_query.get_untracked() == normalized {
                                        set_refresh_key.update(|value| *value += 1);
                                    } else {
                                        set_active_query.set(normalized.clone());
                                    }

                                    if normalized.is_empty() {
                                        set_status.set("已切回最近笔记流".to_string());
                                    } else {
                                        set_status.set(format!("已按“{normalized}”检索"));
                                    }
                                }
                            >
                                "⌕"
                            </button>
                        </div>
                    </div>

                    <div class="list-title">
                        {move || {
                            match notes.get() {
                                Some(Ok(items)) => {
                                    if active_query.get().is_empty() {
                                        format!("最近笔记 · {} 条", items.len())
                                    } else {
                                        format!("搜索结果 · {} 条", items.len())
                                    }
                                }
                                Some(Err(_)) => "笔记列表".to_string(),
                                None => "正在加载笔记…".to_string(),
                            }
                        }}
                    </div>

                    <div class="sidebar-scroll-area">
                        {move || {
                            match notes.get() {
                                None => {
                                    view! {
                                        <div class="list-empty">
                                            "正在从 synap-core 载入数据..."
                                        </div>
                                    }
                                        .into_any()
                                }
                                Some(Err(error)) => {
                                    view! {
                                        <div class="list-empty error-text">
                                            {format!("加载失败: {error}")}
                                        </div>
                                    }
                                        .into_any()
                                }
                                Some(Ok(items)) if items.is_empty() => {
                                    let empty_text = if active_query.get().is_empty() {
                                        "还没有笔记，点击右上角 + 开始记录。"
                                    } else {
                                        "没有匹配结果，试试别的关键词。"
                                    };

                                    view! { <div class="list-empty">{empty_text}</div> }.into_any()
                                }
                                Some(Ok(items)) => {
                                    view! {
                                        <ul class="note-list">
                                            {items
                                                .into_iter()
                                                .map(|note| {
                                                    let note_for_click = note.clone();
                                                    let note_id = note.id.clone();
                                                    let content = preview(&note.content, 96);
                                                    let tags = note.tags.clone();

                                                    view! {
                                                        <li
                                                            class=move || {
                                                                let class_name = if active_note
                                                                    .get()
                                                                    .is_some_and(|active| active.id == note_id)
                                                                {
                                                                    "note-item active"
                                                                } else {
                                                                    "note-item"
                                                                };
                                                                class_name
                                                            }
                                                            on:click=move |_| {
                                                                set_is_creating_new.set(false);
                                                                set_focused_note.set(Some(note_for_click.clone()));
                                                                set_editor_initialized.set(true);
                                                                set_editor_content
                                                                    .set(note_for_click.content.clone());
                                                                set_editor_tags
                                                                    .set(join_tags(&note_for_click.tags));
                                                                set_status
                                                                    .set(format!(
                                                                        "已选择 {}",
                                                                        short_note_id(&note_for_click.id)
                                                                    ));
                                                            }
                                                        >
                                                            <div class="note-content">{content}</div>

                                                            <div class="note-meta">
                                                                <span class="note-time">
                                                                    {format_timestamp(note.created_at)}
                                                                </span>
                                                                {tags
                                                                    .into_iter()
                                                                    .map(|tag| {
                                                                        view! {
                                                                            <span class="note-meta-tag">
                                                                                {tag}
                                                                            </span>
                                                                        }
                                                                    })
                                                                    .collect_view()}
                                                            </div>
                                                        </li>
                                                    }
                                                })
                                                .collect_view()}
                                        </ul>
                                    }
                                        .into_any()
                                }
                            }
                        }}
                    </div>
                </div>

                <div class="resizer"></div>

                <div class="main-content">
                    <div class="editor-container">
                        <div class="editor-header">
                            <h2>{move || editor_title.get()}</h2>
                            <p class="editor-subtitle">{move || editor_subtitle.get()}</p>
                        </div>

                        <div class="tag-bar">
                            {move || {
                                effective_tags
                                    .get()
                                    .into_iter()
                                    .map(|tag| {
                                        let tag_name = tag.clone();
                                        view! {
                                            <span class="tag-pill">
                                                {tag}
                                                <button
                                                    class="tag-delete"
                                                    type="button"
                                                    title="删除标签"
                                                    on:click=move |_| {
                                                        let next = remove_tag(
                                                            &effective_tags_input.get_untracked(),
                                                            &tag_name,
                                                        );
                                                        set_editor_initialized.set(true);
                                                        set_editor_tags.set(next);
                                                    }
                                                >
                                                    "×"
                                                </button>
                                            </span>
                                        }
                                    })
                                    .collect_view()
                            }}
                            <input
                                type="text"
                                class="tag-input"
                                placeholder="用逗号分隔标签，如 rust, idea, search"
                                prop:value=move || effective_tags_input.get()
                                on:input=move |ev| {
                                    set_editor_initialized.set(true);
                                    set_editor_tags.set(event_target_value(&ev));
                                }
                            />
                        </div>

                        <Transition fallback=move || view! { <></> }>
                            {move || {
                                let suggestions = available_recommended_tags.get();
                                if suggestions.is_empty() {
                                    view! { <></> }.into_any()
                                } else {
                                    view! {
                                        <div class="tag-suggestions">
                                            <span class="tag-suggestion-label">"推荐标签"</span>
                                            {suggestions
                                                .into_iter()
                                                .map(|tag| {
                                                    let tag_value = tag.clone();
                                                    view! {
                                                        <button
                                                            class="tag-suggestion"
                                                            type="button"
                                                            on:click=move |_| {
                                                                let mut tags = parse_tags(
                                                                    &effective_tags_input.get_untracked(),
                                                                );
                                                                if !tags.iter().any(|current| current == &tag_value) {
                                                                    tags.push(tag_value.clone());
                                                                }
                                                                set_editor_initialized.set(true);
                                                                set_editor_tags.set(tags.join(", "));
                                                            }
                                                        >
                                                            {tag}
                                                        </button>
                                                    }
                                                })
                                                .collect_view()}
                                        </div>
                                    }
                                        .into_any()
                                }
                            }}
                        </Transition>

                        <div class="toolbar">
                            <button
                                class="action-btn primary-btn"
                                type="button"
                                disabled=move || {
                                    is_saving.get()
                                        || is_deleting.get()
                                        || effective_content.get().trim().is_empty()
                                }
                                on:click=move |_| {
                                    let content = effective_content.get_untracked().trim().to_string();
                                    if content.is_empty() {
                                        set_status.set("请输入笔记内容".to_string());
                                        return;
                                    }

                                    let tags = effective_tags.get_untracked();
                                    let current_note = active_note.get_untracked();
                                    set_is_saving.set(true);

                                    spawn_local(async move {
                                        let result = if let Some(note) = current_note.clone() {
                                            edit_note_server(note.id.clone(), content.clone(), tags).await
                                        } else {
                                            create_note_server(content.clone(), tags).await
                                        };

                                        set_is_saving.set(false);

                                        match result {
                                            Ok(note) => {
                                                let message = if current_note.is_some() {
                                                    "已生成新版本"
                                                } else {
                                                    "已创建笔记"
                                                };
                                                set_is_creating_new.set(false);
                                                set_focused_note.set(Some(note.clone()));
                                                set_editor_initialized.set(true);
                                                set_editor_content.set(note.content.clone());
                                                set_editor_tags.set(join_tags(&note.tags));
                                                set_refresh_key.update(|value| *value += 1);
                                                set_status
                                                    .set(format!("{message} {}", short_note_id(&note.id)));
                                            }
                                            Err(error) => {
                                                set_status.set(format!("保存失败: {error}"));
                                            }
                                        }
                                    });
                                }
                            >
                                {move || {
                                    if is_saving.get() {
                                        "正在保存...".to_string()
                                    } else {
                                        save_label.get()
                                    }
                                }}
                            </button>

                            <button
                                class="action-btn"
                                type="button"
                                on:click=move |_| {
                                    set_is_creating_new.set(true);
                                    set_focused_note.set(None);
                                    set_editor_initialized.set(true);
                                    set_editor_content.set(String::new());
                                    set_editor_tags.set(String::new());
                                    set_status.set("已切换到新建模式".to_string());
                                }
                            >
                                "清空编辑器"
                            </button>

                            <button
                                class="action-btn danger-btn"
                                type="button"
                                disabled=move || {
                                    is_deleting.get()
                                        || is_saving.get()
                                        || active_note.get().is_none()
                                        || is_creating_new.get()
                                }
                                on:click=move |_| {
                                    let Some(note) = active_note.get_untracked() else {
                                        set_status.set("先选择一条笔记再删除".to_string());
                                        return;
                                    };

                                    let note_id = note.id.clone();
                                    set_is_deleting.set(true);

                                    spawn_local(async move {
                                        let result = delete_note_server(note_id.clone()).await;
                                        set_is_deleting.set(false);

                                        match result {
                                            Ok(()) => {
                                                set_focused_note.set(None);
                                                set_editor_initialized.set(false);
                                                set_editor_content.set(String::new());
                                                set_editor_tags.set(String::new());
                                                set_refresh_key.update(|value| *value += 1);
                                                set_status
                                                    .set(format!("已删除 {}", short_note_id(&note_id)));
                                            }
                                            Err(error) => {
                                                set_status.set(format!("删除失败: {error}"));
                                            }
                                        }
                                    });
                                }
                            >
                                {move || {
                                    if is_deleting.get() {
                                        "正在删除...".to_string()
                                    } else {
                                        "删除当前笔记".to_string()
                                    }
                                }}
                            </button>
                        </div>

                        <textarea
                            id="editor"
                            placeholder="点此开始记录笔记"
                            prop:value=move || effective_content.get()
                            on:input=move |ev| {
                                set_editor_initialized.set(true);
                                set_editor_content.set(event_target_value(&ev));
                            }
                        />

                        <div class="status-bar">
                            <span>{move || status.get()}</span>
                            <span class="subtitle-text">
                                {move || {
                                    if active_query.get().is_empty() {
                                        "数据源：最近笔记".to_string()
                                    } else {
                                        format!("当前检索：{}", active_query.get())
                                    }
                                }}
                            </span>
                        </div>
                    </div>
                </div>
            </div>
        </Transition>
    }
}

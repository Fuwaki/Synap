use synap_core::dto::NoteDTO;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ContentView {
    Notes,
    Trash,
    Settings,
}

impl ContentView {
    pub fn title(self) -> &'static str {
        match self {
            Self::Notes => "笔记列表",
            Self::Trash => "回收站",
            Self::Settings => "设置",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NoteLayout {
    Waterfall,
    List,
}

impl NoteLayout {
    pub fn from_index(index: u32) -> Self {
        match index {
            1 => Self::List,
            _ => Self::Waterfall,
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::Waterfall => "瀑布流",
            Self::List => "列表",
        }
    }

    pub fn index(self) -> u32 {
        match self {
            Self::Waterfall => 0,
            Self::List => 1,
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct HomeData {
    pub notes: Vec<NoteDTO>,
    pub deleted_notes: Vec<NoteDTO>,
}

#[derive(Debug, Clone)]
pub struct NoteListItemViewModel {
    pub preview: String,
}

impl From<&NoteDTO> for NoteListItemViewModel {
    fn from(value: &NoteDTO) -> Self {
        Self {
            preview: build_preview(&value.content),
        }
    }
}

#[derive(Debug, Clone)]
pub struct AppState {
    pub home: HomeData,
    pub search_query: String,
    pub content_view: ContentView,
    pub layout: NoteLayout,
    pub selected_note_id: Option<String>,
    pub status: Option<String>,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            home: HomeData::default(),
            search_query: String::new(),
            content_view: ContentView::Notes,
            layout: NoteLayout::Waterfall,
            selected_note_id: None,
            status: None,
        }
    }
}

impl AppState {
    pub fn visible_notes(&self) -> Vec<NoteDTO> {
        match self.content_view {
            ContentView::Notes => self.home.notes.clone(),
            ContentView::Trash => {
                filter_deleted_notes(&self.home.deleted_notes, &self.search_query)
            }
            ContentView::Settings => Vec::new(),
        }
    }

    pub fn sync_selection(&mut self) {
        if self.content_view == ContentView::Settings {
            return;
        }

        let visible = self.visible_notes();
        let is_selected_visible = self
            .selected_note_id
            .as_ref()
            .is_some_and(|selected_id| visible.iter().any(|note| note.id == *selected_id));

        if !is_selected_visible {
            self.selected_note_id = visible.first().map(|note| note.id.clone());
        }
    }

    pub fn selected_index_in(&self, notes: &[NoteDTO]) -> Option<usize> {
        let selected = self.selected_note_id.as_ref()?;
        notes.iter().position(|note| note.id == *selected)
    }
}

fn filter_deleted_notes(notes: &[NoteDTO], query: &str) -> Vec<NoteDTO> {
    let trimmed = query.trim();
    if trimmed.is_empty() {
        return notes.to_vec();
    }

    let needle = trimmed.to_lowercase();
    notes
        .iter()
        .filter(|note| {
            note.content.to_lowercase().contains(&needle)
                || note
                    .tags
                    .iter()
                    .any(|tag| tag.to_lowercase().contains(&needle))
        })
        .cloned()
        .collect()
}

fn build_preview(content: &str) -> String {
    let trimmed = content.trim();
    if trimmed.is_empty() {
        return "空白笔记".to_string();
    }

    let normalized = trimmed.split_whitespace().collect::<Vec<_>>().join(" ");
    const MAX_CHARS: usize = 220;

    if normalized.chars().count() <= MAX_CHARS {
        normalized
    } else {
        let preview: String = normalized.chars().take(MAX_CHARS).collect();
        format!("{preview}...")
    }
}

use leptos::prelude::*;
use server_fn::error::ServerFnError;
use synap_core::NoteDTO;

#[cfg(feature = "ssr")]
use std::sync::{Arc, Mutex};
#[cfg(feature = "ssr")]
use synap_core::{ServiceError, SynapService};
#[cfg(feature = "ssr")]
use tokio::task::spawn_blocking;

#[cfg(feature = "ssr")]
pub type SharedService = Arc<Mutex<SynapService>>;

#[cfg(not(feature = "ssr"))]
pub type SharedService = ();

#[cfg_attr(not(feature = "ssr"), allow(dead_code))]
const NOTE_LIMIT: usize = 50;
#[cfg_attr(not(feature = "ssr"), allow(dead_code))]
const TAG_SUGGESTION_LIMIT: usize = 6;

#[cfg(feature = "ssr")]
async fn with_service<T, F>(f: F) -> Result<T, ServerFnError>
where
    T: Send + 'static,
    F: FnOnce(&SynapService) -> Result<T, ServiceError> + Send + 'static,
{
    let service = use_context::<SharedService>()
        .ok_or_else(|| ServerFnError::new("Synap service is not available"))?;
    spawn_blocking(move || -> Result<T, ServerFnError> {
        let guard = service
            .lock()
            .map_err(|_| ServerFnError::new("Synap service lock poisoned"))?;

        f(&guard).map_err(|error| ServerFnError::new(error.to_string()))
    })
    .await
    .map_err(|error| ServerFnError::new(error.to_string()))?
}

#[server]
pub async fn list_notes(query: String) -> Result<Vec<NoteDTO>, ServerFnError> {
    let query = query.trim().to_string();
    with_service(move |service| {
        if query.is_empty() {
            service.get_recent_note(None, Some(NOTE_LIMIT))
        } else {
            service.search(&query, NOTE_LIMIT)
        }
    })
    .await
}

#[server]
pub async fn create_note_server(
    content: String,
    #[server(default)] tags: Vec<String>,
) -> Result<NoteDTO, ServerFnError> {
    with_service(move |service| service.create_note(content, tags)).await
}

#[server]
pub async fn edit_note_server(
    note_id: String,
    content: String,
    #[server(default)] tags: Vec<String>,
) -> Result<NoteDTO, ServerFnError> {
    with_service(move |service| service.edit_note(&note_id, content, tags)).await
}

#[server]
pub async fn delete_note_server(note_id: String) -> Result<(), ServerFnError> {
    with_service(move |service| service.delete_note(&note_id)).await
}

#[server]
pub async fn recommend_tags_server(content: String) -> Result<Vec<String>, ServerFnError> {
    let content = content.trim().to_string();
    if content.is_empty() {
        return Ok(Vec::new());
    }

    with_service(move |service| service.recommend_tag(&content, TAG_SUGGESTION_LIMIT)).await
}

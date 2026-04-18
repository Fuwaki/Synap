use crate::{
    core::{CoreResult, DesktopCore},
    domain::HomeData,
};

pub fn load_home(core: &dyn DesktopCore, query: &str) -> CoreResult<HomeData> {
    let trimmed = query.trim();

    let notes = if trimmed.is_empty() {
        core.recent_notes(None, Some(50))?
    } else {
        core.search(trimmed, 50)?
    };

    let deleted_notes = core.deleted_notes(None, Some(50))?;

    Ok(HomeData {
        notes,
        deleted_notes,
    })
}

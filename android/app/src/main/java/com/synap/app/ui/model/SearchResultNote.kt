package com.synap.app.ui.model

import com.synap.app.data.model.SearchResultRecord
import com.synap.app.data.model.SearchSourceRecord

enum class SearchSourceBadge {
    Fuzzy,
    Semantic,
}

data class SearchResultNote(
    val note: Note,
    val score: Float,
    val sources: List<SearchSourceBadge>,
)

fun SearchResultRecord.toUiSearchResultNote(): SearchResultNote = SearchResultNote(
    note = note.toUiNote(),
    score = score,
    sources = sources.map(SearchSourceRecord::toUiSearchSourceBadge),
)

private fun SearchSourceRecord.toUiSearchSourceBadge(): SearchSourceBadge =
    when (this) {
        SearchSourceRecord.Fuzzy -> SearchSourceBadge.Fuzzy
        SearchSourceRecord.Semantic -> SearchSourceBadge.Semantic
    }

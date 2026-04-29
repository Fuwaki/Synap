package com.synap.app.data.model

import com.fuwaki.synap.bindings.uniffi.synap_coreffi.SearchResultDto
import com.fuwaki.synap.bindings.uniffi.synap_coreffi.SearchSourceDto

enum class SearchSourceRecord {
    Fuzzy,
    Semantic,
}

data class SearchResultRecord(
    val note: NoteRecord,
    val score: Float,
    val sources: List<SearchSourceRecord>,
) {
    companion object {
        fun fromDto(dto: SearchResultDto): SearchResultRecord = SearchResultRecord(
            note = NoteRecord.fromDto(dto.note),
            score = dto.score,
            sources = dto.sources.map(SearchSourceDto::toSearchSourceRecord),
        )
    }
}

internal fun SearchSourceDto.toSearchSourceRecord(): SearchSourceRecord =
    when (this) {
        SearchSourceDto.FUZZY -> SearchSourceRecord.Fuzzy
        SearchSourceDto.SEMANTIC -> SearchSourceRecord.Semantic
    }

internal fun SearchResultDto.toSearchResultRecord(): SearchResultRecord =
    SearchResultRecord.fromDto(this)

internal fun List<SearchResultDto>.toSearchResultRecords(): List<SearchResultRecord> =
    map(SearchResultRecord::fromDto)

package com.synap.app.data.model

enum class NoteFeedStatus {
    All,
    Normal,
    Deleted,
}

data class NoteFeedFilter(
    val selectedTags: List<String> = emptyList(),
    val includeUntagged: Boolean = true,
    val tagFilterEnabled: Boolean = false,
    val status: NoteFeedStatus = NoteFeedStatus.All,
)

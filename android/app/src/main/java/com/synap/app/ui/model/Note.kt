package com.synap.app.ui.model

data class NoteBrief(
    val id: String,
    val contentPreview: String,
    val createdAt: Long,
)

data class Note(
    val id: String,
    val content: String,
    val tags: List<String>,
    val timestamp: Long,
    val parentSummary: String? = null,
    val isDeleted: Boolean = false,
    val replyTo: NoteBrief? = null,
    val editedFrom: NoteBrief? = null,
)

package com.synap.app.data.model

import com.fuwaki.synap.bindings.uniffi.synap_coreffi.NoteDto
import com.fuwaki.synap.bindings.uniffi.synap_coreffi.NoteBriefDto
import com.fuwaki.synap.bindings.uniffi.synap_coreffi.TimelineNotesPageDto
import com.synap.app.data.portal.CursorPage

data class NoteBriefRecord(
    val id: String,
    val contentPreview: String,
    val createdAt: Long,
) {
    companion object {
        fun fromDto(dto: NoteBriefDto): NoteBriefRecord = NoteBriefRecord(
            id = dto.id,
            contentPreview = dto.contentPreview,
            createdAt = dto.createdAt,
        )
    }
}

data class NoteRecord(
    val id: String,
    val content: String,
    val tags: List<String>,
    val createdAt: Long,
    val deleted: Boolean,
    val replyTo: NoteBriefRecord? = null,
    val editedFrom: NoteBriefRecord? = null,
) {
    companion object {
        fun fromDto(dto: NoteDto): NoteRecord = NoteRecord(
            id = dto.id,
            content = dto.content,
            tags = dto.tags,
            createdAt = dto.createdAt,
            deleted = dto.deleted,
            replyTo = dto.replyTo?.let(NoteBriefRecord::fromDto),
            editedFrom = dto.editedFrom?.let(NoteBriefRecord::fromDto),
        )
    }
}

data class ReplyItem(
    val note: NoteRecord,
    val parentId: String,
)

internal fun NoteDto.toNoteRecord(): NoteRecord = NoteRecord.fromDto(this)

internal fun List<NoteDto>.toNoteRecords(): List<NoteRecord> = map(NoteRecord::fromDto)

internal fun TimelineNotesPageDto.toCursorPage(): CursorPage<NoteRecord> = CursorPage(
    items = notes.toNoteRecords(),
    nextCursor = nextCursor,
)

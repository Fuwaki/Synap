package com.synap.app.data.model

import com.fuwaki.synap.bindings.uniffi.synap_coreffi.NoteSegmentBranchChoiceDto
import com.fuwaki.synap.bindings.uniffi.synap_coreffi.NoteSegmentDirectionDto
import com.fuwaki.synap.bindings.uniffi.synap_coreffi.NoteSegmentDto
import com.fuwaki.synap.bindings.uniffi.synap_coreffi.NoteSegmentStepDto

enum class NoteSegmentDirection {
    Forward,
    Backward,
    ;

    companion object {
        fun fromDto(dto: NoteSegmentDirectionDto): NoteSegmentDirection = when (dto) {
            NoteSegmentDirectionDto.FORWARD -> Forward
            NoteSegmentDirectionDto.BACKWARD -> Backward
        }
    }
}

data class NoteSegmentBranchChoiceRecord(
    val note: NoteRecord,
    val weight: UInt,
) {
    companion object {
        fun fromDto(dto: NoteSegmentBranchChoiceDto): NoteSegmentBranchChoiceRecord =
            NoteSegmentBranchChoiceRecord(
                note = dto.note.toNoteRecord(),
                weight = dto.weight,
            )
    }
}

data class NoteSegmentStepRecord(
    val note: NoteRecord,
    val nextChoices: List<NoteSegmentBranchChoiceRecord>,
    val prevChoices: List<NoteSegmentBranchChoiceRecord>,
    val stopsHere: Boolean,
) {
    companion object {
        fun fromDto(dto: NoteSegmentStepDto): NoteSegmentStepRecord =
            NoteSegmentStepRecord(
                note = dto.note.toNoteRecord(),
                nextChoices = dto.nextChoices.map(NoteSegmentBranchChoiceRecord::fromDto),
                prevChoices = dto.prevChoices.map(NoteSegmentBranchChoiceRecord::fromDto),
                stopsHere = dto.stopsHere,
            )
    }
}

data class NoteSegmentRecord(
    val anchorId: String,
    val direction: NoteSegmentDirection,
    val steps: List<NoteSegmentStepRecord>,
) {
    companion object {
        fun fromDto(dto: NoteSegmentDto): NoteSegmentRecord = NoteSegmentRecord(
            anchorId = dto.anchorId,
            direction = NoteSegmentDirection.fromDto(dto.direction),
            steps = dto.steps.map(NoteSegmentStepRecord::fromDto),
        )
    }
}

internal fun NoteSegmentDto.toNoteSegmentRecord(): NoteSegmentRecord = NoteSegmentRecord.fromDto(this)

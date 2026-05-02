package com.synap.app.ui.model

import com.synap.app.data.model.NoteSegmentBranchChoiceRecord
import com.synap.app.data.model.NoteSegmentDirection
import com.synap.app.data.model.NoteSegmentRecord
import com.synap.app.data.model.NoteSegmentStepRecord

data class ThreadBranchChoice(
    val note: Note,
    val weight: UInt,
    val isRecommended: Boolean = false,
)

data class ThreadReaderStep(
    val note: Note,
    val nextChoices: List<ThreadBranchChoice>,
    val prevChoices: List<ThreadBranchChoice>,
    val stopsHere: Boolean,
    val shouldInlineNextChoices: Boolean,
)

data class ThreadReaderSegment(
    val anchorId: String,
    val steps: List<ThreadReaderStep>,
) {
    val title: String
        get() = steps.firstOrNull()?.note?.content
            ?.lineSequence()
            ?.map(String::trim)
            ?.firstOrNull(String::isNotEmpty)
            ?.take(24)
            ?.ifBlank { "线程阅读" }
            ?: "线程阅读"
}

private fun List<NoteSegmentBranchChoiceRecord>.toUiBranchChoices(): List<ThreadBranchChoice> {
    val recommendedId = maxByOrNull { it.weight }?.note?.id
    return map { choice ->
        ThreadBranchChoice(
            note = choice.note.toUiNote(),
            weight = choice.weight,
            isRecommended = choice.note.id == recommendedId,
        )
    }
}

private fun NoteSegmentStepRecord.toUiStep(): ThreadReaderStep {
    val nextChoices = nextChoices.toUiBranchChoices()
    return ThreadReaderStep(
        note = note.toUiNote(),
        nextChoices = nextChoices,
        prevChoices = prevChoices.toUiBranchChoices(),
        stopsHere = stopsHere,
        shouldInlineNextChoices = nextChoices.size in 1..2 &&
            nextChoices.all { it.note.content.length <= 140 && it.note.tags.size <= 4 },
    )
}

private fun NoteSegmentRecord.toUiSteps(): List<ThreadReaderStep> =
    steps.map(NoteSegmentStepRecord::toUiStep)

internal fun buildThreadReaderSegment(
    anchorId: String,
    backward: NoteSegmentRecord,
    forward: NoteSegmentRecord,
): ThreadReaderSegment {
    val backwardSteps = backward.toUiSteps().drop(1).reversed()
    val forwardSteps = forward.toUiSteps()
    return ThreadReaderSegment(
        anchorId = anchorId,
        steps = backwardSteps + forwardSteps,
    )
}

internal fun NoteSegmentDirection.isForward(): Boolean = this == NoteSegmentDirection.Forward

package com.synap.app.ui.model

import com.synap.app.data.model.NoteSegmentBranchChoiceRecord
import com.synap.app.data.model.NoteNeighborsRecord
import com.synap.app.data.model.NoteSegmentRecord
import com.synap.app.data.model.NoteSegmentStepRecord

data class ThreadBranchContext(
    val parents: List<ThreadBranchChoice>,
    val children: List<ThreadBranchChoice>,
)

private fun emptyThreadBranchContext(): ThreadBranchContext = ThreadBranchContext(
    parents = emptyList(),
    children = emptyList(),
)

data class ThreadBranchChoice(
    val note: Note,
    val weight: UInt,
    val isRecommended: Boolean = false,
    val context: ThreadBranchContext = emptyThreadBranchContext(),
)

data class ThreadReaderStep(
    val note: Note,
    val parentChoices: List<ThreadBranchChoice>,
    val childChoices: List<ThreadBranchChoice>,
    val stopsHere: Boolean,
)

data class ThreadGraphNode(
    val id: String,
    val note: Note,
    val depth: Int,
    val isFocused: Boolean,
    val isInPrimarySegment: Boolean,
    val isBranchChoice: Boolean,
)

data class ThreadGraphEdge(
    val fromId: String,
    val toId: String,
    val isPrimary: Boolean,
)

data class ThreadGraph(
    val nodes: List<ThreadGraphNode>,
    val edges: List<ThreadGraphEdge>,
)

data class ThreadReaderSegment(
    val anchorId: String,
    val focusedNodeId: String,
    val steps: List<ThreadReaderStep>,
    val graph: ThreadGraph,
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

private fun NoteNeighborsRecord.contextByNoteId(
    isParentSide: Boolean,
): Map<String, ThreadBranchContext> {
    val contexts = if (isParentSide) parentContexts else childContexts
    return contexts.associate { context ->
        val excludedIds = setOf(context.note.id)
        context.note.id to ThreadBranchContext(
            parents = context.parents.filterNot { it.note.id in excludedIds }.toUiBranchChoices(),
            children = context.children.filterNot { it.note.id in excludedIds }.toUiBranchChoices(),
        )
    }
}

private fun List<NoteSegmentBranchChoiceRecord>.toUiBranchChoices(
    contextsByNoteId: Map<String, ThreadBranchContext>,
    excludedIds: Set<String>,
): List<ThreadBranchChoice> {
    val visibleChoices = filterNot { it.note.id in excludedIds }
    val recommendedId = visibleChoices.maxByOrNull { it.weight }?.note?.id
    return visibleChoices.map { choice ->
        ThreadBranchChoice(
            note = choice.note.toUiNote(),
            weight = choice.weight,
            isRecommended = choice.note.id == recommendedId,
            context = contextsByNoteId[choice.note.id] ?: emptyThreadBranchContext(),
        )
    }
}

private fun NoteSegmentStepRecord.toUiStep(
    neighbors: NoteNeighborsRecord?,
): ThreadReaderStep {
    val centerId = neighbors?.note?.id ?: note.id
    val parentChoices = neighbors?.parents?.toUiBranchChoices(
        contextsByNoteId = neighbors.contextByNoteId(isParentSide = true),
        excludedIds = setOf(centerId),
    )
        ?: prevChoices.toUiBranchChoices()
    val childChoices = neighbors?.children?.toUiBranchChoices(
        contextsByNoteId = neighbors.contextByNoteId(isParentSide = false),
        excludedIds = setOf(centerId),
    )
        ?: nextChoices.toUiBranchChoices()
    return ThreadReaderStep(
        note = (neighbors?.note ?: note).toUiNote(),
        parentChoices = parentChoices,
        childChoices = childChoices,
        stopsHere = stopsHere,
    )
}

internal fun buildThreadReaderSegment(
    anchorId: String,
    backward: NoteSegmentRecord,
    forward: NoteSegmentRecord,
    neighborsByNoteId: Map<String, NoteNeighborsRecord> = emptyMap(),
    focusedNodeId: String = anchorId,
): ThreadReaderSegment {
    val backwardSteps = backward.steps.map { step -> step.toUiStep(neighborsByNoteId[step.note.id]) }
    val forwardSteps = forward.steps.map { step -> step.toUiStep(neighborsByNoteId[step.note.id]) }
    val steps = backwardSteps.drop(1).reversed() + forwardSteps
    return ThreadReaderSegment(
        anchorId = anchorId,
        focusedNodeId = focusedNodeId,
        steps = steps,
        graph = buildThreadGraph(
            anchorId = anchorId,
            focusedNodeId = focusedNodeId,
            backwardSteps = backwardSteps,
            forwardSteps = forwardSteps,
        ),
    )
}

private fun buildThreadGraph(
    anchorId: String,
    focusedNodeId: String,
    backwardSteps: List<ThreadReaderStep>,
    forwardSteps: List<ThreadReaderStep>,
): ThreadGraph {
    val graphBackwardSteps = backwardSteps
    val graphForwardSteps = forwardSteps
    val primarySteps = graphBackwardSteps.drop(1).reversed() + graphForwardSteps
    val primaryIds = primarySteps.map { it.note.id }.toSet()
    val nodeMap = linkedMapOf<String, MutableThreadGraphNode>()
    val edgePriority = linkedMapOf<Pair<String, String>, Boolean>()

    fun upsertNode(
        note: Note,
        isInPrimarySegment: Boolean,
        isBranchChoice: Boolean,
    ) {
        val existing = nodeMap[note.id]
        if (existing == null) {
            nodeMap[note.id] = MutableThreadGraphNode(
                note = note,
                isInPrimarySegment = isInPrimarySegment,
                isBranchChoice = isBranchChoice,
            )
        } else {
            existing.isInPrimarySegment = existing.isInPrimarySegment || isInPrimarySegment
            existing.isBranchChoice = existing.isBranchChoice || isBranchChoice
        }
    }

    fun addEdge(
        fromId: String,
        toId: String,
        isPrimary: Boolean,
    ) {
        val key = fromId to toId
        edgePriority[key] = (edgePriority[key] == true) || isPrimary
    }

    fun addGraphNode(
        note: Note,
        isInPrimarySegment: Boolean,
        isBranchChoice: Boolean,
    ) {
        upsertNode(
            note = note,
            isInPrimarySegment = isInPrimarySegment,
            isBranchChoice = isBranchChoice,
        )
    }

    primarySteps.forEach { step ->
        addGraphNode(
            note = step.note,
            isInPrimarySegment = true,
            isBranchChoice = false,
        )
    }

    graphForwardSteps.zipWithNext { parent, child ->
        addEdge(
            fromId = parent.note.id,
            toId = child.note.id,
            isPrimary = true,
        )
    }

    graphBackwardSteps.zipWithNext { child, parent ->
        addEdge(
            fromId = parent.note.id,
            toId = child.note.id,
            isPrimary = true,
        )
    }

    primarySteps.forEach { step ->
        step.parentChoices.forEach { choice ->
            addGraphNode(
                note = choice.note,
                isInPrimarySegment = choice.note.id in primaryIds,
                isBranchChoice = choice.note.id !in primaryIds,
            )
            addEdge(
                fromId = choice.note.id,
                toId = step.note.id,
                isPrimary = choice.note.id in primaryIds,
            )
            choice.context.parents.forEach { contextChoice ->
                addGraphNode(
                    note = contextChoice.note,
                    isInPrimarySegment = contextChoice.note.id in primaryIds,
                    isBranchChoice = contextChoice.note.id !in primaryIds,
                )
                addEdge(
                    fromId = contextChoice.note.id,
                    toId = choice.note.id,
                    isPrimary = contextChoice.note.id in primaryIds && choice.note.id in primaryIds,
                )
            }
            choice.context.children.forEach { contextChoice ->
                addGraphNode(
                    note = contextChoice.note,
                    isInPrimarySegment = contextChoice.note.id in primaryIds,
                    isBranchChoice = contextChoice.note.id !in primaryIds,
                )
                addEdge(
                    fromId = choice.note.id,
                    toId = contextChoice.note.id,
                    isPrimary = contextChoice.note.id in primaryIds && choice.note.id in primaryIds,
                )
            }
        }

        step.childChoices.forEach { choice ->
            addGraphNode(
                note = choice.note,
                isInPrimarySegment = choice.note.id in primaryIds,
                isBranchChoice = choice.note.id !in primaryIds,
            )
            addEdge(
                fromId = step.note.id,
                toId = choice.note.id,
                isPrimary = choice.note.id in primaryIds,
            )
            choice.context.parents.forEach { contextChoice ->
                addGraphNode(
                    note = contextChoice.note,
                    isInPrimarySegment = contextChoice.note.id in primaryIds,
                    isBranchChoice = contextChoice.note.id !in primaryIds,
                )
                addEdge(
                    fromId = contextChoice.note.id,
                    toId = choice.note.id,
                    isPrimary = contextChoice.note.id in primaryIds && choice.note.id in primaryIds,
                )
            }
            choice.context.children.forEach { contextChoice ->
                addGraphNode(
                    note = contextChoice.note,
                    isInPrimarySegment = contextChoice.note.id in primaryIds,
                    isBranchChoice = contextChoice.note.id !in primaryIds,
                )
                addEdge(
                    fromId = choice.note.id,
                    toId = contextChoice.note.id,
                    isPrimary = contextChoice.note.id in primaryIds && choice.note.id in primaryIds,
                )
            }
        }
    }

    val edges = edgePriority.map { (edge, isPrimary) ->
        ThreadGraphEdge(
            fromId = edge.first,
            toId = edge.second,
            isPrimary = isPrimary,
        )
    }
    val depths = computeThreadGraphDepths(
        anchorId = anchorId,
        nodeIds = nodeMap.keys,
        edges = edges,
    )

    return ThreadGraph(
        nodes = nodeMap.map { (id, node) ->
            ThreadGraphNode(
                id = id,
                note = node.note,
                depth = depths[id] ?: 0,
                isFocused = id == focusedNodeId,
                isInPrimarySegment = node.isInPrimarySegment,
                isBranchChoice = node.isBranchChoice && !node.isInPrimarySegment,
            )
        },
        edges = edges,
    )
}

private data class MutableThreadGraphNode(
    val note: Note,
    var isInPrimarySegment: Boolean,
    var isBranchChoice: Boolean,
)

private fun computeThreadGraphDepths(
    anchorId: String,
    nodeIds: Set<String>,
    edges: List<ThreadGraphEdge>,
): Map<String, Int> {
    if (nodeIds.isEmpty()) return emptyMap()

    val outgoing = nodeIds.associateWith { mutableListOf<String>() }.toMutableMap()
    val indegree = nodeIds.associateWith { 0 }.toMutableMap()

    edges.forEach { edge ->
        if (edge.fromId !in nodeIds || edge.toId !in nodeIds) return@forEach
        outgoing.getValue(edge.fromId).add(edge.toId)
        indegree[edge.toId] = indegree.getValue(edge.toId) + 1
    }

    val queue = ArrayDeque(
        indegree
            .filterValues { it == 0 }
            .keys
            .sortedWith(compareBy<String> { if (it == anchorId) 0 else 1 }.thenBy { it }),
    )
    val topoOrder = mutableListOf<String>()
    val mutableIndegree = indegree.toMutableMap()

    while (queue.isNotEmpty()) {
        val nodeId = queue.removeFirst()
        topoOrder += nodeId
        outgoing.getValue(nodeId)
            .sorted()
            .forEach { targetId ->
                val nextIndegree = mutableIndegree.getValue(targetId) - 1
                mutableIndegree[targetId] = nextIndegree
                if (nextIndegree == 0) {
                    queue.addLast(targetId)
                }
            }
    }

    if (topoOrder.size < nodeIds.size) {
        topoOrder += nodeIds
            .filterNot(topoOrder::contains)
            .sorted()
    }

    val depths = topoOrder.associateWith { 0 }.toMutableMap()
    topoOrder.forEach { nodeId ->
        val sourceDepth = depths.getValue(nodeId)
        outgoing.getValue(nodeId).forEach { targetId ->
            depths[targetId] = maxOf(depths.getValue(targetId), sourceDepth + 1)
        }
    }

    val anchorDepth = depths[anchorId] ?: 0
    return depths.mapValues { (_, depth) -> depth - anchorDepth }
}

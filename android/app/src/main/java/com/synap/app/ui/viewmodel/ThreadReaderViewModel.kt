package com.synap.app.ui.viewmodel

import androidx.lifecycle.SavedStateHandle
import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import com.synap.app.data.model.NoteNeighborsRecord
import com.synap.app.data.model.NoteSegmentDirection
import com.synap.app.data.model.NoteSegmentRecord
import com.synap.app.data.repository.SynapMutation
import com.synap.app.data.repository.SynapRepository
import com.synap.app.ui.model.ThreadBranchChoice
import com.synap.app.ui.model.ThreadReaderSegment
import com.synap.app.ui.model.buildThreadReaderSegment
import dagger.hilt.android.lifecycle.HiltViewModel
import javax.inject.Inject
import kotlinx.coroutines.async
import kotlinx.coroutines.awaitAll
import kotlinx.coroutines.coroutineScope
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.launch

data class ThreadReaderUiState(
    val segment: ThreadReaderSegment? = null,
    val originNoteId: String = "",
    val currentAnchorId: String = "",
    val focusedNodeId: String = "",
    val historyDepth: Int = 0,
    val isLoading: Boolean = true,
    val isBranchSheetVisible: Boolean = false,
    val isGraphSheetVisible: Boolean = false,
    val activeBranchChoices: List<ThreadBranchChoice> = emptyList(),
    val errorMessage: String? = null,
)

private data class ThreadHistoryEntry(
    val anchorId: String,
)

private data class ThreadCacheEntry(
    val backward: NoteSegmentRecord,
    val forward: NoteSegmentRecord,
    val neighborsByNoteId: Map<String, NoteNeighborsRecord>,
)

@HiltViewModel
class ThreadReaderViewModel @Inject constructor(
    savedStateHandle: SavedStateHandle,
    private val repository: SynapRepository,
) : ViewModel() {
    private val originNoteId: String = checkNotNull(savedStateHandle["noteId"])

    private val _uiState = MutableStateFlow(
        ThreadReaderUiState(
            originNoteId = originNoteId,
            currentAnchorId = originNoteId,
            focusedNodeId = originNoteId,
        ),
    )
    val uiState: StateFlow<ThreadReaderUiState> = _uiState.asStateFlow()

    private val history = ArrayDeque<ThreadHistoryEntry>()
    private val cache = LinkedHashMap<String, ThreadCacheEntry>()

    init {
        loadSegment(originNoteId, pushHistory = false)

        viewModelScope.launch {
            repository.mutations.collect {
                cache.clear()
                loadSegment(
                    anchorId = _uiState.value.currentAnchorId,
                    pushHistory = false,
                )
            }
        }
    }

    fun refresh() {
        cache.remove(_uiState.value.currentAnchorId)
        loadSegment(
            anchorId = _uiState.value.currentAnchorId,
            pushHistory = false,
        )
    }

    fun selectBranch(choice: ThreadBranchChoice) {
        dismissBranchSheet()
        loadSegment(choice.note.id, pushHistory = true)
    }

    fun openNodeAsAnchor(noteId: String) {
        if (_uiState.value.currentAnchorId == noteId) {
            focusNode(noteId)
            return
        }
        loadSegment(noteId, pushHistory = true)
    }

    fun openBranchSheet(choices: List<ThreadBranchChoice>) {
        _uiState.value = _uiState.value.copy(
            isBranchSheetVisible = true,
            activeBranchChoices = choices,
        )
    }

    fun openGraphSheet() {
        _uiState.value = _uiState.value.copy(isGraphSheetVisible = true)
    }

    fun dismissGraphSheet() {
        _uiState.value = _uiState.value.copy(isGraphSheetVisible = false)
    }

    fun focusNode(noteId: String) {
        val current = _uiState.value
        val segment = current.segment ?: return
        if (segment.focusedNodeId == noteId) return

        _uiState.value = current.copy(
            segment = segment.copy(
                focusedNodeId = noteId,
                graph = segment.graph.copy(
                    nodes = segment.graph.nodes.map { node ->
                        node.copy(isFocused = node.id == noteId)
                    },
                ),
            ),
            focusedNodeId = noteId,
        )
    }

    fun dismissBranchSheet() {
        _uiState.value = _uiState.value.copy(
            isBranchSheetVisible = false,
            activeBranchChoices = emptyList(),
        )
    }

    fun goBackInHistory() {
        val previous = history.removeLastOrNull() ?: return
        loadSegment(previous.anchorId, pushHistory = false)
    }

    private fun loadSegment(
        anchorId: String,
        pushHistory: Boolean,
    ) {
        val current = _uiState.value
        if (pushHistory && current.segment != null) {
            history.addLast(ThreadHistoryEntry(anchorId = current.currentAnchorId))
        }

        val cached = cache[anchorId]
        if (cached != null) {
            _uiState.value = _uiState.value.copy(
                segment = buildThreadReaderSegment(
                    anchorId = anchorId,
                    backward = cached.backward,
                    forward = cached.forward,
                    neighborsByNoteId = cached.neighborsByNoteId,
                ),
                currentAnchorId = anchorId,
                focusedNodeId = anchorId,
                historyDepth = history.size,
                isLoading = false,
                isBranchSheetVisible = false,
                isGraphSheetVisible = false,
                activeBranchChoices = emptyList(),
                errorMessage = null,
            )
            return
        }

        _uiState.value = _uiState.value.copy(
            currentAnchorId = anchorId,
            focusedNodeId = anchorId,
            historyDepth = history.size,
            isLoading = true,
            isBranchSheetVisible = false,
            isGraphSheetVisible = false,
            activeBranchChoices = emptyList(),
            errorMessage = null,
        )

        viewModelScope.launch {
            runCatching {
                coroutineScope {
                    val backwardDeferred = async {
                        repository.getNoteSegment(anchorId, NoteSegmentDirection.Backward)
                    }
                    val forwardDeferred = async {
                        repository.getNoteSegment(anchorId, NoteSegmentDirection.Forward)
                    }
                    val entry = ThreadCacheEntry(
                        backward = backwardDeferred.await(),
                        forward = forwardDeferred.await(),
                        neighborsByNoteId = emptyMap(),
                    )
                    val routeIds = (entry.backward.steps.asSequence() + entry.forward.steps.asSequence())
                        .map { step -> step.note.id }
                        .distinct()
                        .toList()
                    val neighbors = routeIds
                        .map { noteId ->
                            async { noteId to repository.getNoteNeighbors(noteId) }
                        }
                        .awaitAll()
                        .toMap()
                    entry.copy(neighborsByNoteId = neighbors)
                }
            }.fold(
                onSuccess = { entry ->
                    cache[anchorId] = entry
                    _uiState.value = _uiState.value.copy(
                        segment = buildThreadReaderSegment(
                            anchorId = anchorId,
                            backward = entry.backward,
                            forward = entry.forward,
                            neighborsByNoteId = entry.neighborsByNoteId,
                        ),
                        currentAnchorId = anchorId,
                        focusedNodeId = anchorId,
                        historyDepth = history.size,
                        isLoading = false,
                        errorMessage = null,
                    )
                },
                onFailure = { throwable ->
                    _uiState.value = _uiState.value.copy(
                        isLoading = false,
                        errorMessage = throwable.message ?: "Failed to load thread",
                    )
                },
            )
        }
    }
}

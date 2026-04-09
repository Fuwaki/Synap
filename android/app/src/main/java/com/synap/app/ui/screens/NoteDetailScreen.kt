package com.synap.app.ui.screens

import androidx.compose.animation.AnimatedVisibility
import androidx.compose.animation.fadeIn
import androidx.compose.animation.fadeOut
import androidx.compose.foundation.clickable
import androidx.compose.foundation.horizontalScroll
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.verticalScroll
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.ArrowBack
import androidx.compose.material.icons.filled.ArrowUpward
import androidx.compose.material.icons.filled.Delete
import androidx.compose.material.icons.filled.Edit
import androidx.compose.material.icons.filled.Home
import androidx.compose.material.icons.filled.Reply
import androidx.compose.material.icons.filled.Share
import androidx.compose.material3.CircularProgressIndicator
import androidx.compose.material3.ExperimentalMaterial3Api
import androidx.compose.material3.ExperimentalMaterial3ExpressiveApi
import androidx.compose.material3.ExtendedFloatingActionButton
import androidx.compose.material3.HorizontalFloatingToolbar
import androidx.compose.material3.Icon
import androidx.compose.material3.IconButton
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.OutlinedButton
import androidx.compose.material3.Scaffold
import androidx.compose.material3.Surface
import androidx.compose.material3.Text
import androidx.compose.material3.TextButton
import androidx.compose.material3.TopAppBar
import androidx.compose.runtime.Composable
import androidx.compose.runtime.derivedStateOf
import androidx.compose.runtime.getValue
import androidx.compose.runtime.remember
import androidx.compose.runtime.rememberCoroutineScope
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import com.synap.app.LocalNoteFontFamily
import com.synap.app.LocalNoteFontWeight
import com.synap.app.LocalNoteLineSpacing
import com.synap.app.LocalNoteTextSize
import com.synap.app.R
import com.synap.app.ui.model.Note
import com.synap.app.ui.util.formatNoteTime
import com.synap.app.ui.viewmodel.DetailUiState
import kotlinx.coroutines.launch

@Suppress("OPT_IN_USAGE", "OPT_IN_USAGE_FUTURE_ERROR")
@OptIn(ExperimentalMaterial3Api::class, ExperimentalMaterial3ExpressiveApi::class)
@Composable
fun NoteDetailScreen(
    uiState: DetailUiState,
    onNavigateBack: () -> Unit,
    onNavigateHome: () -> Unit,
    onDelete: () -> Unit,
    onReply: () -> Unit,
    onEdit: () -> Unit,
    onOpenRelatedNote: (String) -> Unit,
    onLoadMoreReplies: () -> Unit,
    onRefresh: () -> Unit,
) {
    val scrollState = rememberScrollState()
    val scope = rememberCoroutineScope()

    val isScrolledDown by remember {
        derivedStateOf {
            scrollState.value > 300
        }
    }

    Scaffold(
        topBar = {
            TopAppBar(
                title = { Text(stringResource(R.string.notedetail_title)) },
                navigationIcon = {
                    Row(verticalAlignment = Alignment.CenterVertically) {
                        IconButton(onClick = onNavigateBack) {
                            Icon(Icons.Filled.ArrowBack, contentDescription = stringResource(R.string.back))
                        }
                        IconButton(onClick = onNavigateHome) {
                            Icon(Icons.Filled.Home, contentDescription = stringResource(R.string.home))
                        }
                    }
                },
            )
        },
        bottomBar = {
            if (uiState.note != null) {
                // 使用 Box 仅用于将其居中并设置底边距
                Box(
                    modifier = Modifier
                        .fillMaxWidth()
                        .padding(bottom = 24.dp),
                    contentAlignment = Alignment.Center
                ) {
                    // --- 彻底使用官方原生组件，无自定义颜色 ---
                    HorizontalFloatingToolbar(
                        expanded = true, // 让工具栏保持展开状态以显示内部按钮
                    ) {
                        TextButton(onClick = onDelete) {
                            Icon(Icons.Filled.Delete, contentDescription = null, modifier = Modifier.size(18.dp))
                            Spacer(modifier = Modifier.width(6.dp))
                            Text(stringResource(R.string.delete))
                        }
                        TextButton(onClick = { /* 仅作 UI 展示，暂无功能 */ }) {
                            Icon(Icons.Filled.Share, contentDescription = null, modifier = Modifier.size(18.dp))
                            Spacer(modifier = Modifier.width(6.dp))
                            Text("分享")
                        }
                        TextButton(onClick = onReply) {
                            Icon(Icons.Filled.Reply, contentDescription = null, modifier = Modifier.size(18.dp))
                            Spacer(modifier = Modifier.width(6.dp))
                            Text(stringResource(R.string.reply))
                        }
                        TextButton(onClick = onEdit) {
                            Icon(Icons.Filled.Edit, contentDescription = null, modifier = Modifier.size(18.dp))
                            Spacer(modifier = Modifier.width(6.dp))
                            Text(stringResource(R.string.edit))
                        }
                    }
                }
            }
        },
        floatingActionButton = {
            if (uiState.note != null) {
                AnimatedVisibility(
                    visible = isScrolledDown,
                    enter = fadeIn(),
                    exit = fadeOut()
                ) {
                    ExtendedFloatingActionButton(
                        onClick = {
                            scope.launch {
                                scrollState.animateScrollTo(0)
                            }
                        },
                        icon = { Icon(Icons.Filled.ArrowUpward, contentDescription = null) },
                        text = { Text(text = stringResource(R.string.backtop)) },
                        containerColor = MaterialTheme.colorScheme.secondaryContainer,
                        contentColor = MaterialTheme.colorScheme.onSecondaryContainer,
                        modifier = Modifier.padding(bottom = 80.dp) // 避开底部工具栏
                    )
                }
            }
        }
    ) { innerPadding ->
        if (uiState.isLoading && uiState.note == null) {
            Column(
                modifier = Modifier
                    .fillMaxSize()
                    .padding(innerPadding),
                horizontalAlignment = Alignment.CenterHorizontally,
                verticalArrangement = Arrangement.Center,
            ) {
                CircularProgressIndicator()
            }
            return@Scaffold
        }

        if (uiState.note == null) {
            Column(
                modifier = Modifier
                    .fillMaxSize()
                    .padding(innerPadding)
                    .padding(24.dp),
                verticalArrangement = Arrangement.Center,
            ) {
                Text(
                    text = uiState.errorMessage ?: stringResource(R.string.notedetail_errorMessage),
                    style = MaterialTheme.typography.bodyLarge,
                    color = MaterialTheme.colorScheme.error,
                )
                OutlinedButton(
                    onClick = onRefresh,
                    modifier = Modifier.padding(top = 16.dp),
                ) {
                    Text("重试")
                }
            }
            return@Scaffold
        }

        val note = uiState.note

        Column(
            modifier = Modifier
                .fillMaxSize()
                .padding(innerPadding)
                .verticalScroll(scrollState)
                .padding(16.dp),
        ) {
            Row(
                verticalAlignment = Alignment.CenterVertically,
                modifier = Modifier.padding(bottom = 16.dp),
            ) {
                Text(
                    text = formatNoteTime(note.timestamp),
                    style = MaterialTheme.typography.bodyMedium,
                    color = MaterialTheme.colorScheme.onSurfaceVariant,
                    modifier = Modifier.padding(end = 12.dp),
                )
                Row(
                    modifier = Modifier.horizontalScroll(rememberScrollState()),
                    horizontalArrangement = Arrangement.spacedBy(8.dp)
                ) {
                    note.tags.forEach { tag ->
                        Surface(
                            color = MaterialTheme.colorScheme.secondaryContainer,
                            shape = MaterialTheme.shapes.small,
                        ) {
                            Text(
                                text = tag,
                                modifier = Modifier.padding(horizontal = 8.dp, vertical = 4.dp),
                                style = MaterialTheme.typography.labelSmall,
                            )
                        }
                    }
                }
            }

            Text(
                text = note.content,
                style = MaterialTheme.typography.bodyLarge.copy(
                    fontFamily = LocalNoteFontFamily.current,
                    fontWeight = LocalNoteFontWeight.current,
                    fontSize = LocalNoteTextSize.current,
                    lineHeight = LocalNoteTextSize.current * LocalNoteLineSpacing.current
                ),
                modifier = Modifier.fillMaxWidth(),
            )

            if (uiState.errorMessage != null) {
                Text(
                    text = uiState.errorMessage,
                    color = MaterialTheme.colorScheme.error,
                    modifier = Modifier.padding(top = 16.dp),
                )
            }

            RelationSection(
                title = stringResource(R.string.notedetail_origins),
                notes = uiState.origins,
                onOpenRelatedNote = onOpenRelatedNote,
            )
            RelationSection(
                title = stringResource(R.string.notedetail_previousVersions),
                notes = uiState.previousVersions,
                onOpenRelatedNote = onOpenRelatedNote,
            )
            RelationSection(
                title = stringResource(R.string.notedetail_nextVersions),
                notes = uiState.nextVersions,
                onOpenRelatedNote = onOpenRelatedNote,
            )
            RelationSection(
                title = stringResource(R.string.notedetail_replies),
                notes = uiState.replies,
                onOpenRelatedNote = onOpenRelatedNote,
            )

            if (uiState.repliesHasMore) {
                OutlinedButton(
                    onClick = onLoadMoreReplies,
                    modifier = Modifier.padding(top = 12.dp),
                ) {
                    Text(if (uiState.repliesLoading) "加载中..." else "加载更多回复")
                }
            }
            Spacer(modifier = Modifier.height(200.dp))
        }
    }
}

@Composable
private fun RelationSection(
    title: String,
    notes: List<Note>,
    onOpenRelatedNote: (String) -> Unit,
) {
    if (notes.isEmpty()) {
        return
    }

    Text(
        text = title,
        style = MaterialTheme.typography.titleMedium,
        color = MaterialTheme.colorScheme.primary,
        modifier = Modifier.padding(top = 24.dp, bottom = 12.dp),
    )

    Column(verticalArrangement = Arrangement.spacedBy(10.dp)) {
        notes.forEach { note ->
            Surface(
                shape = MaterialTheme.shapes.medium,
                color = MaterialTheme.colorScheme.surfaceVariant,
                modifier = Modifier
                    .fillMaxWidth()
                    .clickable { onOpenRelatedNote(note.id) },
            ) {
                Column(modifier = Modifier.padding(14.dp)) {
                    Text(
                        text = note.content,
                        style = MaterialTheme.typography.bodyMedium.copy(
                            fontFamily = LocalNoteFontFamily.current,
                            fontWeight = LocalNoteFontWeight.current,
                            fontSize = (LocalNoteTextSize.current.value - 2).coerceAtLeast(10f).sp,
                            lineHeight = (LocalNoteTextSize.current.value - 2).coerceAtLeast(10f).sp * LocalNoteLineSpacing.current
                        ),
                    )
                    if (note.tags.isNotEmpty()) {
                        Text(
                            text = note.tags.joinToString(" · "),
                            style = MaterialTheme.typography.labelSmall,
                            color = MaterialTheme.colorScheme.onSurfaceVariant,
                            modifier = Modifier.padding(top = 6.dp),
                        )
                    }
                }
            }
        }
    }
}
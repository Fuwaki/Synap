package com.synap.app.ui.screens

import androidx.compose.animation.AnimatedVisibility
import androidx.compose.animation.fadeIn
import androidx.compose.animation.fadeOut
import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.navigationBarsPadding
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.itemsIndexed
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.AccountTree
import androidx.compose.material.icons.filled.ArrowBack
import androidx.compose.material3.Card
import androidx.compose.material3.CardDefaults
import androidx.compose.material3.ExperimentalMaterial3Api
import androidx.compose.material3.ExperimentalMaterial3ExpressiveApi
import androidx.compose.material3.FilledTonalButton
import androidx.compose.material3.FloatingToolbarDefaults
import androidx.compose.material3.HorizontalFloatingToolbar
import androidx.compose.material3.Icon
import androidx.compose.material3.IconButton
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.ModalBottomSheet
import androidx.compose.material3.OutlinedButton
import androidx.compose.material3.Scaffold
import androidx.compose.material3.Surface
import androidx.compose.material3.Text
import androidx.compose.material3.TextButton
import androidx.compose.material3.TopAppBar
import androidx.compose.material3.rememberModalBottomSheetState
import androidx.compose.runtime.Composable
import androidx.compose.runtime.remember
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import com.synap.app.LocalNoteFontFamily
import com.synap.app.LocalNoteFontWeight
import com.synap.app.LocalNoteLineSpacing
import com.synap.app.LocalNoteTextSize
import com.synap.app.ui.model.Note
import com.synap.app.ui.model.ThreadBranchChoice
import com.synap.app.ui.viewmodel.ThreadReaderUiState

@OptIn(ExperimentalMaterial3Api::class, ExperimentalMaterial3ExpressiveApi::class)
@Composable
fun ThreadReaderScreen(
    uiState: ThreadReaderUiState,
    onNavigateBack: () -> Unit,
    onOpenOriginDetail: (String) -> Unit,
    onOpenBranch: (ThreadBranchChoice) -> Unit,
    onShowBranchSheet: (List<ThreadBranchChoice>) -> Unit,
    onDismissBranchSheet: () -> Unit,
    onBacktrack: () -> Unit,
    onRefresh: () -> Unit,
) {
    val branchSheetState = rememberModalBottomSheetState(skipPartiallyExpanded = true)

    if (uiState.isBranchSheetVisible) {
        ModalBottomSheet(
            onDismissRequest = onDismissBranchSheet,
            sheetState = branchSheetState,
            containerColor = MaterialTheme.colorScheme.surfaceContainerLow,
        ) {
            Column(
                modifier = Modifier
                    .fillMaxWidth()
                    .padding(horizontal = 20.dp, vertical = 12.dp),
                verticalArrangement = Arrangement.spacedBy(12.dp),
            ) {
                Text(
                    text = "选择继续延伸的路径",
                    style = MaterialTheme.typography.titleLarge,
                    fontWeight = FontWeight.SemiBold,
                )
                uiState.activeBranchChoices.forEach { choice ->
                    BranchChoiceSheetCard(
                        choice = choice,
                        onClick = { onOpenBranch(choice) },
                    )
                }
                Spacer(modifier = Modifier.height(12.dp))
            }
        }
    }
    
    Scaffold(
        topBar = {
            TopAppBar(
                title = {
                    Column {
                        Text(
                            text = uiState.segment?.title ?: "线程阅读",
                            maxLines = 1,
                            overflow = TextOverflow.Ellipsis,
                        )
                        Text(
                            text = "上下文连续阅读",
                            style = MaterialTheme.typography.labelMedium,
                            color = MaterialTheme.colorScheme.onSurfaceVariant,
                        )
                    }
                },
                navigationIcon = {
                    IconButton(onClick = onNavigateBack) {
                        Icon(Icons.Filled.ArrowBack, contentDescription = "返回")
                    }
                },
                actions = {
                    IconButton(onClick = { onOpenOriginDetail(uiState.originNoteId) }) {
                        Icon(Icons.Filled.AccountTree, contentDescription = "返回详情")
                    }
                },
            )
        },
    ) { innerPadding ->
        Box(
            modifier = Modifier
                .fillMaxSize()
                .background(MaterialTheme.colorScheme.surface)
                .padding(innerPadding),
        ) {
            when {
                uiState.isLoading && uiState.segment == null -> {
                    Column(
                        modifier = Modifier
                            .fillMaxSize()
                            .padding(24.dp),
                        verticalArrangement = Arrangement.Center,
                        horizontalAlignment = Alignment.CenterHorizontally,
                    ) {
                        androidx.compose.material3.CircularProgressIndicator()
                    }
                }

                uiState.segment == null -> {
                    Column(
                        modifier = Modifier
                            .fillMaxSize()
                            .padding(24.dp),
                        verticalArrangement = Arrangement.Center,
                        horizontalAlignment = Alignment.CenterHorizontally,
                    ) {
                        Text(
                            text = uiState.errorMessage ?: "加载失败",
                            color = MaterialTheme.colorScheme.error,
                        )
                        OutlinedButton(
                            onClick = onRefresh,
                            modifier = Modifier.padding(top = 16.dp),
                        ) {
                            Text("重试")
                        }
                    }
                }

                else -> {
                    val segment = checkNotNull(uiState.segment)
                    LazyColumn(
                        modifier = Modifier.fillMaxSize(),
                        verticalArrangement = Arrangement.spacedBy(14.dp),
                        contentPadding = androidx.compose.foundation.layout.PaddingValues(
                            start = 16.dp,
                            end = 16.dp,
                            top = 12.dp,
                            bottom = 120.dp,
                        ),
                    ) {
                        itemsIndexed(segment.steps, key = { _, step -> step.note.id }) { index, step ->
                            ThreadStepCard(
                                note = step.note,
                                isFirst = index == 0,
                                isDecisionPoint = step.stopsHere,
                            )

                            if (step.shouldInlineNextChoices && step.nextChoices.size > 1) {
                                InlineBranchChoices(
                                    choices = step.nextChoices,
                                    onOpenBranch = onOpenBranch,
                                )
                            } else if (step.stopsHere && step.nextChoices.size > 1) {
                                BranchDecisionFooter(
                                    choices = step.nextChoices,
                                    onOpenRecommended = {
                                        step.nextChoices.firstOrNull { it.isRecommended }?.let(onOpenBranch)
                                    },
                                    onShowAll = { onShowBranchSheet(step.nextChoices) },
                                )
                            }
                        }
                    }
                }
            }

            AnimatedVisibility(
                visible = uiState.segment != null,
                enter = fadeIn(),
                exit = fadeOut(),
                modifier = Modifier
                    .align(Alignment.BottomCenter)
                    .navigationBarsPadding()
                    .padding(bottom = 20.dp),
            ) {
                HorizontalFloatingToolbar(
                    expanded = true,
                    colors = FloatingToolbarDefaults.vibrantFloatingToolbarColors(
                        toolbarContainerColor = MaterialTheme.colorScheme.secondaryContainer,
                        toolbarContentColor = MaterialTheme.colorScheme.onSecondaryContainer,
                    ),
                ) {
                    TextButton(
                        onClick = onBacktrack,
                        enabled = uiState.historyDepth > 0,
                    ) {
                        Text("返回上一分支")
                    }
                    TextButton(onClick = { onOpenOriginDetail(uiState.originNoteId) }) {
                        Text("查看原笔记")
                    }
                    FilledTonalButton(onClick = onRefresh) {
                        Text("刷新当前线程")
                    }
                }
            }
        }
    }
}

@Composable
private fun ThreadStepCard(
    note: Note,
    isFirst: Boolean,
    isDecisionPoint: Boolean,
) {
    val primaryColor = MaterialTheme.colorScheme.primary
    val highlightColor = MaterialTheme.colorScheme.tertiaryContainer
    val baseFontSize = LocalNoteTextSize.current.value
    val annotatedContent = remember(note.content, primaryColor, highlightColor, baseFontSize) {
        buildMarkdownAnnotatedString(
            text = note.content,
            primaryColor = primaryColor,
            highlightColor = highlightColor,
            baseFontSize = baseFontSize,
            isCompact = false,
        )
    }

    Card(
        modifier = Modifier.fillMaxWidth(),
        shape = if (isFirst) RoundedCornerShape(28.dp) else RoundedCornerShape(24.dp),
        colors = CardDefaults.cardColors(
            containerColor = if (isDecisionPoint) {
                MaterialTheme.colorScheme.surfaceContainer
            } else {
                MaterialTheme.colorScheme.surfaceContainerLow
            },
        ),
        elevation = CardDefaults.cardElevation(defaultElevation = if (isDecisionPoint) 3.dp else 1.dp),
    ) {
        Column(
            modifier = Modifier
                .fillMaxWidth()
                .padding(horizontal = 18.dp, vertical = 16.dp),
            verticalArrangement = Arrangement.spacedBy(12.dp),
        ) {
            Row(
                modifier = Modifier.fillMaxWidth(),
                horizontalArrangement = Arrangement.SpaceBetween,
                verticalAlignment = Alignment.CenterVertically,
            ) {
                Text(
                    text = note.tags.joinToString(" · ").ifBlank { "无标签" },
                    style = MaterialTheme.typography.labelMedium,
                    color = MaterialTheme.colorScheme.onSurfaceVariant,
                    maxLines = 1,
                    overflow = TextOverflow.Ellipsis,
                    modifier = Modifier.weight(1f, fill = false),
                )
                if (isDecisionPoint) {
                    Surface(
                        color = MaterialTheme.colorScheme.secondaryContainer,
                        shape = RoundedCornerShape(999.dp),
                    ) {
                        Text(
                            text = "决策点",
                            style = MaterialTheme.typography.labelMedium,
                            color = MaterialTheme.colorScheme.onSecondaryContainer,
                            modifier = Modifier.padding(horizontal = 10.dp, vertical = 5.dp),
                        )
                    }
                }
            }

            Text(
                text = annotatedContent,
                style = MaterialTheme.typography.bodyLarge.copy(
                    fontFamily = LocalNoteFontFamily.current,
                    fontWeight = LocalNoteFontWeight.current,
                    fontSize = LocalNoteTextSize.current,
                    lineHeight = LocalNoteTextSize.current * LocalNoteLineSpacing.current,
                ),
                modifier = Modifier.fillMaxWidth(),
            )
        }
    }
}

@Composable
private fun InlineBranchChoices(
    choices: List<ThreadBranchChoice>,
    onOpenBranch: (ThreadBranchChoice) -> Unit,
) {
    Column(
        verticalArrangement = Arrangement.spacedBy(10.dp),
        modifier = Modifier.padding(top = 2.dp, bottom = 6.dp),
    ) {
        Text(
            text = "继续延伸",
            style = MaterialTheme.typography.titleSmall,
            color = MaterialTheme.colorScheme.primary,
            modifier = Modifier.padding(horizontal = 4.dp),
        )
        choices.forEach { choice ->
            BranchPreviewCard(choice = choice, onClick = { onOpenBranch(choice) })
        }
    }
}

@Composable
private fun BranchDecisionFooter(
    choices: List<ThreadBranchChoice>,
    onOpenRecommended: () -> Unit,
    onShowAll: () -> Unit,
) {
    Surface(
        modifier = Modifier.fillMaxWidth(),
        color = MaterialTheme.colorScheme.tertiaryContainer.copy(alpha = 0.5f),
        shape = MaterialTheme.shapes.large,
    ) {
        Column(
            modifier = Modifier
                .fillMaxWidth()
                .padding(16.dp),
            verticalArrangement = Arrangement.spacedBy(12.dp),
        ) {
            Text(
                text = "这里可以沿 ${choices.size} 条路径继续阅读",
                style = MaterialTheme.typography.titleMedium,
                color = MaterialTheme.colorScheme.onSurface,
            )
            Row(horizontalArrangement = Arrangement.spacedBy(10.dp)) {
                FilledTonalButton(
                    onClick = onOpenRecommended,
                    enabled = choices.isNotEmpty(),
                ) {
                    Text("继续推荐路径")
                }
                OutlinedButton(onClick = onShowAll) {
                    Text("查看全部路径")
                }
            }
        }
    }
}

@Composable
private fun BranchPreviewCard(
    choice: ThreadBranchChoice,
    onClick: () -> Unit,
) {
    val primaryColor = MaterialTheme.colorScheme.primary
    val highlightColor = MaterialTheme.colorScheme.tertiaryContainer
    val previewSize = (LocalNoteTextSize.current.value - 2).coerceAtLeast(11f)
    val annotatedContent = remember(choice.note.content, primaryColor, highlightColor, previewSize) {
        buildMarkdownAnnotatedString(
            text = choice.note.content,
            primaryColor = primaryColor,
            highlightColor = highlightColor,
            baseFontSize = previewSize,
            isCompact = true,
        )
    }

    Surface(
        modifier = Modifier
            .fillMaxWidth()
            .clip(MaterialTheme.shapes.medium)
            .clickable(onClick = onClick),
        color = if (choice.isRecommended) {
            MaterialTheme.colorScheme.secondaryContainer
        } else {
            MaterialTheme.colorScheme.surfaceContainerHighest
        },
        shape = MaterialTheme.shapes.medium,
    ) {
        Column(
            modifier = Modifier
                .fillMaxWidth()
                .padding(14.dp),
            verticalArrangement = Arrangement.spacedBy(8.dp),
        ) {
            Row(
                modifier = Modifier.fillMaxWidth(),
                horizontalArrangement = Arrangement.SpaceBetween,
                verticalAlignment = Alignment.CenterVertically,
            ) {
                Text(
                    text = if (choice.isRecommended) "推荐路径" else "候选路径",
                    style = MaterialTheme.typography.labelMedium,
                    color = MaterialTheme.colorScheme.onSurfaceVariant,
                )
                Text(
                    text = "长度 ${choice.weight}",
                    style = MaterialTheme.typography.labelMedium,
                    color = MaterialTheme.colorScheme.onSurfaceVariant,
                )
            }
            Text(
                text = annotatedContent,
                maxLines = 5,
                overflow = TextOverflow.Ellipsis,
                style = MaterialTheme.typography.bodyMedium.copy(
                    fontFamily = LocalNoteFontFamily.current,
                    fontWeight = LocalNoteFontWeight.current,
                ),
            )
        }
    }
}

@Composable
private fun BranchChoiceSheetCard(
    choice: ThreadBranchChoice,
    onClick: () -> Unit,
) {
    Surface(
        modifier = Modifier
            .fillMaxWidth()
            .clip(MaterialTheme.shapes.large)
            .clickable(onClick = onClick),
        color = if (choice.isRecommended) {
            MaterialTheme.colorScheme.secondaryContainer
        } else {
            MaterialTheme.colorScheme.surfaceContainer
        },
        shape = MaterialTheme.shapes.large,
    ) {
        Column(
            modifier = Modifier
                .fillMaxWidth()
                .padding(16.dp),
            verticalArrangement = Arrangement.spacedBy(8.dp),
        ) {
            Row(
                verticalAlignment = Alignment.CenterVertically,
                horizontalArrangement = Arrangement.spacedBy(8.dp),
            ) {
                if (choice.isRecommended) {
                    Surface(
                        color = MaterialTheme.colorScheme.primaryContainer,
                        shape = RoundedCornerShape(999.dp),
                    ) {
                        Text(
                            text = "推荐",
                            style = MaterialTheme.typography.labelMedium,
                            color = MaterialTheme.colorScheme.onPrimaryContainer,
                            modifier = Modifier.padding(horizontal = 10.dp, vertical = 4.dp),
                        )
                    }
                }
                Text(
                    text = "可继续 ${choice.weight} 步",
                    style = MaterialTheme.typography.labelMedium,
                    color = MaterialTheme.colorScheme.onSurfaceVariant,
                )
            }
            Text(
                text = choice.note.content,
                style = MaterialTheme.typography.bodyLarge,
                maxLines = 6,
                overflow = TextOverflow.Ellipsis,
            )
        }
    }
}

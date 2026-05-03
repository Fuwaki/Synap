package com.synap.app.ui.components

import androidx.compose.animation.core.animateFloatAsState
import androidx.compose.animation.core.Animatable
import androidx.compose.animation.core.spring
import androidx.compose.foundation.Canvas
import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.gestures.detectTransformGestures
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.offset
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Surface
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.rememberCoroutineScope
import androidx.compose.runtime.rememberUpdatedState
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.geometry.Offset
import androidx.compose.ui.geometry.Size
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.Path
import androidx.compose.ui.graphics.PathEffect
import androidx.compose.ui.graphics.TransformOrigin
import androidx.compose.ui.graphics.drawscope.Stroke
import androidx.compose.ui.graphics.graphicsLayer
import androidx.compose.ui.input.pointer.pointerInput
import androidx.compose.ui.layout.onSizeChanged
import androidx.compose.ui.platform.LocalDensity
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.IntOffset
import androidx.compose.ui.unit.IntSize
import androidx.compose.ui.unit.dp
import com.synap.app.ui.model.ThreadGraph
import com.synap.app.ui.model.ThreadGraphEdge
import com.synap.app.ui.model.ThreadGraphNode
import kotlinx.coroutines.launch
import kotlin.math.absoluteValue
import kotlin.math.max
import kotlin.math.min
import kotlin.math.roundToInt

@Composable
fun ThreadDagCanvas(
    graph: ThreadGraph,
    onNodeClick: (ThreadGraphNode) -> Unit,
    modifier: Modifier = Modifier,
    orientation: DagCanvasOrientation = DagCanvasOrientation.Horizontal,
) {
    val density = LocalDensity.current
    val nodeWidthDp = 132.dp
    val nodeHeightDp = 44.dp
    val nodeWidthPx = with(density) { nodeWidthDp.toPx() }
    val nodeHeightPx = with(density) { nodeHeightDp.toPx() }
    val horizontalGapPx = with(density) { 72.dp.toPx() }
    val verticalGapPx = with(density) { 12.dp.toPx() }
    val contentPaddingPx = with(density) { 64.dp.toPx() }
    val minScale = 0.55f
    val maxScale = 2.8f
    val canvasBackground = MaterialTheme.colorScheme.surfaceContainerLow
    val gridColor = MaterialTheme.colorScheme.outlineVariant.copy(alpha = 0.18f)
    val primaryEdgeColor = MaterialTheme.colorScheme.primary
    val secondaryEdgeColor = MaterialTheme.colorScheme.tertiary
    val outlineEdgeColor = MaterialTheme.colorScheme.outlineVariant
    val scope = rememberCoroutineScope()

    val layout = remember(
        graph,
        orientation,
        nodeWidthPx,
        nodeHeightPx,
        horizontalGapPx,
        verticalGapPx,
        contentPaddingPx,
    ) {
        buildDagCanvasLayout(
            graph = graph,
            nodeWidth = nodeWidthPx,
            nodeHeight = nodeHeightPx,
            horizontalGap = horizontalGapPx,
            verticalGap = verticalGapPx,
            contentPadding = contentPaddingPx,
            orientation = orientation,
        )
    }

    var viewportSize by remember(graph) { mutableStateOf(IntSize.Zero) }
    var scale by remember(graph) { mutableStateOf(1f) }
    var translation by remember(graph) { mutableStateOf(Offset.Zero) }
    val settleTranslationX = remember(graph) { Animatable(0f) }
    val settleTranslationY = remember(graph) { Animatable(0f) }
    val latestViewportSize by rememberUpdatedState(viewportSize)
    val latestLayout by rememberUpdatedState(layout)
    val latestScale by rememberUpdatedState(scale)
    val latestTranslation by rememberUpdatedState(translation)

    fun applyFitToViewport() {
        if (viewportSize.width == 0 || viewportSize.height == 0) return
        val fitted = fitDagToViewport(
            contentWidth = layout.contentWidth,
            contentHeight = layout.contentHeight,
            viewportWidth = viewportSize.width.toFloat(),
            viewportHeight = viewportSize.height.toFloat(),
            minScale = minScale,
            maxScale = maxScale,
        )
        scale = fitted.scale
        translation = fitted.translation
        scope.launch {
            settleTranslationX.snapTo(fitted.translation.x)
            settleTranslationY.snapTo(fitted.translation.y)
        }
    }

    LaunchedEffect(graph, viewportSize) {
        applyFitToViewport()
    }

    Box(
        modifier = modifier
            .clip(RoundedCornerShape(30.dp))
            .background(canvasBackground)
            .onSizeChanged { viewportSize = it }
            .pointerInput(layout) {
                detectTransformGestures { centroid, pan, zoom, _ ->
                    val viewport = latestViewportSize
                    if (viewport.width == 0 || viewport.height == 0) return@detectTransformGestures

                    val oldScale = latestScale
                    val oldTranslation = latestTranslation
                    val newScale = (oldScale * zoom).coerceIn(minScale, maxScale)
                    val contentPoint = (centroid - oldTranslation) / oldScale
                    val rawTranslation = centroid - (contentPoint * newScale) + pan
                    val clampedTranslation = clampTranslation(
                        translation = rawTranslation,
                        scale = newScale,
                        contentWidth = latestLayout.contentWidth,
                        contentHeight = latestLayout.contentHeight,
                        viewportWidth = viewport.width.toFloat(),
                        viewportHeight = viewport.height.toFloat(),
                    )

                    scale = newScale
                    translation = clampedTranslation
                    scope.launch {
                        settleTranslationX.snapTo(clampedTranslation.x)
                        settleTranslationY.snapTo(clampedTranslation.y)
                    }
                }
            },
    ) {
        Canvas(modifier = Modifier.fillMaxSize()) {
            drawMindMapBackground(
                scale = scale,
                translation = translation,
                viewport = size,
                gridColor = gridColor,
            )
        }

        if (layout.nodes.isNotEmpty()) {
            Box(
                modifier = Modifier
                    .width(with(density) { layout.contentWidth.toDp() })
                    .height(with(density) { layout.contentHeight.toDp() })
                    .graphicsLayer {
                        translationX = translation.x
                        translationY = translation.y
                        scaleX = scale
                        scaleY = scale
                        transformOrigin = TransformOrigin(0f, 0f)
                    },
            ) {
                Canvas(modifier = Modifier.fillMaxSize()) {
                    val nodeMap = layout.nodes.associateBy { it.node.id }

                    layout.edges.forEach { edge ->
                        val from = nodeMap[edge.fromId] ?: return@forEach
                        val to = nodeMap[edge.toId] ?: return@forEach
                        val edgePath = buildEdgePath(
                            from = from,
                            to = to,
                            orientation = orientation,
                        )

                        drawPath(
                            path = edgePath.path,
                            color = if (edge.isPrimary) {
                                primaryEdgeColor.copy(alpha = 0.72f)
                            } else {
                                secondaryEdgeColor.copy(alpha = 0.44f)
                            },
                            style = Stroke(
                                width = if (edge.isPrimary) 5f else 2.4f,
                                pathEffect = if (edge.isPrimary) null else PathEffect.dashPathEffect(floatArrayOf(14f, 10f)),
                            ),
                        )

                        drawCircle(
                            color = if (edge.isPrimary) primaryEdgeColor else outlineEdgeColor,
                            radius = if (edge.isPrimary) 4.6f else 3.2f,
                            center = edgePath.end,
                        )
                    }
                }

                layout.nodes.forEach { nodeLayout ->
                    DagNodeCard(
                        node = nodeLayout.node,
                        modifier = Modifier.offset {
                            IntOffset(
                                x = nodeLayout.x.roundToInt(),
                                y = nodeLayout.y.roundToInt(),
                            )
                        },
                        onClick = { onNodeClick(nodeLayout.node) },
                    )
                }
            }
        }

        Surface(
            modifier = Modifier
                .align(Alignment.TopEnd)
                .padding(10.dp),
            shape = RoundedCornerShape(999.dp),
            color = MaterialTheme.colorScheme.surfaceContainerHighest.copy(alpha = 0.78f),
            tonalElevation = 2.dp,
        ) {
            Text(
                text = "适配",
                style = MaterialTheme.typography.labelMedium,
                color = MaterialTheme.colorScheme.onSurfaceVariant,
                modifier = Modifier
                    .clickable {
                        val viewport = latestViewportSize
                        if (viewport.width == 0 || viewport.height == 0) return@clickable
                        val fitted = fitDagToViewport(
                            contentWidth = latestLayout.contentWidth,
                            contentHeight = latestLayout.contentHeight,
                            viewportWidth = viewport.width.toFloat(),
                            viewportHeight = viewport.height.toFloat(),
                            minScale = minScale,
                            maxScale = maxScale,
                        )
                        scale = fitted.scale
                        scope.launch {
                            settleTranslationX.stop()
                            settleTranslationY.stop()
                            settleTranslationX.snapTo(translation.x)
                            settleTranslationY.snapTo(translation.y)
                            launch {
                                settleTranslationX.animateTo(
                                    targetValue = fitted.translation.x,
                                    animationSpec = spring(
                                        dampingRatio = 0.92f,
                                        stiffness = 420f,
                                    ),
                                ) {
                                    translation = Offset(value, translation.y)
                                }
                            }
                            launch {
                                settleTranslationY.animateTo(
                                    targetValue = fitted.translation.y,
                                    animationSpec = spring(
                                        dampingRatio = 0.92f,
                                        stiffness = 420f,
                                    ),
                                ) {
                                    translation = Offset(translation.x, value)
                                }
                            }
                        }
                    }
                    .padding(horizontal = 12.dp, vertical = 8.dp),
            )
        }
    }
}

enum class DagCanvasOrientation {
    Horizontal,
    Vertical,
}

@Composable
private fun DagNodeCard(
    node: ThreadGraphNode,
    modifier: Modifier = Modifier,
    onClick: () -> Unit,
) {
    val animatedScale by animateFloatAsState(
        targetValue = if (node.isFocused) 1.035f else 1f,
        label = "dag_node_scale",
    )
    val title = remember(node.note.content) {
        node.note.content
            .lineSequence()
            .map(String::trim)
            .firstOrNull(String::isNotEmpty)
            ?.take(28)
            ?.ifBlank { "空笔记" }
            ?: "空笔记"
    }

    Surface(
        modifier = modifier
            .graphicsLayer {
                scaleX = animatedScale
                scaleY = animatedScale
                transformOrigin = TransformOrigin(0.5f, 0.5f)
            }
            .width(132.dp)
            .height(44.dp)
            .clickable(onClick = onClick),
        shape = RoundedCornerShape(999.dp),
        color = when {
            node.isFocused -> MaterialTheme.colorScheme.primaryContainer.copy(alpha = 0.96f)
            node.isInPrimarySegment -> MaterialTheme.colorScheme.secondaryContainer.copy(alpha = 0.94f)
            node.isBranchChoice -> MaterialTheme.colorScheme.surfaceContainerHigh
            else -> MaterialTheme.colorScheme.surfaceContainerLow
        },
        tonalElevation = if (node.isFocused) 4.dp else 1.dp,
        shadowElevation = if (node.isFocused) 5.dp else 0.dp,
    ) {
        Row(
            modifier = Modifier
                .fillMaxSize()
                .padding(horizontal = 10.dp, vertical = 7.dp),
            horizontalArrangement = Arrangement.spacedBy(8.dp),
            verticalAlignment = Alignment.CenterVertically,
        ) {
            Box(
                modifier = Modifier
                    .width(10.dp)
                    .height(10.dp)
                    .clip(RoundedCornerShape(999.dp))
                    .background(
                        when {
                            node.isFocused -> MaterialTheme.colorScheme.primary
                            node.isInPrimarySegment -> MaterialTheme.colorScheme.secondary
                            else -> MaterialTheme.colorScheme.outline
                        },
                    ),
            )
            Column(
                modifier = Modifier.weight(1f),
                verticalArrangement = Arrangement.Center,
            ) {
                Text(
                    text = title,
                    style = MaterialTheme.typography.labelLarge,
                    fontWeight = if (node.isFocused) FontWeight.SemiBold else FontWeight.Medium,
                    maxLines = 1,
                    overflow = TextOverflow.Ellipsis,
                )
                if (!node.isInPrimarySegment) {
                    Text(
                        text = if (node.depth < 0) "上游分支" else "下游分支",
                        style = MaterialTheme.typography.labelSmall,
                        color = MaterialTheme.colorScheme.onSurfaceVariant,
                        maxLines = 1,
                        overflow = TextOverflow.Clip,
                    )
                }
            }
        }
    }
}

private data class DagCanvasNodeLayout(
    val node: ThreadGraphNode,
    val x: Float,
    val y: Float,
    val width: Float,
    val height: Float,
)

private data class DagCanvasLayout(
    val nodes: List<DagCanvasNodeLayout>,
    val edges: List<ThreadGraphEdge>,
    val contentWidth: Float,
    val contentHeight: Float,
)

private data class DagCanvasEdgePath(
    val path: Path,
    val end: Offset,
)

private data class FitTransform(
    val scale: Float,
    val translation: Offset,
)

private fun buildDagCanvasLayout(
    graph: ThreadGraph,
    nodeWidth: Float,
    nodeHeight: Float,
    horizontalGap: Float,
    verticalGap: Float,
    contentPadding: Float,
    orientation: DagCanvasOrientation,
): DagCanvasLayout {
    if (graph.nodes.isEmpty()) {
        return DagCanvasLayout(
            nodes = emptyList(),
            edges = emptyList(),
            contentWidth = nodeWidth + contentPadding * 2,
            contentHeight = nodeHeight + contentPadding * 2,
        )
    }

    val sortedDepths = graph.nodes.map { it.depth }.distinct().sorted()
    val depthToColumn = sortedDepths.withIndex().associate { (index, depth) -> depth to index }
    val nodeById = graph.nodes.associateBy(ThreadGraphNode::id)
    val incoming = mutableMapOf<String, MutableList<String>>()
    val outgoing = mutableMapOf<String, MutableList<String>>()

    graph.nodes.forEach { node ->
        incoming[node.id] = mutableListOf()
        outgoing[node.id] = mutableListOf()
    }

    graph.edges.forEach { edge ->
        if (edge.fromId in nodeById && edge.toId in nodeById) {
            outgoing.getValue(edge.fromId).add(edge.toId)
            incoming.getValue(edge.toId).add(edge.fromId)
        }
    }

    val initialComparator = compareByDescending<ThreadGraphNode> { it.isInPrimarySegment }
        .thenByDescending { it.isFocused }
        .thenBy { it.id }

    val columns = sortedDepths.map { depth ->
        graph.nodes
            .filter { it.depth == depth }
            .sortedWith(initialComparator)
            .toMutableList()
    }.toMutableList()

    repeat(6) {
        var orderMap = buildColumnOrderMap(columns)
        for (columnIndex in 1 until columns.size) {
            columns[columnIndex].sortWith(
                compareBy<ThreadGraphNode> {
                    barycenter(
                        ids = incoming.getValue(it.id)
                            .filter { parentId -> depthToColumn.getValue(nodeById.getValue(parentId).depth) < columnIndex },
                        orderMap = orderMap,
                    ) ?: orderMap.getValue(it.id)
                }.thenByDescending { it.isInPrimarySegment }
                    .thenBy { it.id },
            )
        }

        orderMap = buildColumnOrderMap(columns)
        for (columnIndex in columns.lastIndex - 1 downTo 0) {
            columns[columnIndex].sortWith(
                compareBy<ThreadGraphNode> {
                    barycenter(
                        ids = outgoing.getValue(it.id)
                            .filter { childId -> depthToColumn.getValue(nodeById.getValue(childId).depth) > columnIndex },
                        orderMap = orderMap,
                    ) ?: orderMap.getValue(it.id)
                }.thenByDescending { it.isInPrimarySegment }
                    .thenBy { it.id },
            )
        }
    }

    val rawPositions = mutableMapOf<String, Pair<Int, Float>>()
    columns.forEachIndexed { columnIndex, nodes ->
        val anchorIndex = nodes.indexOfFirst { it.isInPrimarySegment }
            .takeIf { it >= 0 }
            ?.toFloat()
            ?: ((nodes.size - 1) / 2f)

        nodes.forEachIndexed { orderIndex, node ->
            val crossAxisGap = when (orientation) {
                DagCanvasOrientation.Horizontal -> nodeHeight + verticalGap
                DagCanvasOrientation.Vertical -> nodeWidth + horizontalGap
            }
            rawPositions[node.id] = columnIndex to ((orderIndex - anchorIndex) * crossAxisGap)
        }
    }

    val minCrossAxis = rawPositions.values.minOf { it.second }
    val maxCrossAxis = rawPositions.values.maxOf { it.second }

    val positionedNodes = columns.flatMapIndexed { columnIndex, nodes ->
        nodes.map { node ->
            val rawCrossAxis = rawPositions.getValue(node.id).second
            when (orientation) {
                DagCanvasOrientation.Horizontal -> {
                    DagCanvasNodeLayout(
                        node = node,
                        x = contentPadding + columnIndex * (nodeWidth + horizontalGap),
                        y = contentPadding + (rawCrossAxis - minCrossAxis),
                        width = nodeWidth,
                        height = nodeHeight,
                    )
                }

                DagCanvasOrientation.Vertical -> {
                    DagCanvasNodeLayout(
                        node = node,
                        x = contentPadding + (rawCrossAxis - minCrossAxis),
                        y = contentPadding + columnIndex * (nodeHeight + verticalGap * 2.2f),
                        width = nodeWidth,
                        height = nodeHeight,
                    )
                }
            }
        }
    }

    val contentWidth = when (orientation) {
        DagCanvasOrientation.Horizontal -> contentPadding * 2 +
            columns.size * nodeWidth +
            max(0, columns.size - 1) * horizontalGap
        DagCanvasOrientation.Vertical -> contentPadding * 2 + (maxCrossAxis - minCrossAxis) + nodeWidth
    }
    val contentHeight = when (orientation) {
        DagCanvasOrientation.Horizontal -> contentPadding * 2 + (maxCrossAxis - minCrossAxis) + nodeHeight
        DagCanvasOrientation.Vertical -> contentPadding * 2 +
            columns.size * nodeHeight +
            max(0, columns.size - 1) * verticalGap * 2.2f
    }

    return DagCanvasLayout(
        nodes = positionedNodes,
        edges = graph.edges,
        contentWidth = contentWidth,
        contentHeight = contentHeight,
    )
}

private fun buildEdgePath(
    from: DagCanvasNodeLayout,
    to: DagCanvasNodeLayout,
    orientation: DagCanvasOrientation,
): DagCanvasEdgePath {
    return when (orientation) {
        DagCanvasOrientation.Horizontal -> {
            val start = Offset(from.x + from.width, from.y + from.height * 0.5f)
            val end = Offset(to.x, to.y + to.height * 0.5f)
            val ctrl = max((end.x - start.x).absoluteValue * 0.42f, from.width * 0.22f)
            DagCanvasEdgePath(
                path = Path().apply {
                    moveTo(start.x, start.y)
                    cubicTo(start.x + ctrl, start.y, end.x - ctrl, end.y, end.x, end.y)
                },
                end = end,
            )
        }

        DagCanvasOrientation.Vertical -> {
            val start = Offset(from.x + from.width * 0.5f, from.y + from.height)
            val end = Offset(to.x + to.width * 0.5f, to.y)
            val ctrl = max((end.y - start.y).absoluteValue * 0.42f, from.height * 0.5f)
            DagCanvasEdgePath(
                path = Path().apply {
                    moveTo(start.x, start.y)
                    cubicTo(start.x, start.y + ctrl, end.x, end.y - ctrl, end.x, end.y)
                },
                end = end,
            )
        }
    }
}

private fun buildColumnOrderMap(columns: List<List<ThreadGraphNode>>): Map<String, Float> =
    buildMap {
        columns.forEach { column ->
            column.forEachIndexed { index, node ->
                put(node.id, index.toFloat())
            }
        }
    }

private fun barycenter(
    ids: List<String>,
    orderMap: Map<String, Float>,
): Float? {
    val positions = ids.mapNotNull(orderMap::get)
    if (positions.isEmpty()) return null
    return positions.sum() / positions.size.toFloat()
}

private fun fitDagToViewport(
    contentWidth: Float,
    contentHeight: Float,
    viewportWidth: Float,
    viewportHeight: Float,
    minScale: Float,
    maxScale: Float,
): FitTransform {
    val horizontalInset = 40f
    val verticalInset = 36f
    val fitScale = min(
        (viewportWidth - horizontalInset * 2) / contentWidth,
        (viewportHeight - verticalInset * 2) / contentHeight,
    ).coerceIn(minScale, maxScale)

    val translation = Offset(
        x = (viewportWidth - contentWidth * fitScale) * 0.5f,
        y = (viewportHeight - contentHeight * fitScale) * 0.5f,
    )

    return FitTransform(scale = fitScale, translation = translation)
}

private fun clampTranslation(
    translation: Offset,
    scale: Float,
    contentWidth: Float,
    contentHeight: Float,
    viewportWidth: Float,
    viewportHeight: Float,
): Offset {
    val margin = 120f
    val scaledWidth = contentWidth * scale
    val scaledHeight = contentHeight * scale

    val x = if (scaledWidth + margin * 2 <= viewportWidth) {
        (viewportWidth - scaledWidth) * 0.5f
    } else {
        translation.x.coerceIn(viewportWidth - scaledWidth - margin, margin)
    }

    val y = if (scaledHeight + margin * 2 <= viewportHeight) {
        (viewportHeight - scaledHeight) * 0.5f
    } else {
        translation.y.coerceIn(viewportHeight - scaledHeight - margin, margin)
    }

    return Offset(x, y)
}

private fun androidx.compose.ui.graphics.drawscope.DrawScope.drawMindMapBackground(
    scale: Float,
    translation: Offset,
    viewport: Size,
    gridColor: Color,
) {
    val spacing = max(42f, min(94f, 68f * scale))
    val startX = ((translation.x % spacing) + spacing) % spacing
    val startY = ((translation.y % spacing) + spacing) % spacing

    var x = startX
    while (x <= viewport.width) {
        drawLine(
            color = gridColor,
            start = Offset(x, 0f),
            end = Offset(x, viewport.height),
            strokeWidth = 1f,
        )
        x += spacing
    }

    var y = startY
    while (y <= viewport.height) {
        drawLine(
            color = gridColor,
            start = Offset(0f, y),
            end = Offset(viewport.width, y),
            strokeWidth = 1f,
        )
        y += spacing
    }

    val dotSpacing = spacing * 0.5f
    var dotX = ((translation.x % dotSpacing) + dotSpacing) % dotSpacing
    while (dotX <= viewport.width) {
        var dotY = ((translation.y % dotSpacing) + dotSpacing) % dotSpacing
        while (dotY <= viewport.height) {
            drawCircle(
                color = gridColor.copy(alpha = 0.38f),
                radius = 1.2f,
                center = Offset(dotX, dotY),
            )
            dotY += dotSpacing
        }
        dotX += dotSpacing
    }
}

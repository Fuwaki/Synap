package com.synap.app.ui.screens

import androidx.compose.animation.AnimatedVisibility
import androidx.compose.animation.fadeIn
import androidx.compose.animation.fadeOut
import androidx.compose.animation.slideInVertically
import androidx.compose.animation.slideOutVertically
import androidx.compose.foundation.clickable
import androidx.compose.foundation.horizontalScroll
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.ExperimentalLayoutApi
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.WindowInsets
import androidx.compose.foundation.layout.consumeWindowInsets
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.imePadding
import androidx.compose.foundation.layout.isImeVisible
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.foundation.text.KeyboardActions
import androidx.compose.foundation.text.KeyboardOptions
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.Add
import androidx.compose.material.icons.filled.ArrowBack
import androidx.compose.material.icons.filled.Check
import androidx.compose.material.icons.filled.CheckBox
import androidx.compose.material.icons.filled.Close
import androidx.compose.material.icons.filled.FormatBold
import androidx.compose.material.icons.filled.FormatColorText
import androidx.compose.material.icons.filled.FormatItalic
import androidx.compose.material.icons.filled.FormatListBulleted
import androidx.compose.material.icons.filled.FormatQuote
import androidx.compose.material.icons.filled.FormatStrikethrough
import androidx.compose.material.icons.filled.FormatUnderlined
import androidx.compose.material.icons.filled.Reply
import androidx.compose.material.icons.filled.Title
import androidx.compose.material3.CircularProgressIndicator
import androidx.compose.material3.ExperimentalMaterial3Api
import androidx.compose.material3.HorizontalDivider
import androidx.compose.material3.Icon
import androidx.compose.material3.IconButton
import androidx.compose.material3.IconButtonDefaults
import androidx.compose.material3.InputChip
import androidx.compose.material3.InputChipDefaults
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.OutlinedTextField
import androidx.compose.material3.Scaffold
import androidx.compose.material3.Surface
import androidx.compose.material3.Text
import androidx.compose.material3.TextButton
import androidx.compose.material3.TextField
import androidx.compose.material3.TextFieldDefaults
import androidx.compose.material3.TopAppBar
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.focus.FocusRequester
import androidx.compose.ui.focus.focusRequester
import androidx.compose.ui.focus.onFocusChanged
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.vector.ImageVector
import androidx.compose.ui.text.AnnotatedString
import androidx.compose.ui.text.ParagraphStyle
import androidx.compose.ui.text.SpanStyle
import androidx.compose.ui.text.TextRange
import androidx.compose.ui.text.buildAnnotatedString
import androidx.compose.ui.text.font.FontStyle
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.input.ImeAction
import androidx.compose.ui.text.input.OffsetMapping
import androidx.compose.ui.text.input.TextFieldValue
import androidx.compose.ui.text.input.TransformedText
import androidx.compose.ui.text.input.VisualTransformation
import androidx.compose.ui.text.style.TextDecoration
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import androidx.compose.ui.res.stringResource
import com.synap.app.LocalNoteFontFamily
import com.synap.app.LocalNoteFontWeight
import com.synap.app.LocalNoteTextSize
import com.synap.app.LocalNoteLineSpacing
import com.synap.app.R
import com.synap.app.ui.viewmodel.EditorMode
import com.synap.app.ui.viewmodel.EditorUiState
import kotlinx.coroutines.delay

// ==================== 工具：获取隐藏符号的精确区间，用于退格保护 ====================
fun getMarkdownTokenRanges(text: String): List<IntRange> {
    val ranges = mutableListOf<IntRange>()
    fun addTokens(regex: Regex) {
        regex.findAll(text).forEach { match ->
            if (match.groups.size >= 2) {
                val content = match.groups[1]!!
                ranges.add(match.range.first until content.range.first)
                ranges.add(content.range.last + 1..match.range.last)
            }
        }
    }
    addTokens(Regex("\\*\\*\\*(.*?)\\*\\*\\*"))
    addTokens(Regex("(?<!\\*)\\*\\*(?!\\*)(.*?)(?<!\\*)\\*\\*(?!\\*)"))
    addTokens(Regex("(?<!\\*)\\*(?!\\*)(.*?)(?<!\\*)\\*(?!\\*)"))
    addTokens(Regex("~~(.*?)~~"))
    addTokens(Regex("<u>(.*?)</u>"))
    addTokens(Regex("==(.*?)=="))

    Regex("^(#{1,4} |> |-\\s+\\[[ x]\\]\\s+|-\\s+|\\d+\\.\\s+)", RegexOption.MULTILINE).findAll(text).forEach {
        ranges.add(it.range)
    }
    return ranges
}

// ==================== 自定义 Markdown 视觉渲染引擎 (隐藏语法符) ====================
class MarkdownVisualTransformation(
    private val primaryColor: Color,
    private val highlightColor: Color,
    private val baseFontSize: Float
) : VisualTransformation {
    override fun filter(text: AnnotatedString): TransformedText {
        val charArray = text.text.toCharArray()
        Regex("^(> )", RegexOption.MULTILINE).findAll(text.text).forEach { charArray[it.range.first] = '“' }
        Regex("^-\\s+\\[ \\]\\s", RegexOption.MULTILINE).findAll(text.text).forEach { match ->
            charArray[match.range.first] = '☐'
            for(i in match.range.first + 1 .. match.range.last) charArray[i] = ' '
        }
        Regex("^-\\s+\\[x\\]\\s", RegexOption.MULTILINE).findAll(text.text).forEach { match ->
            charArray[match.range.first] = '☑'
            for(i in match.range.first + 1 .. match.range.last) charArray[i] = ' '
        }
        Regex("^- (?!(\\[ \\]|\\[x\\]))", RegexOption.MULTILINE).findAll(text.text).forEach { charArray[it.range.first] = '•' }
        val visualString = String(charArray)

        val annotatedString = buildAnnotatedString {
            append(visualString)
            val hiddenSpanStyle = SpanStyle(color = Color.Transparent, fontSize = 0.1.sp)

            fun processMatches(regex: Regex, style: SpanStyle) {
                regex.findAll(visualString).forEach { match ->
                    if (match.groups.size >= 2) {
                        val content = match.groups[1]!!
                        addStyle(style, content.range.first, content.range.last + 1)
                        addStyle(hiddenSpanStyle, match.range.first, content.range.first)
                        addStyle(hiddenSpanStyle, content.range.last + 1, match.range.last + 1)
                    }
                }
            }

            processMatches(Regex("\\*\\*\\*(.*?)\\*\\*\\*"), SpanStyle(fontWeight = FontWeight.Bold, fontStyle = FontStyle.Italic))
            processMatches(Regex("(?<!\\*)\\*\\*(?!\\*)(.*?)(?<!\\*)\\*\\*(?!\\*)"), SpanStyle(fontWeight = FontWeight.Bold))
            processMatches(Regex("(?<!\\*)\\*(?!\\*)(.*?)(?<!\\*)\\*(?!\\*)"), SpanStyle(fontStyle = FontStyle.Italic))
            processMatches(Regex("~~(.*?)~~"), SpanStyle(textDecoration = TextDecoration.LineThrough))
            processMatches(Regex("<u>(.*?)</u>"), SpanStyle(textDecoration = TextDecoration.Underline))
            processMatches(Regex("==(.*?)=="), SpanStyle(background = highlightColor, color = Color.Black))

            Regex("^(#{1,4} )(.*)", RegexOption.MULTILINE).findAll(visualString).forEach { match ->
                if (match.groups.size >= 3) {
                    val level = match.groups[1]!!.value.trim().length
                    val scale = 1.8f - (level * 0.15f)
                    addStyle(hiddenSpanStyle, match.groups[1]!!.range.first, match.groups[1]!!.range.last + 1)
                    val lineEnd = visualString.indexOf('\n', match.range.last).takeIf { it != -1 } ?: visualString.length
                    addStyle(SpanStyle(fontWeight = FontWeight.ExtraBold, fontSize = (baseFontSize * scale).sp, color = primaryColor), match.groups[2]!!.range.first, match.groups[2]!!.range.last + 1)
                    addStyle(ParagraphStyle(lineHeight = (baseFontSize * 1.5f).sp), match.range.first, match.range.last + 1)
                }
            }

            val lines = visualString.split('\n')
            var offset = 0
            var inQuote = false
            var quoteStart = 0

            for (i in lines.indices) {
                val line = lines[i]
                val lineLength = line.length

                if (line.startsWith("“ ")) {
                    if (!inQuote) {
                        inQuote = true
                        quoteStart = offset
                        addStyle(SpanStyle(color = Color.Gray, fontSize = (baseFontSize * 1.5f).sp, fontWeight = FontWeight.Black), offset, offset + 1)
                        addStyle(hiddenSpanStyle, offset + 1, offset + 2)
                    } else {
                        addStyle(hiddenSpanStyle, offset, offset + 2)
                    }
                    addStyle(SpanStyle(color = Color.Gray), offset + 2, offset + lineLength)
                } else {
                    if (inQuote) {
                        inQuote = false
                        addStyle(ParagraphStyle(lineHeight = (baseFontSize * 1.5f).sp), quoteStart, offset)
                    }
                }
                offset += lineLength + 1
            }
            if (inQuote) {
                addStyle(ParagraphStyle(lineHeight = (baseFontSize * 1.5f).sp), quoteStart, offset - 1)
            }

            Regex("^☐(     )", RegexOption.MULTILINE).findAll(visualString).forEach { match ->
                addStyle(SpanStyle(color = primaryColor, fontSize = (baseFontSize * 1.3f).sp), match.range.first, match.range.first + 1)
                addStyle(hiddenSpanStyle, match.groups[1]!!.range.first, match.groups[1]!!.range.last + 1)
            }
            Regex("^☑(     )", RegexOption.MULTILINE).findAll(visualString).forEach { match ->
                addStyle(SpanStyle(color = primaryColor, fontSize = (baseFontSize * 1.3f).sp), match.range.first, match.range.first + 1)
                addStyle(hiddenSpanStyle, match.groups[1]!!.range.first, match.groups[1]!!.range.last + 1)
            }
            Regex("^•( )", RegexOption.MULTILINE).findAll(visualString).forEach { match ->
                addStyle(SpanStyle(color = primaryColor, fontWeight = FontWeight.Bold), match.range.first, match.range.first + 1)
            }
        }
        return TransformedText(annotatedString, OffsetMapping.Identity)
    }
}

enum class EditorSubMenu { NONE, HEADING, QUOTE, LIST }

// ==================== 工具栏按钮组件 (修复上下文报错) ====================
@Composable
fun EditorIconButton(
    isActive: Boolean,
    onClick: () -> Unit,
    icon: ImageVector,
    isEnabled: Boolean = true,
    primaryColor: Color = MaterialTheme.colorScheme.primary
) {
    val colors = IconButtonDefaults.iconButtonColors(
        contentColor = if (!isEnabled) MaterialTheme.colorScheme.onSurfaceVariant.copy(alpha = 0.38f)
        else if (isActive) MaterialTheme.colorScheme.onPrimary
        else MaterialTheme.colorScheme.onSurfaceVariant,
        containerColor = if (isActive && isEnabled) primaryColor else Color.Transparent
    )
    IconButton(onClick = onClick, enabled = isEnabled, colors = colors) {
        Icon(icon, contentDescription = null)
    }
}

@OptIn(ExperimentalMaterial3Api::class, ExperimentalLayoutApi::class)
@Composable
fun NewNoteScreen(
    uiState: EditorUiState,
    onNavigateBack: () -> Unit,
    onContentChange: (String) -> Unit,
    onAddTag: (String) -> Unit,
    onRemoveTag: (Int) -> Unit,
    onSave: () -> Unit,
) {
    var tagInputText by remember { mutableStateOf("") }
    var isTagInputVisible by remember { mutableStateOf(false) }
    var tagInputHasFocus by remember { mutableStateOf(false) }

    val bodyFocusRequester = remember { FocusRequester() }
    val tagFocusRequester = remember { FocusRequester() }
    val isImeVisible = WindowInsets.isImeVisible

    var textFieldValue by remember { mutableStateOf(TextFieldValue(uiState.content)) }
    var activeSubMenu by remember { mutableStateOf(EditorSubMenu.NONE) }

    val primaryColor = MaterialTheme.colorScheme.primary
    val highlightColor = MaterialTheme.colorScheme.tertiaryContainer
    val baseFontSize = LocalNoteTextSize.current.value

    LaunchedEffect(uiState.content) {
        if (uiState.content != textFieldValue.text) {
            textFieldValue = textFieldValue.copy(text = uiState.content)
        }
    }
    LaunchedEffect(Unit) {
        delay(300)
        bodyFocusRequester.requestFocus()
        textFieldValue = textFieldValue.copy(selection = TextRange(textFieldValue.text.length))
    }
    LaunchedEffect(isTagInputVisible) {
        if (isTagInputVisible) {
            delay(100)
            tagFocusRequester.requestFocus()
        }
    }

    val currentCursor = textFieldValue.selection.min
    var lineStart = if (currentCursor <= 0) -1 else textFieldValue.text.lastIndexOf('\n', currentCursor - 1)
    lineStart = if (lineStart == -1) 0 else lineStart + 1
    var lineEnd = textFieldValue.text.indexOf('\n', lineStart)
    if (lineEnd == -1) lineEnd = textFieldValue.text.length
    val currentLine = textFieldValue.text.substring(lineStart, lineEnd)

    val activeHeading = Regex("^(#{1,4}) ").find(currentLine)?.groups?.get(1)?.value?.length ?: 0
    val isQuoteActive = currentLine.startsWith("> ")
    val isUnorderedListActive = currentLine.startsWith("- ") && !currentLine.startsWith("- [")
    val isOrderedListActive = Regex("^\\d+\\.\\s+").find(currentLine) != null
    val isCheckboxEmpty = currentLine.startsWith("- [ ] ")
    val isCheckboxChecked = currentLine.startsWith("- [x] ")
    val isAnyCheckboxActive = isCheckboxEmpty || isCheckboxChecked
    val isAnyListActive = isUnorderedListActive || isOrderedListActive

    val isBoldActive = Regex("(?<!\\*)\\*\\*(?!\\*)(.*?)(?<!\\*)\\*\\*(?!\\*)").findAll(textFieldValue.text).any { currentCursor in it.range }
    val isItalicActive = Regex("(?<!\\*)\\*(?!\\*)(.*?)(?<!\\*)\\*(?!\\*)").findAll(textFieldValue.text).any { currentCursor in it.range }
    val isStrikeActive = Regex("~~(.*?)~~").findAll(textFieldValue.text).any { currentCursor in it.range }
    val isUnderlineActive = Regex("<u>(.*?)</u>").findAll(textFieldValue.text).any { currentCursor in it.range }
    val isHighlightActive = Regex("==(.*?)==").findAll(textFieldValue.text).any { currentCursor in it.range }

    val isTextSelected = !textFieldValue.selection.collapsed

    val applyWrapStyle: (String, String) -> Unit = { prefix, suffix ->
        val text = textFieldValue.text
        val min = textFieldValue.selection.min
        val max = textFieldValue.selection.max

        val escapedPre = Regex.escape(prefix)
        val escapedSuf = Regex.escape(suffix)
        val match = Regex("$escapedPre(.*?)$escapedSuf").findAll(text).find { min in it.range && max in it.range }

        if (match != null) {
            val innerContent = match.groups[1]!!.value
            val newText = text.substring(0, match.range.first) + innerContent + text.substring(match.range.last + 1)
            val shift = prefix.length
            val newMin = (min - shift).coerceIn(match.range.first, match.range.first + innerContent.length)
            val newMax = (max - shift).coerceIn(match.range.first, match.range.first + innerContent.length)
            textFieldValue = TextFieldValue(newText, TextRange(newMin, newMax))
            onContentChange(newText)
        } else {
            val selectedText = text.substring(min, max)
            val newText = text.substring(0, min) + prefix + selectedText + suffix + text.substring(max)
            val newSelection = if (min == max) TextRange(min + prefix.length) else TextRange(min, max + prefix.length + suffix.length)
            textFieldValue = TextFieldValue(newText, newSelection)
            onContentChange(newText)
        }
    }

    val applyLinePrefix: (String) -> Unit = { prefix ->
        val cleanLine = currentLine.replaceFirst(Regex("^(#{1,4} |> |-\\s+\\[[ x]\\]\\s+|-\\s+|\\d+\\.\\s+)"), "")
        val finalPrefix = if (currentLine.startsWith(prefix)) "" else prefix
        val newText = textFieldValue.text.substring(0, lineStart) + finalPrefix + cleanLine + textFieldValue.text.substring(lineEnd)
        val newCursor = if (currentCursor == lineStart) currentCursor + finalPrefix.length else currentCursor + finalPrefix.length - (currentLine.length - cleanLine.length)

        textFieldValue = TextFieldValue(newText, TextRange(newCursor.coerceIn(0, newText.length)))
        onContentChange(newText)
        activeSubMenu = EditorSubMenu.NONE
    }

    val applyCheckboxToggle = {
        val newText: String
        val newCursor: Int
        if (isCheckboxEmpty) {
            val cleanLine = currentLine.removePrefix("- [ ] ")
            newText = textFieldValue.text.substring(0, lineStart) + "- [x] " + cleanLine + textFieldValue.text.substring(lineEnd)
            newCursor = currentCursor
        } else if (isCheckboxChecked) {
            val cleanLine = currentLine.removePrefix("- [x] ")
            newText = textFieldValue.text.substring(0, lineStart) + cleanLine + textFieldValue.text.substring(lineEnd)
            newCursor = (currentCursor - 6).coerceAtLeast(lineStart)
        } else {
            val cleanLine = currentLine.replaceFirst(Regex("^(#{1,4} |> |-\\s+|\\d+\\.\\s+)"), "")
            newText = textFieldValue.text.substring(0, lineStart) + "- [ ] " + cleanLine + textFieldValue.text.substring(lineEnd)
            val diff = 6 - (currentLine.length - cleanLine.length)
            newCursor = if (currentCursor == lineStart) currentCursor + 6 else currentCursor + diff
        }
        textFieldValue = TextFieldValue(newText, TextRange(newCursor.coerceIn(0, newText.length)))
        onContentChange(newText)
    }

    Scaffold(
        topBar = {
            TopAppBar(
                title = {
                    Text(
                        when (uiState.mode) {
                            EditorMode.Create -> stringResource(R.string.edit_title_creat)
                            is EditorMode.Reply -> stringResource(R.string.edit_title_reply)
                            is EditorMode.Edit -> stringResource(R.string.edit_title_edit)
                        }
                    )
                },
                navigationIcon = { IconButton(onClick = onNavigateBack) { Icon(Icons.Filled.ArrowBack, "返回") } },
                actions = {
                    IconButton(onClick = onSave, enabled = !uiState.isSaving && !uiState.isLoading) {
                        if (uiState.isSaving) CircularProgressIndicator(modifier = Modifier.padding(8.dp)) else Icon(Icons.Filled.Check, "保存")
                    }
                },
            )
        },
    ) { innerPadding ->
        Box(modifier = Modifier.fillMaxSize()) {
            Column(
                modifier = Modifier
                    .fillMaxSize()
                    .padding(innerPadding)
                    .consumeWindowInsets(innerPadding)
            ) {
                // ==================== 顶部区域 ====================
                Column(modifier = Modifier.fillMaxWidth().padding(horizontal = 16.dp)) {
                    if (isTagInputVisible) {
                        OutlinedTextField(
                            value = tagInputText,
                            onValueChange = { tagInputText = it },
                            placeholder = { Text("输入标签") },
                            modifier = Modifier
                                .fillMaxWidth()
                                .height(56.dp)
                                .focusRequester(tagFocusRequester)
                                .onFocusChanged { focusState ->
                                    if (focusState.isFocused) tagInputHasFocus = true
                                    else {
                                        if (tagInputHasFocus && tagInputText.isBlank()) isTagInputVisible = false
                                        tagInputHasFocus = false
                                    }
                                },
                            singleLine = true,
                            keyboardOptions = KeyboardOptions(imeAction = ImeAction.Done),
                            keyboardActions = KeyboardActions(onDone = {
                                if (tagInputText.isNotBlank()) {
                                    onAddTag(tagInputText.trim())
                                    tagInputText = ""
                                    isTagInputVisible = false
                                }
                            })
                        )
                    } else {
                        Row(
                            modifier = Modifier.fillMaxWidth().horizontalScroll(rememberScrollState()),
                            horizontalArrangement = Arrangement.spacedBy(8.dp),
                            verticalAlignment = Alignment.CenterVertically,
                        ) {
                            uiState.tags.forEachIndexed { index, tag ->
                                InputChip(
                                    selected = true,
                                    onClick = { },
                                    label = { Text(tag) },
                                    trailingIcon = { Icon(Icons.Filled.Close, "删除", modifier = Modifier.size(InputChipDefaults.AvatarSize).clickable { onRemoveTag(index) }) }
                                )
                            }
                            InputChip(
                                selected = false,
                                onClick = { isTagInputVisible = true },
                                label = { Text("添加标签") },
                                trailingIcon = { Icon(Icons.Filled.Add, null, modifier = Modifier.size(16.dp)) }
                            )
                        }
                    }

                    if (uiState.recommendedTags.isNotEmpty() || uiState.isRecommendingTags) {
                        Row(
                            modifier = Modifier
                                .fillMaxWidth()
                                .padding(top = 4.dp, bottom = 8.dp)
                                .horizontalScroll(rememberScrollState()),
                            verticalAlignment = Alignment.CenterVertically
                        ) {
                            Text("推荐标签：", style = MaterialTheme.typography.labelLarge, color = MaterialTheme.colorScheme.onSurfaceVariant)
                            if (uiState.isRecommendingTags) {
                                CircularProgressIndicator(modifier = Modifier.size(14.dp).padding(start = 4.dp), strokeWidth = 2.dp)
                            } else {
                                uiState.recommendedTags.forEach { tag ->
                                    Text(
                                        text = "#$tag",
                                        style = MaterialTheme.typography.labelLarge,
                                        color = MaterialTheme.colorScheme.primary,
                                        modifier = Modifier.clip(RoundedCornerShape(4.dp)).clickable { onAddTag(tag) }.padding(horizontal = 4.dp, vertical = 2.dp)
                                    )
                                }
                            }
                        }
                    }
                    HorizontalDivider(modifier = Modifier.padding(bottom = 8.dp))
                }

                // ==================== 正文编辑 ====================
                Column(modifier = Modifier.weight(1f).padding(horizontal = 16.dp)) {
                    if (uiState.errorMessage != null) Text(uiState.errorMessage, color = MaterialTheme.colorScheme.error, modifier = Modifier.padding(bottom = 12.dp))

                    if (uiState.isLoading) Box(modifier = Modifier.fillMaxSize(), contentAlignment = Alignment.Center) { CircularProgressIndicator() }
                    else {
                        TextField(
                            value = textFieldValue,
                            onValueChange = { newValue: TextFieldValue ->
                                var finalValue = newValue
                                val oldText = textFieldValue.text
                                val newText = newValue.text
                                val cursor = newValue.selection.min

                                // 拦截1：符号转换墙 (禁止前端乱输符号)
                                if (newText.length > oldText.length) {
                                    val diffLen = newText.length - oldText.length
                                    val addedText = newText.substring(cursor - diffLen, cursor)
                                    val filteredText = addedText.replace("*", "＊").replace("[", "【").replace("]", "】").replace("<", "《").replace(">", "》").replace("|", "｜")
                                    if (addedText != filteredText) {
                                        val correctedText = newText.substring(0, cursor - diffLen) + filteredText + newText.substring(cursor)
                                        finalValue = TextFieldValue(correctedText, newValue.selection)
                                    }
                                }

                                // 拦截2：智能退格保护
                                if (finalValue.text.length < oldText.length && finalValue.selection.collapsed) {
                                    val deletedIndex = finalValue.selection.start
                                    val tokens = getMarkdownTokenRanges(oldText)
                                    val hitToken = tokens.find { deletedIndex in it }

                                    if (hitToken != null) {
                                        val charToDeleteIndex = hitToken.first - 1
                                        if (charToDeleteIndex >= 0) {
                                            val correctedText = oldText.removeRange(charToDeleteIndex, charToDeleteIndex + 1)
                                            finalValue = TextFieldValue(correctedText, TextRange(charToDeleteIndex))
                                        } else finalValue = textFieldValue
                                    }
                                }

                                // 拦截3：回车自动换行前缀延续
                                if (finalValue.text.length == oldText.length + 1 && cursor > 0 && finalValue.text[cursor - 1] == '\n') {
                                    val lStart = oldText.lastIndexOf('\n', cursor - 2).let { if (it == -1) 0 else it + 1 }
                                    val prevLine = oldText.substring(lStart, cursor - 1)
                                    val prefixMatch = Regex("^(> |-\\s+\\[[ x]\\]\\s+|-\\s+|\\d+\\.\\s+)").find(prevLine)

                                    if (prefixMatch != null) {
                                        val prefix = prefixMatch.value
                                        if (prevLine == prefix) {
                                            val correctedText = oldText.substring(0, lStart) + oldText.substring(cursor - 1)
                                            finalValue = TextFieldValue(correctedText, TextRange(lStart))
                                        } else {
                                            var autoPrefix = prefix
                                            if (autoPrefix.startsWith("- [x] ")) autoPrefix = "- [ ] "
                                            else if (autoPrefix.matches(Regex("^\\d+\\.\\s+"))) autoPrefix = "${autoPrefix.substringBefore(".").toInt() + 1}. "

                                            val correctedText = finalValue.text.substring(0, cursor) + autoPrefix + finalValue.text.substring(cursor)
                                            finalValue = TextFieldValue(correctedText, TextRange(cursor + autoPrefix.length))
                                        }
                                    }
                                }

                                textFieldValue = finalValue
                                onContentChange(finalValue.text)
                            },
                            modifier = Modifier.fillMaxSize().focusRequester(bodyFocusRequester),
                            visualTransformation = MarkdownVisualTransformation(primaryColor, highlightColor, baseFontSize),
                            textStyle = MaterialTheme.typography.bodyLarge.copy(
                                fontFamily = LocalNoteFontFamily.current,
                                fontWeight = LocalNoteFontWeight.current,
                                fontSize = LocalNoteTextSize.current,
                                lineHeight = (LocalNoteTextSize.current.value * LocalNoteLineSpacing.current).sp,
                                color = MaterialTheme.colorScheme.onSurface
                            ),
                            placeholder = { Text(stringResource(R.string.edit_placeholder)) },
                            colors = TextFieldDefaults.colors(
                                focusedContainerColor = Color.Transparent,
                                unfocusedContainerColor = Color.Transparent,
                                focusedIndicatorColor = Color.Transparent,
                                unfocusedIndicatorColor = Color.Transparent,
                            ),
                        )
                    }
                }
            }

            // ==================== 底部区域：浮动两层工具栏 ====================
            Column(
                modifier = Modifier
                    .align(Alignment.BottomCenter)
                    .fillMaxWidth()
                    .imePadding()
                    .padding(bottom = if (isImeVisible) 8.dp else 24.dp),
                horizontalAlignment = Alignment.CenterHorizontally
            ) {
                // 1. 二级浮动栏
                AnimatedVisibility(
                    visible = activeSubMenu != EditorSubMenu.NONE,
                    enter = slideInVertically(initialOffsetY = { it }) + fadeIn(),
                    exit = slideOutVertically(targetOffsetY = { it }) + fadeOut()
                ) {
                    Surface(
                        modifier = Modifier.padding(bottom = 12.dp),
                        shape = RoundedCornerShape(percent = 50),
                        color = MaterialTheme.colorScheme.surfaceContainerHighest,
                        tonalElevation = 6.dp,
                        shadowElevation = 6.dp
                    ) {
                        Row(
                            modifier = Modifier.padding(horizontal = 8.dp, vertical = 4.dp).horizontalScroll(rememberScrollState()),
                            verticalAlignment = Alignment.CenterVertically,
                            horizontalArrangement = Arrangement.spacedBy(4.dp)
                        ) {
                            when (activeSubMenu) {
                                EditorSubMenu.HEADING -> {
                                    TextButton(onClick = { applyLinePrefix("# ") }) { Text("H1", color = if(activeHeading==1) primaryColor else MaterialTheme.colorScheme.onSurfaceVariant) }
                                    TextButton(onClick = { applyLinePrefix("## ") }) { Text("H2", color = if(activeHeading==2) primaryColor else MaterialTheme.colorScheme.onSurfaceVariant) }
                                    TextButton(onClick = { applyLinePrefix("### ") }) { Text("H3", color = if(activeHeading==3) primaryColor else MaterialTheme.colorScheme.onSurfaceVariant) }
                                    TextButton(onClick = { applyLinePrefix("#### ") }) { Text("H4", color = if(activeHeading==4) primaryColor else MaterialTheme.colorScheme.onSurfaceVariant) }
                                }
                                EditorSubMenu.LIST -> {
                                    TextButton(onClick = { applyLinePrefix("- ") }) { Text("无序列表", color = if(isUnorderedListActive) primaryColor else MaterialTheme.colorScheme.onSurfaceVariant) }
                                    TextButton(onClick = { applyLinePrefix("1. ") }) { Text("有序列表", color = if(isOrderedListActive) primaryColor else MaterialTheme.colorScheme.onSurfaceVariant) }
                                }
                                else -> Unit
                            }
                        }
                    }
                }

                // 2. 主层浮动栏
                Surface(
                    shape = RoundedCornerShape(percent = 50),
                    color = MaterialTheme.colorScheme.surfaceContainerHigh,
                    tonalElevation = 8.dp,
                    shadowElevation = 6.dp,
                    modifier = Modifier.padding(horizontal = 16.dp)
                ) {
                    Row(
                        modifier = Modifier
                            .fillMaxWidth()
                            .padding(horizontal = 8.dp, vertical = 6.dp)
                            .horizontalScroll(rememberScrollState()),
                        horizontalArrangement = Arrangement.SpaceBetween,
                        verticalAlignment = Alignment.CenterVertically
                    ) {
                        EditorIconButton(isActive = isBoldActive, onClick = { applyWrapStyle("**", "**") }, icon = Icons.Filled.FormatBold, isEnabled = isTextSelected, primaryColor = primaryColor)
                        EditorIconButton(isActive = isItalicActive, onClick = { applyWrapStyle("*", "*") }, icon = Icons.Filled.FormatItalic, isEnabled = isTextSelected, primaryColor = primaryColor)
                        EditorIconButton(isActive = isStrikeActive, onClick = { applyWrapStyle("~~", "~~") }, icon = Icons.Filled.FormatStrikethrough, isEnabled = isTextSelected, primaryColor = primaryColor)
                        EditorIconButton(isActive = isUnderlineActive, onClick = { applyWrapStyle("<u>", "</u>") }, icon = Icons.Filled.FormatUnderlined, isEnabled = isTextSelected, primaryColor = primaryColor)
                        EditorIconButton(isActive = isHighlightActive, onClick = { applyWrapStyle("==", "==") }, icon = Icons.Filled.FormatColorText, isEnabled = isTextSelected, primaryColor = primaryColor)
                        EditorIconButton(isActive = activeHeading > 0, onClick = { activeSubMenu = if (activeSubMenu == EditorSubMenu.HEADING) EditorSubMenu.NONE else EditorSubMenu.HEADING }, icon = Icons.Filled.Title, primaryColor = primaryColor)
                        EditorIconButton(isActive = isAnyCheckboxActive, onClick = applyCheckboxToggle, icon = Icons.Filled.CheckBox, primaryColor = primaryColor)
                        EditorIconButton(isActive = isQuoteActive, onClick = { applyLinePrefix("> ") }, icon = Icons.Filled.FormatQuote, primaryColor = primaryColor)
                        EditorIconButton(isActive = isAnyListActive, onClick = { activeSubMenu = if (activeSubMenu == EditorSubMenu.LIST) EditorSubMenu.NONE else EditorSubMenu.LIST }, icon = Icons.Filled.FormatListBulleted, primaryColor = primaryColor)
                    }
                }
            }
        }
    }
}
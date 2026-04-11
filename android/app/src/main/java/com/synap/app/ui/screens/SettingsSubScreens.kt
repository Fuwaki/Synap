package com.synap.app.ui.screens

import androidx.compose.runtime.Composable

@Composable
fun TypographySettingsContainer(
    currentFontFamily: String,
    onFontFamilyChange: (String) -> Unit,
    currentFontWeight: Int,
    onFontWeightChange: (Int) -> Unit,
    noteTextSize: Float,
    onNoteTextSizeChange: (Float) -> Unit,
    noteLineSpacing: Float, // --- 补充行距参数 ---
    onNoteLineSpacingChange: (Float) -> Unit, // --- 补充行距回调 ---
    onNavigateBack: () -> Unit
) {
    TypographySettingsScreen(
        currentFontFamily = currentFontFamily,
        onFontFamilyChange = onFontFamilyChange,
        currentFontWeight = currentFontWeight,
        onFontWeightChange = onFontWeightChange,
        noteTextSize = noteTextSize,
        onNoteTextSizeChange = onNoteTextSizeChange,
        noteLineSpacing = noteLineSpacing, // --- 透传给内部组件 ---
        onNoteLineSpacingChange = onNoteLineSpacingChange, // --- 透传给内部组件 ---
        onNavigateBack = onNavigateBack
    )
}

@Composable
fun LanguageSelectionContainer(
    languages: List<String>,
    selectedIndex: Int,
    onLanguageSelect: (Int) -> Unit,
    onNavigateBack: () -> Unit
) {
    LanguageSelectionScreen(
        languages = languages,
        selectedIndex = selectedIndex,
        onLanguageSelect = onLanguageSelect,
        onNavigateBack = onNavigateBack
    )
}
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
    onNavigateBack: () -> Unit
) {
    TypographySettingsScreen(
        currentFontFamily = currentFontFamily,
        onFontFamilyChange = onFontFamilyChange,
        currentFontWeight = currentFontWeight,
        onFontWeightChange = onFontWeightChange,
        noteTextSize = noteTextSize,
        onNoteTextSizeChange = onNoteTextSizeChange,
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

package io.element.android.wysiwyg.compose.testutils

import androidx.compose.foundation.background
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.material3.MaterialTheme
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.test.junit4.ComposeContentTestRule
import io.element.android.wysiwyg.compose.RichTextEditor
import io.element.android.wysiwyg.compose.RichTextEditorState

fun ComposeContentTestRule.showContent(
    state: RichTextEditorState,
) = setContent {
    MaterialTheme {
        RichTextEditor(
            state = state, modifier = Modifier.fillMaxWidth().background(Color.Cyan)
        )
    }
}


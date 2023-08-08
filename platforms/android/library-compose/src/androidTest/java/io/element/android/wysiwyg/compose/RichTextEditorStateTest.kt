package io.element.android.wysiwyg.compose

import androidx.compose.material3.MaterialTheme
import androidx.compose.ui.test.junit4.createComposeRule
import kotlinx.coroutines.test.runTest
import org.junit.Rule
import org.junit.Test

class RichTextEditorStateTest {
    @get:Rule
    val composeTestRule = createComposeRule()

    @Test(expected = IllegalStateException::class)
    fun testSharingState() = runTest {
        val state = RichTextEditorState()
        composeTestRule.setContent {
            MaterialTheme {
                RichTextEditor(state)
                RichTextEditor(state)
            }
        }
    }
}

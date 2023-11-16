package io.element.android.wysiwyg.compose

import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.collectAsState
import androidx.compose.runtime.getValue
import androidx.compose.ui.Modifier
import androidx.compose.ui.test.assertIsDisplayed
import androidx.compose.ui.test.junit4.createComposeRule
import androidx.compose.ui.test.onNodeWithText
import androidx.test.espresso.Espresso.onView
import androidx.test.espresso.assertion.ViewAssertions.matches
import androidx.test.espresso.matcher.ViewMatchers.isDisplayed
import androidx.test.espresso.matcher.ViewMatchers.withText
import io.element.android.wysiwyg.utils.NBSP
import io.element.android.wysiwyg.view.models.InlineFormat
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.test.runTest
import org.junit.Assert.assertEquals
import org.junit.Rule
import org.junit.Test

class RichTextEditorStateTest {
    @get:Rule
    val composeTestRule = createComposeRule()

    @Test
    fun testSharingState() = runTest {
        val state = RichTextEditorState()
        val showAlternateEditor = MutableStateFlow(false)
        composeTestRule.setContent {
            MaterialTheme {
                val showAlt by showAlternateEditor.collectAsState()
                Column {
                    if (!showAlt) {
                        Text("Main editor")
                        RichTextEditor(state = state, modifier = Modifier.fillMaxWidth())
                    } else {
                        Text("Alternative editor")
                        RichTextEditor(state = state, modifier = Modifier.fillMaxWidth())
                    }
                }
            }
        }

        state.setHtml("Hello,<br />world")
        state.setSelection(0, 5)
        composeTestRule.awaitIdle()

        composeTestRule.onNodeWithText("Main editor").assertIsDisplayed()
        onView(withText("Hello,\nworld")).check(matches(isDisplayed()))

        showAlternateEditor.emit(true)
        composeTestRule.awaitIdle()

        composeTestRule.onNodeWithText("Alternative editor").assertIsDisplayed()
        onView(withText("Hello,\nworld")).check(matches(isDisplayed()))
        assertEquals(0, state.lineCount) // FIXME - line count should be 2

        state.toggleInlineFormat(InlineFormat.Bold)
        composeTestRule.awaitIdle()

        assertEquals("<strong>Hello</strong>,<br />world", state.messageHtml)
    }

    @Test
    fun setHtmlWithTrailingNbspKeepsIt() = runTest {
        val state = RichTextEditorState()
        composeTestRule.setContent {
            MaterialTheme {
                RichTextEditor(state = state)
            }
        }

        state.setHtml("<b>Hey</b>$NBSP")
        composeTestRule.awaitIdle()

        onView(withText("Hey ")).check(matches(isDisplayed()))
        state.setSelection(4)
    }

    @Test
    fun testStateUpdatesDisabled() = runTest {
        val state = RichTextEditorState(
            "Original text"
        )
        composeTestRule.setContent {
            MaterialTheme {
                RichTextEditor(
                    state = state,
                    registerStateUpdates = false
                )
            }
        }

        state.setHtml("Updated text")
        composeTestRule.awaitIdle()

        onView(withText("Original text")).check(matches(isDisplayed()))
    }
}

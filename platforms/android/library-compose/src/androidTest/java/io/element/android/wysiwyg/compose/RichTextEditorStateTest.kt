package io.element.android.wysiwyg.compose

import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.collectAsState
import androidx.compose.runtime.getValue
import androidx.compose.ui.test.assertIsDisplayed
import androidx.compose.ui.test.junit4.createComposeRule
import androidx.compose.ui.test.onNodeWithText
import androidx.test.espresso.Espresso.onView
import androidx.test.espresso.assertion.ViewAssertions.matches
import androidx.test.espresso.matcher.ViewMatchers.isDisplayed
import androidx.test.espresso.matcher.ViewMatchers.withText
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.test.runTest
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
                if(!showAlt) {
                    Text("Main editor")
                    RichTextEditor(state = state)
                } else {
                    Text("Alternative editor")
                    RichTextEditor(state = state)
                }
            }
        }

        state.setHtml("Hello, world")
        composeTestRule.awaitIdle()

        composeTestRule.onNodeWithText("Main editor").assertIsDisplayed()
        onView(withText("Hello, world")).check(matches(isDisplayed()))

        showAlternateEditor.emit(true)
        composeTestRule.awaitIdle()

        composeTestRule.onNodeWithText("Alternative editor").assertIsDisplayed()
        onView(withText("Hello, world")).check(matches(isDisplayed()))
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

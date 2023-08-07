package io.element.android.wysiwyg.compose

import androidx.activity.ComponentActivity
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.material3.MaterialTheme
import androidx.compose.ui.Modifier
import androidx.compose.ui.test.junit4.ComposeContentTestRule
import androidx.compose.ui.test.junit4.createAndroidComposeRule
import androidx.test.espresso.Espresso.onView
import androidx.test.espresso.action.ViewActions.replaceText
import androidx.test.espresso.assertion.ViewAssertions.matches
import androidx.test.espresso.matcher.ViewMatchers.isAssignableFrom
import androidx.test.espresso.matcher.ViewMatchers.isDisplayed
import androidx.test.espresso.matcher.ViewMatchers.withText
import io.element.android.wysiwyg.EditorEditText
import kotlinx.coroutines.test.runTest
import org.junit.Assert.assertEquals
import org.junit.Rule
import org.junit.Test
import uniffi.wysiwyg_composer.ActionState
import uniffi.wysiwyg_composer.ComposerAction
import uniffi.wysiwyg_composer.MenuAction

class RichTextEditorTest {
    @get:Rule
    val composeTestRule = createAndroidComposeRule<ComponentActivity>()

    @Test
    fun testTypeText() = runTest {
        val state = createState()
        composeTestRule.showContent(state)

        onView(isRichTextEditor())
            // Note that ViewAction.typeText is not working in this setup
            // so use ViewAction.replaceText instead
            .perform(replaceText("Hello, world"))

        assertEquals(
            DEFAULT_ACTIONS.copy(mapOf(ComposerAction.UNDO to ActionState.ENABLED)),
            state.actions
        )
        assertEquals(MenuAction.None, state.menuAction)
        assertEquals(12 to 12, state.selection)
        assertEquals("Hello, world", state.messageHtml)
        assertEquals("Hello, world", state.messageMarkdown)
    }

    @Test
    fun testSetHtml() {
        val state = createState()
        composeTestRule.showContent(state)

        state.setHtml("Hello, world")

        onView(withText("Hello, world")).check(matches(isDisplayed()))

        assertEquals(DEFAULT_ACTIONS, state.actions)
        assertEquals(12 to 12, state.selection)
        assertEquals(MenuAction.None, state.menuAction)
        assertEquals("Hello, world", state.messageHtml)
        assertEquals("Hello, world", state.messageMarkdown)
    }

    @Test
    fun testSetHtmlFormatted() = runTest {
        val state = createState()
        composeTestRule.showContent(state)

        state.setHtml("Hello, <b><i>world</i></b>")

        onView(withText("Hello, world")).check(matches(isDisplayed()))

        assertEquals(
            DEFAULT_ACTIONS.copy(
                mapOf(
                    ComposerAction.BOLD to ActionState.REVERSED,
                    ComposerAction.ITALIC to ActionState.REVERSED,
                )
            ), state.actions
        )
        assertEquals(12 to 12, state.selection)
        assertEquals(MenuAction.None, state.menuAction)
        assertEquals("Hello, <b><i>world</i></b>", state.messageHtml)
        assertEquals("Hello, __*world*__", state.messageMarkdown)
    }

    private fun ComposeContentTestRule.showContent(
        state: RichTextEditorState,
    ) = setContent {
        MaterialTheme {
            RichTextEditor(
                state, modifier = Modifier.fillMaxWidth()
            )
        }
    }

    private fun createState(): RichTextEditorState {
        val context = composeTestRule.activity
        return RichTextEditorState(
            EditorEditText(
                context
            )
        )
    }

    companion object {
        val DEFAULT_ACTIONS =
            mapOf(
                ComposerAction.INDENT to ActionState.DISABLED,
                ComposerAction.STRIKE_THROUGH to ActionState.ENABLED,
                ComposerAction.UNORDERED_LIST to ActionState.ENABLED,
                ComposerAction.ORDERED_LIST to ActionState.ENABLED,
                ComposerAction.ITALIC to ActionState.ENABLED,
                ComposerAction.UNDO to ActionState.DISABLED,
                ComposerAction.QUOTE to ActionState.ENABLED,
                ComposerAction.UNDERLINE to ActionState.ENABLED,
                ComposerAction.REDO to ActionState.DISABLED,
                ComposerAction.BOLD to ActionState.ENABLED,
                ComposerAction.LINK to ActionState.ENABLED,
                ComposerAction.INLINE_CODE to ActionState.ENABLED,
                ComposerAction.CODE_BLOCK to ActionState.ENABLED,
                ComposerAction.UNINDENT to ActionState.DISABLED
            )

    }
}


private fun isRichTextEditor() = isAssignableFrom(EditorEditText::class.java)

private fun Map<ComposerAction, ActionState>.copy(
    newEntries: Map<ComposerAction, ActionState>
) = RichTextEditorTest.DEFAULT_ACTIONS.mapValues {
    newEntries[it.key] ?: it.value
}

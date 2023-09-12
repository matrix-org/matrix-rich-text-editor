package io.element.android.wysiwyg.compose

import androidx.activity.ComponentActivity
import androidx.compose.ui.test.junit4.createAndroidComposeRule
import androidx.test.espresso.Espresso.onView
import androidx.test.espresso.action.ViewActions.replaceText
import androidx.test.espresso.assertion.ViewAssertions.matches
import androidx.test.espresso.matcher.ViewMatchers.isDisplayed
import androidx.test.espresso.matcher.ViewMatchers.withText
import io.element.android.wysiwyg.compose.testutils.ComposerActions.DEFAULT_ACTIONS
import io.element.android.wysiwyg.compose.testutils.EditorActions
import io.element.android.wysiwyg.compose.testutils.StateFactory.createState
import io.element.android.wysiwyg.compose.testutils.ViewMatchers.isRichTextEditor
import io.element.android.wysiwyg.compose.testutils.copy
import io.element.android.wysiwyg.compose.testutils.showContent
import io.element.android.wysiwyg.view.models.LinkAction
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
    fun testTypeText() {
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
    fun testSetHtml() = runTest {
        val state = createState()
        composeTestRule.showContent(state)

        composeTestRule.runOnUiThread {
            state.setHtml("Hello, world")
        }

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

        composeTestRule.runOnUiThread {
            state.setHtml("Hello, <b><i>world</i></b>")
        }

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

    @Test
    fun testInsertLink() = runTest {
        val state = createState()
        composeTestRule.showContent(state)

        composeTestRule.runOnUiThread {
            state.setHtml("Hello, ")
            state.insertLink("https://element.io", "element")
        }

        assertEquals("Hello, <a href=\"https://element.io\">element</a>", state.messageHtml)
        assertEquals("Hello, [element](<https://element.io>)", state.messageMarkdown)
    }

    @Test
    fun testRemoveLink() = runTest {
        val state = createState()
        composeTestRule.showContent(state)

        composeTestRule.runOnUiThread {
            state.setHtml("Hello, <a href=\"https://element.io\">element</a>")
            state.removeLink()
        }

        assertEquals("Hello, element", state.messageHtml)
        assertEquals("Hello, element", state.messageMarkdown)
    }

    @Test
    fun testSetLink() = runTest {
        val state = createState()
        composeTestRule.showContent(state)

        composeTestRule.runOnUiThread {
            state.setHtml("Hello, <a href=\"https://element.io\">element</a>")
            state.setLink("https://matrix.org")
        }

        assertEquals("Hello, <a href=\"https://matrix.org\">element</a>", state.messageHtml)
        assertEquals("Hello, [element](<https://matrix.org>)", state.messageMarkdown)
    }

    @Test
    fun testLinkActionUpdates() = runTest {
        val state = createState()
        composeTestRule.showContent(state)

        composeTestRule.runOnUiThread {
            state.setHtml("<a href=\"https://matrix.org\">matrix</a> <a href=\"https://element.io\">element</a> plain")
        }

        onView(withText("matrix element plain")).perform(EditorActions.setSelection(0, 0))
        assertEquals(LinkAction.SetLink("https://matrix.org"), state.linkAction)

        onView(withText("matrix element plain")).perform(EditorActions.setSelection(8, 8))
        assertEquals(LinkAction.SetLink("https://element.io"), state.linkAction)

        onView(withText("matrix element plain")).perform(EditorActions.setSelection(16, 16))
        assertEquals(LinkAction.InsertLink, state.linkAction)

        onView(withText("matrix element plain")).perform(EditorActions.setSelection(16, 20))
        assertEquals(LinkAction.SetLink(null), state.linkAction)
    }
}

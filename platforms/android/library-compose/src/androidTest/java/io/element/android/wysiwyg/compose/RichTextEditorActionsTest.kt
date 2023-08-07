package io.element.android.wysiwyg.compose

import androidx.compose.ui.test.junit4.createComposeRule
import io.element.android.wysiwyg.compose.testutils.ComposerActions
import io.element.android.wysiwyg.compose.testutils.StateFactory
import io.element.android.wysiwyg.compose.testutils.copy
import io.element.android.wysiwyg.compose.testutils.showContent
import io.element.android.wysiwyg.view.models.InlineFormat
import kotlinx.coroutines.test.runTest
import org.junit.Assert.assertEquals
import org.junit.Rule
import org.junit.Test
import uniffi.wysiwyg_composer.ActionState
import uniffi.wysiwyg_composer.ComposerAction

class RichTextEditorActionsTest {
    @get:Rule
    val composeTestRule = createComposeRule()

    @Test
    fun testBold() = runTest {
        val state = StateFactory.createState()
        composeTestRule.showContent(state)

        state.toggleInlineFormat(InlineFormat.Bold)

        assertEquals(
            ComposerActions.DEFAULT_ACTIONS.copy(
                mapOf(
                    ComposerAction.UNDO to ActionState.ENABLED,
                    ComposerAction.BOLD to ActionState.REVERSED
                )
            ),
            state.actions
        )
    }

    @Test
    fun testItalic() = runTest {
        val state = StateFactory.createState()
        composeTestRule.showContent(state)

        state.toggleInlineFormat(InlineFormat.Italic)

        assertEquals(
            ComposerActions.DEFAULT_ACTIONS.copy(
                mapOf(
                    ComposerAction.UNDO to ActionState.ENABLED,
                    ComposerAction.ITALIC to ActionState.REVERSED
                )
            ),
            state.actions
        )
    }

    @Test
    fun testStrikeThrough() = runTest {
        val state = StateFactory.createState()
        composeTestRule.showContent(state)

        state.toggleInlineFormat(InlineFormat.StrikeThrough)

        assertEquals(
            ComposerActions.DEFAULT_ACTIONS.copy(
                mapOf(
                    ComposerAction.UNDO to ActionState.ENABLED,
                    ComposerAction.STRIKE_THROUGH to ActionState.REVERSED
                )
            ),
            state.actions
        )
    }

    @Test
    fun testUnderline() = runTest {
        val state = StateFactory.createState()
        composeTestRule.showContent(state)

        state.toggleInlineFormat(InlineFormat.Underline)

        assertEquals(
            ComposerActions.DEFAULT_ACTIONS.copy(
                mapOf(
                    ComposerAction.UNDO to ActionState.ENABLED,
                    ComposerAction.UNDERLINE to ActionState.REVERSED
                )
            ),
            state.actions
        )
    }

    @Test
    fun testQuote() = runTest {
        val state = StateFactory.createState()
        composeTestRule.showContent(state)

        state.toggleQuote()

        assertEquals(
            ComposerActions.DEFAULT_ACTIONS.copy(
                mapOf(
                    ComposerAction.QUOTE to ActionState.REVERSED,
                )
            ),
            state.actions
        )
    }

    @Test
    fun testCodeBlock() = runTest {
        val state = StateFactory.createState()
        composeTestRule.showContent(state)

        state.toggleCodeBlock()

        assertEquals(
            ComposerActions.DEFAULT_ACTIONS.copy(
                mapOf(
                    ComposerAction.CODE_BLOCK to ActionState.REVERSED,
                    ComposerAction.INLINE_CODE to ActionState.DISABLED,
                    ComposerAction.QUOTE to ActionState.DISABLED,
                    ComposerAction.LINK to ActionState.DISABLED,
                    ComposerAction.UNORDERED_LIST to ActionState.DISABLED,
                    ComposerAction.ORDERED_LIST to ActionState.DISABLED,
                )
            ),
            state.actions
        )
    }

    @Test
    fun testInlineCode() = runTest {
        val state = StateFactory.createState()
        composeTestRule.showContent(state)

        state.toggleInlineFormat(InlineFormat.InlineCode)

        assertEquals(
            ComposerActions.DEFAULT_ACTIONS.copy(
                mapOf(
                    ComposerAction.UNDO to ActionState.ENABLED,
                    ComposerAction.INLINE_CODE to ActionState.REVERSED,
                    ComposerAction.LINK to ActionState.DISABLED,
                    ComposerAction.BOLD to ActionState.DISABLED,
                    ComposerAction.STRIKE_THROUGH to ActionState.DISABLED,
                    ComposerAction.ITALIC to ActionState.DISABLED,
                    ComposerAction.UNDERLINE to ActionState.DISABLED,
                )
            ),
            state.actions
        )
    }

    @Test
    fun testUndoRedo() = runTest {
        val state = StateFactory.createState()
        composeTestRule.showContent(state)

        state.toggleInlineFormat(InlineFormat.Bold)

        assertEquals(
            ComposerActions.DEFAULT_ACTIONS.copy(
                mapOf(
                    ComposerAction.UNDO to ActionState.ENABLED,
                    ComposerAction.BOLD to ActionState.REVERSED
                )
            ),
            state.actions
        )

        state.undo()

        assertEquals(
            ComposerActions.DEFAULT_ACTIONS.copy(
                mapOf(
                    ComposerAction.REDO to ActionState.ENABLED,
                )
            ),
            state.actions
        )

        state.redo()

        assertEquals(
            ComposerActions.DEFAULT_ACTIONS.copy(
                mapOf(
                    ComposerAction.UNDO to ActionState.ENABLED,
                    ComposerAction.BOLD to ActionState.REVERSED
                )
            ),
            state.actions
        )
    }

    @Test
    fun testOrderedList() = runTest {
        val state = StateFactory.createState()

        composeTestRule.showContent(state)

        state.toggleList(ordered = true)

        assertEquals(
            ComposerActions.DEFAULT_ACTIONS.copy(
                mapOf(
                    ComposerAction.UNDO to ActionState.ENABLED,
                    ComposerAction.ORDERED_LIST to ActionState.REVERSED,
                )
            ), state.actions)
    }

    @Test
    fun testUnorderedList() = runTest {
        val state = StateFactory.createState()

        composeTestRule.showContent(state)

        state.toggleList(ordered = false)

        assertEquals(
            ComposerActions.DEFAULT_ACTIONS.copy(
                mapOf(
                    ComposerAction.UNDO to ActionState.ENABLED,
                    ComposerAction.UNORDERED_LIST to ActionState.REVERSED,
                )
            ), state.actions)

    }

    @Test
    fun testIndent() = runTest {
        val state = StateFactory.createState()

        composeTestRule.showContent(state)

        state.setHtml("<ol><li>Test</li><li>Test</li></ol>")

        assertEquals(
            ComposerActions.DEFAULT_ACTIONS.copy(
                mapOf(
                    ComposerAction.ORDERED_LIST to ActionState.REVERSED,
                    ComposerAction.INDENT to ActionState.ENABLED,
                    ComposerAction.UNINDENT to ActionState.DISABLED,
                )
            ), state.actions)

        state.indent()

        assertEquals(
            ComposerActions.DEFAULT_ACTIONS.copy(
                mapOf(
                    ComposerAction.UNDO to ActionState.ENABLED,
                    ComposerAction.ORDERED_LIST to ActionState.REVERSED,
                    ComposerAction.INDENT to ActionState.DISABLED,
                    ComposerAction.UNINDENT to ActionState.ENABLED,
                )
            ), state.actions)
    }

    @Test
    fun testUnindent() = runTest {
        val state = StateFactory.createState()

        composeTestRule.showContent(state)

        state.setHtml("<ol><li>Test</li><li><ol><li>Test</li></ol></li></ol>")

        assertEquals(
            ComposerActions.DEFAULT_ACTIONS.copy(
                mapOf(
                    ComposerAction.ORDERED_LIST to ActionState.REVERSED,
                    ComposerAction.INDENT to ActionState.DISABLED,
                    ComposerAction.UNINDENT to ActionState.ENABLED,
                )
            ), state.actions)

        state.unindent()

        assertEquals(
            ComposerActions.DEFAULT_ACTIONS.copy(
                mapOf(
                    ComposerAction.UNDO to ActionState.ENABLED,
                    ComposerAction.ORDERED_LIST to ActionState.REVERSED,
                    ComposerAction.INDENT to ActionState.ENABLED,
                    ComposerAction.UNINDENT to ActionState.DISABLED,
                )
            ), state.actions)

    }

}
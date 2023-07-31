package io.element.android.wysiwyg.compose

import androidx.compose.material3.MaterialTheme
import androidx.compose.runtime.collectAsState
import androidx.compose.runtime.getValue
import androidx.compose.ui.test.junit4.ComposeContentTestRule
import androidx.compose.ui.test.junit4.createComposeRule
import androidx.test.espresso.Espresso.onView
import androidx.test.espresso.action.ViewActions.replaceText
import androidx.test.espresso.assertion.ViewAssertions.matches
import androidx.test.espresso.matcher.ViewMatchers.isAssignableFrom
import androidx.test.espresso.matcher.ViewMatchers.isDisplayed
import androidx.test.espresso.matcher.ViewMatchers.withText
import io.element.android.wysiwyg.EditorEditText
import io.mockk.confirmVerified
import io.mockk.mockk
import io.mockk.spyk
import io.mockk.verify
import kotlinx.coroutines.flow.MutableSharedFlow
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.test.runTest
import org.junit.Rule
import org.junit.Test
import uniffi.wysiwyg_composer.ActionState
import uniffi.wysiwyg_composer.ComposerAction
import uniffi.wysiwyg_composer.MenuAction

class RichTextEditorTest {
    @get:Rule
    val composeTestRule = createComposeRule()

    private val actions = MutableSharedFlow<EditorAction>()
    private val onMarkdownChanged = mockk<(String) -> Unit>(relaxed = true) {
    }
    private val onHtmlChanged = mockk<(String) -> Unit>(relaxed = true)
    private val onActionsChanged =
        mockk<(actions: Map<ComposerAction, ActionState>) -> Unit>(relaxed = true)
    private val onSelectionChanged = mockk<(start: Int, end: Int) -> Unit>(relaxed = true)
    private val onMenuActionChanged = mockk<(menuAction: MenuAction) -> Unit>(relaxed = true)

    @Test
    fun testTypeText() = runTest {
        composeTestRule.showContent()

        onView(isRichTextEditor())
            // Note that ViewAction.typeText is not working in this setup
            // so use ViewAction.replaceText instead
            .perform(replaceText("Hello, world"))

        verify {
            onActionsChanged(DEFAULT_ACTIONS.copy(mapOf(ComposerAction.UNDO to ActionState.ENABLED)))
            onSelectionChanged(12, 12)
            onMenuActionChanged(MenuAction.None)
            onHtmlChanged("Hello, world")
            onMarkdownChanged("Hello, world")
        }

        confirmNoMoreChanges()
    }

    @Test
    fun testSetHtml() = runTest {
        composeTestRule.showContent()

        actions.emit(EditorAction.SetHtml("Hello, world"))

        onView(withText("Hello, world")).check(matches(isDisplayed()))

        verify {
            onActionsChanged(DEFAULT_ACTIONS)
            onSelectionChanged(12, 12)
            onMenuActionChanged(MenuAction.None)
            onHtmlChanged("Hello, world")
            onMarkdownChanged("Hello, world")
        }

        confirmNoMoreChanges()
    }

    @Test
    fun testSetHtmlFormatted() = runTest {
        composeTestRule.showContent()

        actions.emit(EditorAction.SetHtml("Hello, <b><i>world</i></b>"))

        onView(withText("Hello, world")).check(matches(isDisplayed()))

        verify {
            onActionsChanged(
                DEFAULT_ACTIONS.copy(
                    mapOf(
                        ComposerAction.BOLD to ActionState.REVERSED,
                        ComposerAction.ITALIC to ActionState.REVERSED,
                    )
                )
            )
            onSelectionChanged(12, 12)
            onMenuActionChanged(MenuAction.None)
            onHtmlChanged("Hello, <b><i>world</i></b>")
            onMarkdownChanged("Hello, __*world*__")
        }
        confirmNoMoreChanges()
    }

    /**
     * Test that new markdown / html callbacks are registered properly on recomposition
     */
    @Test
    fun testUpdateCallbacks() = runTest {
        val markdownCallbacks = MutableStateFlow(onMarkdownChanged)
        val htmlCallbacks = MutableStateFlow(onHtmlChanged)
        val newOnMarkdownChanged = spyk<(String) -> Unit>({ })
        val newOnHtmlChanged = spyk<(String) -> Unit>({ })

        composeTestRule.setContent {
            MaterialTheme {
                val onMarkdownChanged by markdownCallbacks.collectAsState()
                val onHtmlChanged by htmlCallbacks.collectAsState()

                RichTextEditor(
                    actions = actions,
                    onMarkdownChanged = onMarkdownChanged,
                    onHtmlChanged = onHtmlChanged,
                    onActionsChanged = onActionsChanged,
                    onMenuActionChanged = onMenuActionChanged,
                    onSelectionChanged = onSelectionChanged,
                )

            }
        }

        actions.emit(EditorAction.SetHtml("<b>Text 1</b>"))

        markdownCallbacks.emit(newOnMarkdownChanged)
        htmlCallbacks.emit(newOnHtmlChanged)

        composeTestRule.awaitIdle()

        actions.emit(EditorAction.SetHtml("<b>Text 2</b>"))

        verify {
            onHtmlChanged("<b>Text 1</b>")
            onMarkdownChanged("__Text 1__")
            newOnHtmlChanged("<b>Text 2</b>")
            newOnMarkdownChanged("__Text 2__")
        }
        confirmVerified(
            onHtmlChanged, onMarkdownChanged, newOnHtmlChanged, newOnMarkdownChanged
        )
    }

    private fun confirmNoMoreChanges() =
        confirmVerified(
            onMarkdownChanged,
            onHtmlChanged,
            onActionsChanged,
            onSelectionChanged,
            onMenuActionChanged
        )

    private fun ComposeContentTestRule.showContent() {
        setContent {
            MaterialTheme {
                RichTextEditor(
                    actions = actions,
                    onMarkdownChanged = onMarkdownChanged,
                    onHtmlChanged = onHtmlChanged,
                    onActionsChanged = onActionsChanged,
                    onMenuActionChanged = onMenuActionChanged,
                    onSelectionChanged = onSelectionChanged,
                )
            }
        }

        verify {
            onActionsChanged(DEFAULT_ACTIONS)
        }
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

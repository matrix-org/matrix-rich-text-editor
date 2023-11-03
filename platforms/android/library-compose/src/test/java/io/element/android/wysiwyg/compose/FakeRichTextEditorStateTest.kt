package io.element.android.wysiwyg.compose

import androidx.compose.runtime.remember
import app.cash.molecule.RecompositionMode
import app.cash.molecule.moleculeFlow
import app.cash.turbine.test
import io.element.android.wysiwyg.view.models.InlineFormat
import io.element.android.wysiwyg.view.models.LinkAction
import kotlinx.coroutines.test.runTest
import org.hamcrest.CoreMatchers.equalTo
import org.hamcrest.MatcherAssert.assertThat
import org.junit.Test
import uniffi.wysiwyg_composer.ActionState
import uniffi.wysiwyg_composer.ComposerAction


class FakeRichTextEditorStateTest {
    @Test
    fun `toggleInlineFormat(bold) updates the state`() = runTest {
        moleculeFlow(RecompositionMode.Immediate) {
            val state = rememberRichTextEditorState(fake = true)
            remember(state.actions) { state }
        }.test {
            val initialState = awaitItem()
            initialState.toggleInlineFormat(inlineFormat = InlineFormat.Bold)
            val actions = awaitItem().actions
            assertThat(actions[ComposerAction.BOLD], equalTo(ActionState.REVERSED))
        }
    }

    @Test
    fun `toggleInlineFormat(italic) updates the state`() = runTest {
        moleculeFlow(RecompositionMode.Immediate) {
            val state = rememberRichTextEditorState(fake = true)
            remember(state.actions) { state }
        }.test {
            val initialState = awaitItem()
            initialState.toggleInlineFormat(inlineFormat = InlineFormat.Italic)
            val actions = awaitItem().actions
            assertThat(actions[ComposerAction.ITALIC], equalTo(ActionState.REVERSED))
        }
    }

    @Test
    fun `toggleInlineFormat(underline) updates the state`() = runTest {
        moleculeFlow(RecompositionMode.Immediate) {
            val state = rememberRichTextEditorState(fake = true)
            remember(state.actions){ state }
        }.test {
            val initialState = awaitItem()
            initialState.toggleInlineFormat(inlineFormat = InlineFormat.Underline)
            val actions = awaitItem().actions
            assertThat(actions[ComposerAction.UNDERLINE], equalTo(ActionState.REVERSED))
        }
    }

    @Test
    fun `toggleInlineFormat(strikethrough) updates the state`() = runTest {
        moleculeFlow(RecompositionMode.Immediate) {
            val state = rememberRichTextEditorState(fake = true)
            remember(state.actions) { state }
        }.test {
            val initialState = awaitItem()
            initialState.toggleInlineFormat(inlineFormat = InlineFormat.StrikeThrough)
            val actions = awaitItem().actions
            assertThat(actions[ComposerAction.STRIKE_THROUGH], equalTo(ActionState.REVERSED))
        }
    }

    @Test
    fun `toggleInlineFormat(inlinecode) updates the state`() = runTest {
        moleculeFlow(RecompositionMode.Immediate) {
            val state = rememberRichTextEditorState(fake = true)
            remember(state.actions) { state }
        }.test {
            val initialState = awaitItem()
            initialState.toggleInlineFormat(inlineFormat = InlineFormat.InlineCode)
            val actions = awaitItem().actions
            assertThat(actions[ComposerAction.INLINE_CODE], equalTo(ActionState.REVERSED))
        }
    }

    @Test
    fun `toggleList(ordered) updates the state`() = runTest {
        moleculeFlow(RecompositionMode.Immediate) {
            val state = rememberRichTextEditorState(fake = true)
            remember(state.actions) { state }
        }.test {
            val initialState = awaitItem()
            initialState.toggleList(ordered = true)
            val actions = awaitItem().actions
            assertThat(actions[ComposerAction.ORDERED_LIST], equalTo(ActionState.REVERSED))
        }
    }

    @Test
    fun `toggleList(unordered) updates the state`() = runTest {
        moleculeFlow(RecompositionMode.Immediate) {
            val state = rememberRichTextEditorState(fake = true)
            remember(state.actions) { state }
        }.test {
            val initialState = awaitItem()
            initialState.toggleList(ordered = false)
            val actions = awaitItem().actions
            assertThat(actions[ComposerAction.UNORDERED_LIST], equalTo(ActionState.REVERSED))
        }
    }

    @Test
    fun `toggleCodeBlock updates the state`() = runTest {
        moleculeFlow(RecompositionMode.Immediate) {
            val state = rememberRichTextEditorState(fake = true)
            remember(state.actions) { state }
        }.test {
            val initialState = awaitItem()
            initialState.toggleCodeBlock()
            val actions = awaitItem().actions
            assertThat(actions[ComposerAction.CODE_BLOCK], equalTo(ActionState.REVERSED))
        }
    }

    @Test
    fun `toggleQuote updates the state`() = runTest {
        moleculeFlow(RecompositionMode.Immediate) {
            val state = rememberRichTextEditorState(fake = true)
            remember(state.actions) { state }
        }.test {
            val initialState = awaitItem()
            initialState.toggleQuote()
            val actions = awaitItem().actions
            assertThat(actions[ComposerAction.QUOTE], equalTo(ActionState.REVERSED))
        }
    }

    @Test
    fun `undo updates the state`() = runTest {
        moleculeFlow(RecompositionMode.Immediate) {
            val state = rememberRichTextEditorState(fake = true)
            remember(state.actions) { state }
        }.test {
            val initialState = awaitItem()
            initialState.undo()
            val actions = awaitItem().actions
            assertThat(actions[ComposerAction.UNDO], equalTo(ActionState.REVERSED))
        }
    }

    @Test
    fun `redo updates the state`() = runTest {
        moleculeFlow(RecompositionMode.Immediate) {
            val state = rememberRichTextEditorState(fake = true)
            remember(state.actions) { state }
        }.test {
            val initialState = awaitItem()
            initialState.redo()
            val actions = awaitItem().actions
            assertThat(actions[ComposerAction.REDO], equalTo(ActionState.REVERSED))
        }
    }

    @Test
    fun `indent updates the state`() = runTest {
        moleculeFlow(RecompositionMode.Immediate) {
            val state = rememberRichTextEditorState(fake = true)
            remember(state.actions) { state }
        }.test {
            val initialState = awaitItem()
            initialState.indent()
            val actions = awaitItem().actions
            assertThat(actions[ComposerAction.INDENT], equalTo(ActionState.REVERSED))
        }
    }

    @Test
    fun `unindent updates the state`() = runTest {
        moleculeFlow(RecompositionMode.Immediate) {
            val state = rememberRichTextEditorState(fake = true)
            remember(state.actions) { state }
        }.test {
            val initialState = awaitItem()
            initialState.unindent()
            val actions = awaitItem().actions
            assertThat(actions[ComposerAction.UNINDENT], equalTo(ActionState.REVERSED))
        }
    }

    @Test
    fun `setLink updates the state`() = runTest {
        moleculeFlow(RecompositionMode.Immediate) {
            val state = rememberRichTextEditorState(fake = true)
            remember(state.linkAction) { state }
        }.test {
            val initialState = awaitItem()
            initialState.setLink("https://element.io")
            val linkAction = awaitItem().linkAction
            assertThat(linkAction, equalTo(LinkAction.SetLink("https://element.io")))
        }
    }

    @Test
    fun `removeLink updates the state`() = runTest {
        moleculeFlow(RecompositionMode.Immediate) {
            val state = rememberRichTextEditorState(fake = true)
            remember(state.linkAction) { state }
        }.test {
            val initialState = awaitItem()
            initialState.setLink("https://element.io")
            val withLink = awaitItem()
            withLink.removeLink()
            val linkAction = awaitItem().linkAction
            assertThat(linkAction, equalTo(LinkAction.InsertLink))
        }
    }

    @Test
    fun `insertLink updates the state`() = runTest {
        moleculeFlow(RecompositionMode.Immediate) {
            val state = rememberRichTextEditorState(fake = true)
            remember(state.linkAction) { state }
        }.test {
            val initialState = awaitItem()
            initialState.insertLink("https://element.io", "hello!")
            val linkAction = awaitItem().linkAction
            assertThat(linkAction, equalTo(LinkAction.SetLink("https://element.io")))
        }
    }

    @Test
    fun `toggling multiple times toggles state`() = runTest {
        moleculeFlow(RecompositionMode.Immediate) {
            val state = rememberRichTextEditorState(fake = true)
            remember(state.actions) { state }
        }.test {
            val initialState = awaitItem()
            assertThat(initialState.actions[ComposerAction.BOLD], equalTo(null))

            initialState.toggleInlineFormat(InlineFormat.Bold)
            val state1 = awaitItem()
            assertThat(state1.actions[ComposerAction.BOLD], equalTo(ActionState.REVERSED))
            state1.toggleInlineFormat(InlineFormat.Bold)
            val state2 = awaitItem()
            assertThat(state2.actions[ComposerAction.BOLD], equalTo(ActionState.ENABLED))
            state2.toggleInlineFormat(InlineFormat.Bold)
            val state3 = awaitItem()
            assertThat(state3.actions[ComposerAction.BOLD], equalTo(ActionState.REVERSED))
            state3.toggleInlineFormat(InlineFormat.Bold)
            val state4 = awaitItem()
            assertThat(state4.actions[ComposerAction.BOLD], equalTo(ActionState.ENABLED))
        }
    }

    @Test
    fun `setHtml updates the state`() = runTest {
        moleculeFlow(RecompositionMode.Immediate) {
            val state = rememberRichTextEditorState(fake = true)
            remember(state.messageHtml, state.messageMarkdown) { state }
        }.test {
            val initialState = awaitItem()
            initialState.setHtml("<b>new html</b>")
            val nextState = awaitItem()
            assertThat(nextState.messageHtml, equalTo("<b>new html</b>"))
            assertThat(nextState.messageMarkdown, equalTo("<b>new html</b>"))
        }
    }

    @Test
    fun `replaceSuggestion updates the state`() = runTest {
        moleculeFlow(RecompositionMode.Immediate) {
            val state = rememberRichTextEditorState(initialHtml = "@ro", initialSelection = 2 to 2, fake = true)
            remember(state.messageHtml, state.messageMarkdown) { state }
        }.test {
            val initialState = awaitItem()
            initialState.replaceSuggestion("@room")
            val nextState = awaitItem()
            assertThat(nextState.messageHtml, equalTo("@room"))
        }
    }

    @Test
    fun `insertMentionAtSuggestion updates the state with a mention link`() = runTest {
        val url = "https://matrix.to/#/@user:matrix.org"
        moleculeFlow(RecompositionMode.Immediate) {
            val state = rememberRichTextEditorState(initialHtml = "@ro", initialSelection = 2 to 2, fake = true)
            remember(state.messageHtml, state.messageMarkdown) { state }
        }.test {
            val initialState = awaitItem()
            initialState.insertMentionAtSuggestion("@room", url)
            val nextState = awaitItem()
            assertThat(nextState.messageHtml, equalTo("<a href='$url'>@room</a>"))
        }
    }

    @Test
    fun `requestFocus updates the state`() = runTest {
        moleculeFlow(RecompositionMode.Immediate) {
            val state = rememberRichTextEditorState(fake = true)
            remember(state.hasFocus) { state }
        }.test {
            val initialState = awaitItem()
            assertThat(initialState.hasFocus, equalTo(false))
            initialState.requestFocus()
            val hasFocus = awaitItem().hasFocus
            assertThat(hasFocus, equalTo(true))
        }
    }
}

package io.element.android.wysiwyg.compose

import io.element.android.wysiwyg.view.models.InlineFormat
import io.element.android.wysiwyg.view.models.LinkAction
import org.hamcrest.CoreMatchers.equalTo
import org.hamcrest.MatcherAssert.assertThat
import org.junit.Test
import uniffi.wysiwyg_composer.ActionState
import uniffi.wysiwyg_composer.ComposerAction


class FakeRichTextEditorStateTest {
    private val state = RichTextEditorState(initialHtml = "", fake = true)

    @Test
    fun `toggleInlineFormat(bold) updates the state`() {
        state.toggleInlineFormat(inlineFormat = InlineFormat.Bold)
        assertThat(state.actions[ComposerAction.BOLD], equalTo(ActionState.REVERSED))
    }

    @Test
    fun `toggleInlineFormat(italic) updates the state`() {
        state.toggleInlineFormat(inlineFormat = InlineFormat.Italic)
        assertThat(state.actions[ComposerAction.ITALIC], equalTo(ActionState.REVERSED))
    }

    @Test
    fun `toggleInlineFormat(underline) updates the state`() {
        state.toggleInlineFormat(inlineFormat = InlineFormat.Underline)
        assertThat(state.actions[ComposerAction.UNDERLINE], equalTo(ActionState.REVERSED))
    }

    @Test
    fun `toggleInlineFormat(strikethrough) updates the state`() {
        state.toggleInlineFormat(inlineFormat = InlineFormat.StrikeThrough)
        assertThat(state.actions[ComposerAction.STRIKE_THROUGH], equalTo(ActionState.REVERSED))
    }

    @Test
    fun `toggleInlineFormat(inlinecode) updates the state`() {
        state.toggleInlineFormat(inlineFormat = InlineFormat.InlineCode)
        assertThat(state.actions[ComposerAction.INLINE_CODE], equalTo(ActionState.REVERSED))
    }

    @Test
    fun `toggleList(ordered) updates the state`() {
        state.toggleList(ordered = true)
        assertThat(state.actions[ComposerAction.ORDERED_LIST], equalTo(ActionState.REVERSED))
    }

    @Test
    fun `toggleList(unordered) updates the state`() {
        state.toggleList(ordered = false)
        assertThat(state.actions[ComposerAction.UNORDERED_LIST], equalTo(ActionState.REVERSED))
    }

    @Test
    fun `toggleCodeBlock updates the state`() {
        state.toggleCodeBlock()
        assertThat(state.actions[ComposerAction.CODE_BLOCK], equalTo(ActionState.REVERSED))
    }

    @Test
    fun `toggleQuote updates the state`() {
        state.toggleQuote()
        assertThat(state.actions[ComposerAction.QUOTE], equalTo(ActionState.REVERSED))
    }

    @Test
    fun `undo updates the state`() {
        state.undo()
        assertThat(state.actions[ComposerAction.UNDO], equalTo(ActionState.REVERSED))
    }

    @Test
    fun `redo updates the state`() {
        state.redo()
        assertThat(state.actions[ComposerAction.REDO], equalTo(ActionState.REVERSED))
    }

    @Test
    fun `indent updates the state`() {
        state.indent()
        assertThat(state.actions[ComposerAction.INDENT], equalTo(ActionState.REVERSED))
    }

    @Test
    fun `unindent updates the state`() {
        state.unindent()
        assertThat(state.actions[ComposerAction.UNINDENT], equalTo(ActionState.REVERSED))
    }

    @Test
    fun `setLink updates the state`() {
        state.setLink("https://element.io")
        assertThat(state.linkAction, equalTo(LinkAction.SetLink("https://element.io")))
    }

    @Test
    fun `removeLink updates the state`() {
        state.setLink("https://element.io")
        state.removeLink()
        assertThat(state.linkAction, equalTo(LinkAction.InsertLink))
    }

    @Test
    fun `insertLink updates the state`() {
        state.insertLink("https://element.io", "hello!")
        assertThat(state.linkAction, equalTo(LinkAction.SetLink("https://element.io")))
    }

    @Test
    fun `toggling multiple times toggles state`() {
        assertThat(state.actions[ComposerAction.BOLD], equalTo(null))
        state.toggleInlineFormat(InlineFormat.Bold)
        assertThat(state.actions[ComposerAction.BOLD], equalTo(ActionState.REVERSED))
        state.toggleInlineFormat(InlineFormat.Bold)
        assertThat(state.actions[ComposerAction.BOLD], equalTo(ActionState.ENABLED))
        state.toggleInlineFormat(InlineFormat.Bold)
        assertThat(state.actions[ComposerAction.BOLD], equalTo(ActionState.REVERSED))
        state.toggleInlineFormat(InlineFormat.Bold)
        assertThat(state.actions[ComposerAction.BOLD], equalTo(ActionState.ENABLED))
    }

    @Test
    fun `setHtml updates the state`() {
        state.setHtml("<b>new html</b>")
        assertThat(state.messageHtml, equalTo("<b>new html</b>"))
        assertThat(state.messageMarkdown, equalTo("<b>new html</b>"))
    }

    @Test
    fun `requestFocus updates the state`() {
        assertThat(state.hasFocus, equalTo(false))
        state.requestFocus()
        assertThat(state.hasFocus, equalTo(true))
    }
}
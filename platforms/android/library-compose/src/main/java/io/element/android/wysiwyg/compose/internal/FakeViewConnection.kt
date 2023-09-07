package io.element.android.wysiwyg.compose.internal

import io.element.android.wysiwyg.compose.RichTextEditorState
import io.element.android.wysiwyg.view.models.InlineFormat
import io.element.android.wysiwyg.view.models.LinkAction
import uniffi.wysiwyg_composer.ActionState
import uniffi.wysiwyg_composer.ComposerAction

/**
 * Fake implementation of [ViewConnection] for use in preview and test environments.
 * This implementation does not actually connect to a view, but instead updates the state
 * in _some_ way. The changes made to the state are not guaranteed to be the same as the
 * real implementation.
 */
internal class FakeViewConnection(
    val state: RichTextEditorState
) : ViewConnection {

    override fun toggleInlineFormat(inlineFormat: InlineFormat): Boolean {
        updateActionState(inlineFormat.toComposerAction())
        return true
    }

    override fun toggleList(ordered: Boolean) {
        updateActionState(
            if (ordered) {
                ComposerAction.ORDERED_LIST
            } else {
                ComposerAction.UNORDERED_LIST
            }
        )
    }

    override fun toggleCodeBlock(): Boolean {
        updateActionState(ComposerAction.CODE_BLOCK)
        return true
    }

    override fun toggleQuote(): Boolean {
        updateActionState(ComposerAction.QUOTE)
        return true
    }

    override fun undo() {
        updateActionState(ComposerAction.UNDO)
    }

    override fun redo() {
        updateActionState(ComposerAction.REDO)
    }

    override fun indent() {
        updateActionState(ComposerAction.INDENT)
    }

    override fun unindent() {
        updateActionState(ComposerAction.UNINDENT)
    }

    override fun setHtml(html: String) {
        state.messageHtml = html
        state.messageMarkdown = html
    }

    override fun requestFocus(): Boolean {
        state.hasFocus = true
        return true
    }

    override fun setLink(url: String?) {
        state.linkAction = url?.let { LinkAction.SetLink(it) } ?: LinkAction.InsertLink
    }

    override fun removeLink() {
        state.linkAction = LinkAction.InsertLink
    }

    override fun insertLink(url: String, text: String) {
        state.linkAction = LinkAction.SetLink(url)
    }

    private fun updateActionState(action: ComposerAction) {
        val actions = state.actions.toMutableMap()
        val currentState: ActionState =
            actions[action] ?: ActionState.ENABLED
        val newAction = if (currentState == ActionState.ENABLED) {
            ActionState.REVERSED
        } else {
            ActionState.ENABLED
        }
        actions[action] = newAction
        state.actions = actions
    }
}

private fun InlineFormat.toComposerAction() = when (this) {
    InlineFormat.Bold -> ComposerAction.BOLD
    InlineFormat.Italic -> ComposerAction.ITALIC
    InlineFormat.StrikeThrough -> ComposerAction.STRIKE_THROUGH
    InlineFormat.InlineCode -> ComposerAction.INLINE_CODE
    InlineFormat.Underline -> ComposerAction.UNDERLINE
}
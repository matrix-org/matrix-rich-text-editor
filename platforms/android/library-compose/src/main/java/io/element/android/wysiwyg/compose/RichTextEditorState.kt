package io.element.android.wysiwyg.compose

import androidx.compose.runtime.Composable
import androidx.compose.runtime.Stable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.saveable.Saver
import androidx.compose.runtime.saveable.SaverScope
import androidx.compose.runtime.saveable.rememberSaveable
import androidx.compose.runtime.setValue
import io.element.android.wysiwyg.compose.internal.ViewConnection
import io.element.android.wysiwyg.view.models.InlineFormat
import uniffi.wysiwyg_composer.ActionState
import uniffi.wysiwyg_composer.ComposerAction
import uniffi.wysiwyg_composer.MenuAction

/**
 * A state holder for the [RichTextEditor] composable.
 *
 * Create an instance using [rememberRichTextEditorState].
 * Ensure that [RichTextEditorState] is not shared between
 * multiple [RichTextEditor] composables.
 */
@Stable
class RichTextEditorState internal constructor() {
    internal var viewConnection: ViewConnection? = null

    /**
     * Toggle inline formatting on the current selection.
     *
     * @param inlineFormat which format to toggle (e.g. [InlineFormat.Bold])
     */
    fun toggleInlineFormat(inlineFormat: InlineFormat) {
        viewConnection?.toggleInlineFormat(inlineFormat)
    }

    /**
     * Undo the last action.
     */
    fun undo() {
        viewConnection?.undo()
    }

    /**
     * Redo the last undone action.
     */
    fun redo() {
        viewConnection?.redo()
    }

    /**
     * Toggle list formatting on the current selection.
     *
     * @param ordered Whether the list should be ordered (numbered) or unordered (bulleted).
     */
    fun toggleList(ordered: Boolean) {
        viewConnection?.toggleList(ordered)
    }

    /**
     * Indent the current selection.
     */
    fun indent() {
        viewConnection?.indent()
    }

    /**
     * Unindent the current selection.
     */
    fun unindent() {
        viewConnection?.unindent()
    }

    /**
     * Toggle code block formatting on the current selection.
     */
    fun toggleCodeBlock() {
        viewConnection?.toggleCodeBlock()
    }

    /**
     * Toggle quote formatting on the current selection.
     */
    fun toggleQuote() {
        viewConnection?.toggleQuote()
    }

    /**
     * Set the HTML content of the editor.
     */
    fun setHtml(html: String) = viewConnection?.setHtml(html)

    /**
     * The content of the editor as HTML formatted for sending as a message.
     */
    var messageHtml by mutableStateOf("")
        internal set

    /**
     * The content of the editor as markdown formatted for sending as a message.
     */
    var messageMarkdown by mutableStateOf("")
        internal set

    /**
     * The current action states of the editor.
     */
    var actions by mutableStateOf(emptyMap<ComposerAction, ActionState>())
        internal set

    /**
     * The current selection of the editor.
     */
    var selection by mutableStateOf(0 to 0)
        internal set

    /**
     * The current menu action of the editor.
     */
    var menuAction: MenuAction by mutableStateOf(MenuAction.None)
        internal set
}

/**
 * Create an instance of the [RichTextEditorState].
 */
@Composable
fun rememberRichTextEditorState(): RichTextEditorState =
    rememberSaveable(saver = RichTextEditorStateSaver) {
        RichTextEditorState()
    }

object RichTextEditorStateSaver : Saver<RichTextEditorState, String> {
    override fun restore(value: String): RichTextEditorState {
        return RichTextEditorState().apply {
            messageHtml = value
        }
    }

    override fun SaverScope.save(value: RichTextEditorState): String {
        return value.messageHtml
    }
}

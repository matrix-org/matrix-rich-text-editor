package io.element.android.wysiwyg.compose

import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.saveable.Saver
import androidx.compose.runtime.saveable.SaverScope
import androidx.compose.runtime.saveable.rememberSaveable
import androidx.compose.runtime.setValue
import androidx.compose.ui.platform.LocalInspectionMode
import io.element.android.wysiwyg.compose.internal.FakeViewConnection
import io.element.android.wysiwyg.compose.internal.ViewConnection
import io.element.android.wysiwyg.view.models.InlineFormat
import io.element.android.wysiwyg.view.models.LinkAction
import uniffi.wysiwyg_composer.ActionState
import uniffi.wysiwyg_composer.ComposerAction
import uniffi.wysiwyg_composer.MenuAction

/**
 * A state holder for the [RichTextEditor] composable.
 *
 * Create an instance using [rememberRichTextEditorState].
 * Ensure that [RichTextEditorState] is not shared between multiple [RichTextEditor] composables.
 *
 * Note that fake mode is only intended for use in preview or test environments and behaviour will
 * not mirror that of the real editor.
 *
 * @param initialHtml The HTML formatted content to initialise the state with.
 * @param fake If true, initialise the state for use in preview or test environment.
 */
class RichTextEditorState(
    initialHtml: String = "",
    fake: Boolean = false,
) {
    internal var viewConnection: ViewConnection? by mutableStateOf(null)

    init {
        if (fake) {
            viewConnection = FakeViewConnection(this)
        }
    }

    private val initialLineCount = if (fake) {
        initialHtml.count { it == '\n' } + 1
    } else {
        1
    }

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
    fun setHtml(html: String) {
        viewConnection?.setHtml(html)
    }

    /**
     * Set a link for the current selection. This method does nothing if there is no text selected.
     *
     * @param url The link URL to set or null to remove
     */
    fun setLink(url: String?) = viewConnection?.setLink(url)

    /**
     * Remove a link for the current selection. Convenience for setLink(null).
     *
     * @see [setLink]
     */
    fun removeLink() = viewConnection?.removeLink()

    /**
     * Insert new text with a link.
     *
     * @param url The link URL to set
     * @param text The new text to insert
     */
    fun insertLink(url: String, text: String) = viewConnection?.insertLink(url, text)

    /**
     * The content of the editor as HTML formatted for sending as a message.
     */
    var messageHtml by mutableStateOf(initialHtml)
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

    /**
     * Whether the editor input field currently has focus.
     */
    var hasFocus: Boolean by mutableStateOf(false)
        internal set

    /**
     * Request focus of the editor input field.
     */
    fun requestFocus(): Boolean =
        viewConnection?.requestFocus() ?: false

    /**
     * The number of lines displayed in the editor.
     */
    var lineCount: Int by mutableStateOf(initialLineCount)
        internal set

    var linkAction: LinkAction? by mutableStateOf(null)
        internal set

}

/**
 * Create an instance of the [RichTextEditorState].
 *
 * Note that fake mode is only intended for use in preview or test environments and behaviour will
 * not mirror that of the real editor.
 *
 * @param initialHtml The HTML formatted content to initialise the state with.
 * @param fake If true, initialise the state for use in preview or test environment.
 */
@Composable
fun rememberRichTextEditorState(
    initialHtml: String = "",
    fake: Boolean = LocalInspectionMode.current,
): RichTextEditorState {
    return rememberSaveable(saver = RichTextEditorStateSaver) {
        RichTextEditorState(
            initialHtml = initialHtml,
            fake = fake,
        )
    }
}

object RichTextEditorStateSaver : Saver<RichTextEditorState, String> {
    override fun restore(value: String): RichTextEditorState {
        return RichTextEditorState(initialHtml = value)
    }

    override fun SaverScope.save(value: RichTextEditorState): String {
        return value.messageHtml
    }
}

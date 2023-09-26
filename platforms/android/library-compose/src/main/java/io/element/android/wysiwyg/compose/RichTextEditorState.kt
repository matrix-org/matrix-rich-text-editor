package io.element.android.wysiwyg.compose

import androidx.compose.runtime.Composable
import androidx.compose.runtime.Stable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableIntStateOf
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.rememberCoroutineScope
import androidx.compose.runtime.saveable.Saver
import androidx.compose.runtime.saveable.SaverScope
import androidx.compose.runtime.saveable.rememberSaveable
import androidx.compose.runtime.setValue
import io.element.android.wysiwyg.compose.internal.FakeViewConnection
import io.element.android.wysiwyg.compose.internal.ViewAction
import io.element.android.wysiwyg.view.models.InlineFormat
import io.element.android.wysiwyg.view.models.LinkAction
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.MutableSharedFlow
import kotlinx.coroutines.launch
import uniffi.wysiwyg_composer.ActionState
import uniffi.wysiwyg_composer.ComposerAction
import uniffi.wysiwyg_composer.MenuAction

/**
 * A state holder for the [RichTextEditor] composable.
 *
 * Create an instance using [rememberRichTextEditorState].
 * Ensure that [RichTextEditorState] is not shared between multiple [RichTextEditor] composables
 * that are displayed at the same time.
 *
 * @param initialHtml The HTML formatted content to initialise the state with.
 * @param initialLineCount The line count to initialise the state with.
 * @param initialFocus The focus value to initialise the state with.
 */
@Stable
class RichTextEditorState(
    initialHtml: String = RichTextEditorDefaults.initialHtml,
    initialLineCount: Int = RichTextEditorDefaults.initialLineCount,
    initialFocus: Boolean = RichTextEditorDefaults.initialFocus,
) {
    // A unique key for the most recent view to subscribe
    internal var activeViewKey: Any? by mutableStateOf(-1)

    private val _viewActions = MutableSharedFlow<ViewAction>()
    internal val viewActions: Flow<ViewAction> = _viewActions

    /**
     * Toggle inline formatting on the current selection.
     *
     * @param inlineFormat which format to toggle (e.g. [InlineFormat.Bold])
     */
    suspend fun toggleInlineFormat(inlineFormat: InlineFormat) {
        _viewActions.emit(ViewAction.ToggleInlineFormat(inlineFormat))
    }

    /**
     * Undo the last action.
     */
    suspend fun undo() {
        _viewActions.emit(ViewAction.Undo)
    }

    /**
     * Redo the last undone action.
     */
    suspend fun redo() {
        _viewActions.emit(ViewAction.Redo)
    }

    /**
     * Toggle list formatting on the current selection.
     *
     * @param ordered Whether the list should be ordered (numbered) or unordered (bulleted).
     */
    suspend fun toggleList(ordered: Boolean) {
        _viewActions.emit(ViewAction.ToggleList(ordered))
    }

    /**
     * Indent the current selection.
     */
    suspend fun indent() {
        _viewActions.emit(ViewAction.Indent)
    }

    /**
     * Unindent the current selection.
     */
    suspend fun unindent() {
        _viewActions.emit(ViewAction.Unindent)
    }

    /**
     * Toggle code block formatting on the current selection.
     */
    suspend fun toggleCodeBlock() {
        _viewActions.emit(ViewAction.ToggleCodeBlock)
    }

    /**
     * Toggle quote formatting on the current selection.
     */
    suspend fun toggleQuote() {
        _viewActions.emit(ViewAction.ToggleQuote)
    }

    /**
     * Set the HTML content of the editor.
     */
    suspend fun setHtml(html: String) {
        _viewActions.emit(ViewAction.SetHtml(html))
    }

    /**
     * Set a link for the current selection. This method does nothing if there is no text selected.
     *
     * @param url The link URL to set or null to remove
     */
    suspend fun setLink(url: String?) {
        _viewActions.emit(ViewAction.SetLink(url))
    }

    /**
     * Remove a link for the current selection. Convenience for setLink(null).
     *
     * @see [setLink]
     */
    suspend fun removeLink() {
        _viewActions.emit(ViewAction.RemoveLink)
    }

    /**
     * Insert new text with a link.
     *
     * @param url The link URL to set
     * @param text The new text to insert
     */
    suspend fun insertLink(url: String, text: String) {
        _viewActions.emit(ViewAction.InsertLink(url, text))
    }

    /**
     * The content of the editor as HTML formatted for sending as a message.
     */
    var messageHtml by mutableStateOf(initialHtml)
        internal set

    /**
     * The content of the editor as represented internally.
     *
     * Can be used to restore the editor state.
     */
    internal var internalHtml by mutableStateOf(initialHtml)
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
    var hasFocus: Boolean by mutableStateOf(initialFocus)
        internal set

    /**
     * Request focus of the editor input field.
     */
    suspend fun requestFocus() {
        _viewActions.emit(ViewAction.RequestFocus)
    }

    /**
     * Notify the state that focus has changed.
     *
     * Ignores the event if the view key does not match the current active view.
     */
    internal fun onFocusChanged(viewKey: Any, hasFocus: Boolean) {
        if (viewKey != activeViewKey) {
            return
        }
        this.hasFocus = hasFocus
    }

    /**
     * The number of lines displayed in the editor.
     */
    var lineCount: Int by mutableIntStateOf(initialLineCount)
        internal set

    var linkAction: LinkAction? by mutableStateOf(null)
        internal set

}

/**
 * Create an instance of the [RichTextEditorState].
 *
 * Initial values can be provided for preview or test environments. [RichTextEditor] will
 * overwrite these values unless configured not to.
 *
 * @param initialHtml The HTML formatted content to initialise the state with.
 * @param initialLineCount The line count to initialise the state with.
 * @param initialFocus The value of hasFocus to initialise the state with.
 */
@Composable
fun rememberRichTextEditorState(
    initialHtml: String = RichTextEditorDefaults.initialHtml,
    initialLineCount: Int = RichTextEditorDefaults.initialLineCount,
    initialFocus: Boolean = RichTextEditorDefaults.initialFocus,
    fake: Boolean = false,
): RichTextEditorState {
    val state = rememberSaveable(saver = RichTextEditorStateSaver) {
        RichTextEditorState(
            initialHtml = initialHtml,
            initialLineCount = initialLineCount,
            initialFocus = initialFocus,
        )

    }

    if (fake) {
        FakeViewConnection(state)
    }

    return state
}

object RichTextEditorStateSaver : Saver<RichTextEditorState, String> {
    override fun restore(value: String): RichTextEditorState {
        return RichTextEditorState(initialHtml = value)
    }

    override fun SaverScope.save(value: RichTextEditorState): String {
        return value.internalHtml
    }
}

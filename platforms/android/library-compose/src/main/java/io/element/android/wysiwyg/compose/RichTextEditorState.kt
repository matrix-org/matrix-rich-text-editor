package io.element.android.wysiwyg.compose

import androidx.compose.runtime.Composable
import androidx.compose.runtime.Stable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.platform.LocalContext
import io.element.android.wysiwyg.EditorEditText
import io.element.android.wysiwyg.view.models.InlineFormat
import uniffi.wysiwyg_composer.ActionState
import uniffi.wysiwyg_composer.ComposerAction
import uniffi.wysiwyg_composer.MenuAction

/**
 * A state holder for the [RichTextEditor] composable.
 */
@Stable
class RichTextEditorState internal constructor(
    internal val view: EditorEditText,
) {

    /**
     * Toggle bold formatting on the current selection.
     */
    fun toggleBold() = view.toggleInlineFormat(inlineFormat = InlineFormat.Bold)

    /**
     * Toggle italic formatting on the current selection.
     */
    fun toggleItalic() = view.toggleInlineFormat(inlineFormat = InlineFormat.Italic)

    /**
     * Set the HTML content of the editor.
     */
    fun setHtml(html: String) = view.setHtml(html)

    /**
     * The content of the editor as HTML formatted for sending as a message.
     */
    var messageHtml by mutableStateOf(view.getContentAsMessageHtml())
        internal set

    /**
     * The content of the editor as markdown formatted for sending as a message.
     */
    var messageMarkdown by mutableStateOf(view.getMarkdown())
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
fun rememberRichTextEditorState(): RichTextEditorState {
    val context = LocalContext.current

    return remember {
        RichTextEditorState(view = EditorEditText(context))
    }
}


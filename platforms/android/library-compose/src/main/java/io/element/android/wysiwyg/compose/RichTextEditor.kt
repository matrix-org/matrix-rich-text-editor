package io.element.android.wysiwyg.compose

import androidx.compose.runtime.Composable
import androidx.compose.runtime.DisposableEffect
import androidx.compose.ui.Modifier
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.viewinterop.AndroidView
import androidx.core.widget.addTextChangedListener
import io.element.android.wysiwyg.EditorEditText
import io.element.android.wysiwyg.compose.internal.ViewConnection
import io.element.android.wysiwyg.view.models.InlineFormat

/**
 * A composable rich text editor.
 *
 * This composable is a wrapper around the [EditorEditText] view.
 *
 * @param state The state holder for this composable. See [rememberRichTextEditorState].
 * @param modifier The modifier for the layout
 */
@Composable
fun RichTextEditor(
    state: RichTextEditorState,
    modifier: Modifier = Modifier,
) {
    val context = LocalContext.current

    // Clean up the connection between view and state holder
    DisposableEffect(Unit) {
        onDispose {
            state.viewConnection = null
        }
    }

    AndroidView(
        modifier = modifier,
        factory = {
            if (state.viewConnection != null) {
                throw IllegalStateException(
                    "Instance of RichTextEditorState is already set up with another RichTextEditor."
                )
            }

            val view = EditorEditText(context).apply {
                actionStatesChangedListener =
                    EditorEditText.OnActionStatesChangedListener { actionStates ->
                        state.actions = actionStates
                    }

                selectionChangeListener =
                    EditorEditText.OnSelectionChangeListener { start, end ->
                        state.selection = start to end
                    }
                menuActionListener = EditorEditText.OnMenuActionChangedListener { menuAction ->
                    state.menuAction = menuAction
                }

                addTextChangedListener {
                    state.messageHtml = getContentAsMessageHtml()
                    state.messageMarkdown = getMarkdown()
                }

                // Restore the state of the view with the saved state
                setHtml(state.messageHtml)
            }

            state.viewConnection = object : ViewConnection {
                override fun toggleInlineFormat(inlineFormat: InlineFormat) =
                    view.toggleInlineFormat(inlineFormat)

                override fun undo() = view.undo()

                override fun redo() = view.redo()

                override fun toggleList(ordered: Boolean) =
                    view.toggleList(ordered)

                override fun indent() = view.indent()

                override fun unindent() = view.unindent()

                override fun toggleCodeBlock() = view.toggleCodeBlock()

                override fun toggleQuote() = view.toggleQuote()

                override fun setHtml(html: String) = view.setHtml(html)
            }

            view
        },
    )
}

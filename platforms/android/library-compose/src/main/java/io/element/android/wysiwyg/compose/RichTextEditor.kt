package io.element.android.wysiwyg.compose

import android.os.Build
import android.view.View
import androidx.appcompat.widget.AppCompatEditText
import androidx.compose.runtime.Composable
import androidx.compose.runtime.DisposableEffect
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.toArgb
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.platform.LocalInspectionMode
import androidx.compose.ui.viewinterop.AndroidView
import androidx.core.content.ContextCompat
import androidx.core.widget.addTextChangedListener
import io.element.android.wysiwyg.EditorEditText
import io.element.android.wysiwyg.compose.internal.ViewConnection
import io.element.android.wysiwyg.compose.internal.toStyleConfig
import io.element.android.wysiwyg.utils.RustErrorCollector
import io.element.android.wysiwyg.view.models.InlineFormat

/**
 * A composable rich text editor.
 *
 * This composable is a wrapper around the [EditorEditText] view.
 *
 * @param state The state holder for this composable. See [rememberRichTextEditorState].
 * @param modifier The modifier for the layout
 * @param style The styles to use for any customisable elements
 * @param onError Called when an internal error occurs
 */
@Composable
fun RichTextEditor(
    state: RichTextEditorState,
    modifier: Modifier = Modifier,
    style: RichTextEditorStyle = RichTextEditorDefaults.style(),
    onError: (Throwable) -> Unit = {},
) {
    val isPreview = LocalInspectionMode.current

    if (isPreview) {
        PreviewEditor(state, modifier, style)
    } else {
        RealEditor(state, modifier, style, onError)
    }
}

@Composable
private fun RealEditor(
    state: RichTextEditorState,
    modifier: Modifier = Modifier,
    style: RichTextEditorStyle = RichTextEditorDefaults.style(),
    onError: (Throwable) -> Unit = {},
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
                onFocusChangeListener =
                    View.OnFocusChangeListener { _, hasFocus -> state.hasFocus = hasFocus }

                addTextChangedListener {
                    state.messageHtml = getContentAsMessageHtml()
                    state.messageMarkdown = getMarkdown()
                    state.lineCount = lineCount
                }

                applyDefaultStyle()

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

                override fun requestFocus() = view.requestFocus()
            }

            view
        },
        update = { view ->
            view.setStyleConfig(style.toStyleConfig(view.context))
            view.applyStyle(style)
            view.rustErrorCollector = RustErrorCollector(onError)
        }
    )
}

@Composable
private fun PreviewEditor(
    state: RichTextEditorState,
    modifier: Modifier = Modifier,
    style: RichTextEditorStyle = RichTextEditorDefaults.style(),
) {
    if (!LocalInspectionMode.current) {
        throw IllegalStateException("PreviewEditor should only be used in preview mode")
    }

    val context = LocalContext.current

    AndroidView(
        modifier = modifier,
        factory = {
            val view = AppCompatEditText(context).apply {
                applyDefaultStyle()

                setText(state.messageHtml)
            }

            view
        },
        update = { view ->
            view.applyStyle(style)
        }
    )
}

private fun AppCompatEditText.applyStyle(style: RichTextEditorStyle) {
    setTextColor(style.text.color.toArgb())
    if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.Q) {
        val cursorDrawable = ContextCompat.getDrawable(context, R.drawable.cursor)
        cursorDrawable?.setTint(style.cursor.color.toArgb())
        textCursorDrawable = cursorDrawable
    }
}

private fun AppCompatEditText.applyDefaultStyle() {
    // Set the style closer to a BasicTextField composable
    setBackgroundDrawable(null)
    setPadding(0, 0, 0, 0)
}
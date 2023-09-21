package io.element.android.wysiwyg.compose

import android.os.Build
import android.view.View
import androidx.appcompat.widget.AppCompatEditText
import androidx.compose.runtime.Composable
import androidx.compose.runtime.derivedStateOf
import androidx.compose.runtime.getValue
import androidx.compose.runtime.remember
import androidx.compose.runtime.rememberCoroutineScope
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.toArgb
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.platform.LocalInspectionMode
import androidx.compose.ui.viewinterop.AndroidView
import androidx.core.content.ContextCompat
import androidx.core.widget.addTextChangedListener
import io.element.android.wysiwyg.EditorEditText
import io.element.android.wysiwyg.compose.internal.ViewAction
import io.element.android.wysiwyg.compose.internal.toStyleConfig
import io.element.android.wysiwyg.utils.RustErrorCollector
import kotlinx.coroutines.android.awaitFrame
import kotlinx.coroutines.launch
import timber.log.Timber


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
    subcomposing: Boolean = false,
    modifier: Modifier = Modifier,
    style: RichTextEditorStyle = RichTextEditorDefaults.style(),
    onError: (Throwable) -> Unit = {},
) {
    val isPreview = LocalInspectionMode.current

    if (isPreview) {
        PreviewEditor(state, modifier, style)
    } else {
        RealEditor(state, subcomposing, modifier, style, onError)
    }
}

@Composable
private fun RealEditor(
    state: RichTextEditorState,
    subcomposing: Boolean,
    modifier: Modifier = Modifier,
    style: RichTextEditorStyle = RichTextEditorDefaults.style(),
    onError: (Throwable) -> Unit,
) {
    val context = LocalContext.current
    val coroutineScope = rememberCoroutineScope()

    Timber.i("RTE: recomposing (subcomposing=$subcomposing)")

    val onFocusChangeListener: (viewHash: Int, hasFocus: Boolean) -> Unit by remember(state.curActiveViewHash) {
        derivedStateOf {
            listener@ { viewHash: Int, hasFocus: Boolean ->
                if (viewHash != state.curActiveViewHash) {
                    return@listener
                }

                state.hasFocus = hasFocus
            }
        }
    }

    AndroidView(
        modifier = modifier,
        factory = {
            val view = EditorEditText(context).apply {
                Timber.i("RTE: factory creating new view ${this.hashCode()} (subcomposing=$subcomposing)")
                if(!subcomposing) {
                    state.curActiveViewHash = hashCode()
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
                    linkActionChangedListener =
                        EditorEditText.OnLinkActionChangedListener { linkAction ->
                            state.linkAction = linkAction
                        }
                    addTextChangedListener {
                        // Todo combine into a single mutable state
                        state.internalHtml = getInternalHtml()
                        state.messageHtml = getContentAsMessageHtml()
                        state.messageMarkdown = getMarkdown()
                        state.lineCount = lineCount
                    }
                    val shouldRestoreFocus = state.hasFocus
                    if(shouldRestoreFocus) {
                        Timber.i("RTE: ${this@apply.hashCode()} Needs to restore focus")

                        requestFocus()
                    }
                }

                applyDefaultStyle()

                // Restore the state of the view with the saved state
                setHtml(state.internalHtml)

                // Only start listening for text changes after the initial state has been restored
                if (!subcomposing) {
                    coroutineScope.launch {
                        state.viewActions.collect {
                            when (it) {
                                is ViewAction.ToggleInlineFormat -> toggleInlineFormat(it.inlineFormat)
                                is ViewAction.ToggleList -> toggleList(it.ordered)
                                is ViewAction.ToggleCodeBlock -> toggleCodeBlock()
                                is ViewAction.ToggleQuote -> toggleQuote()
                                is ViewAction.Undo -> undo()
                                is ViewAction.Redo -> redo()
                                is ViewAction.Indent -> indent()
                                is ViewAction.Unindent -> unindent()
                                is ViewAction.SetHtml -> setHtml(it.html)
                                is ViewAction.RequestFocus -> {
                                    requestFocus()
                                }
                                is ViewAction.SetLink -> setLink(it.url)
                                is ViewAction.RemoveLink -> removeLink()
                                is ViewAction.InsertLink -> insertLink(it.url, it.text)
                            }
                        }
                    }
                }
            }

            view
        },
        update = { view ->
            if (view.onFocusChangeListener == null) {
                view.onFocusChangeListener = View.OnFocusChangeListener { view, hasFocus ->
                    onFocusChangeListener(view.hashCode(), hasFocus)
                }
            }
            Timber.i("RTE: update style (subcomposing=$subcomposing)")
            view.setStyleConfig(style.toStyleConfig(view.context))
            view.applyStyle(style)
            view.rustErrorCollector = RustErrorCollector(onError)
        },
//        onReset = {
//            Timber.i("RTE: reset ${it.hashCode()} (subcomposing=$subcomposing)")
//            it.onFocusChangeListener = null
//        },
//        onRelease = {
//            Timber.i("RTE: release ${it.hashCode()} (subcomposing=$subcomposing)")
//            it.onFocusChangeListener = null
//        }
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
        setLinkTextColor(style.link.color.toArgb())
    }
}

private fun AppCompatEditText.applyDefaultStyle() {
    // Set the style closer to a BasicTextField composable
    setBackgroundDrawable(null)
    setPadding(0, 0, 0, 0)
}
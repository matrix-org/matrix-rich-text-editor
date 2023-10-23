package io.element.android.wysiwyg.compose

import android.view.View
import androidx.appcompat.widget.AppCompatEditText
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.rememberCoroutineScope
import androidx.compose.runtime.setValue
import androidx.compose.ui.Modifier
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.platform.LocalInspectionMode
import androidx.compose.ui.viewinterop.AndroidView
import androidx.core.widget.addTextChangedListener
import io.element.android.wysiwyg.EditorEditText
import io.element.android.wysiwyg.compose.internal.ViewAction
import io.element.android.wysiwyg.compose.internal.applyDefaultStyle
import io.element.android.wysiwyg.compose.internal.applyStyle
import io.element.android.wysiwyg.compose.internal.rememberTypeface
import io.element.android.wysiwyg.compose.internal.toStyleConfig
import io.element.android.wysiwyg.display.MentionDisplayHandler
import io.element.android.wysiwyg.utils.RustErrorCollector
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.launch


/**
 * A composable rich text editor.
 *
 * This composable is a wrapper around the [EditorEditText] view.
 *
 * To use within a subcomposition, set the [registerStateUpdates] parameter to false.
 *
 * @param modifier The modifier for the layout
 * @param state The state holder for this composable. See [rememberRichTextEditorState].
 * @param registerStateUpdates If true, register the state for updates.
 * @param style The styles to use for any customisable elements
 * @param onError Called when an internal error occurs
 */
@Composable
fun RichTextEditor(
    modifier: Modifier = Modifier,
    state: RichTextEditorState = rememberRichTextEditorState(),
    registerStateUpdates: Boolean = true,
    style: RichTextEditorStyle = RichTextEditorDefaults.style(),
    mentionDisplayHandler: MentionDisplayHandler? = null,
    onError: (Throwable) -> Unit = {},
) {
    val isPreview = LocalInspectionMode.current

    if (isPreview) {
        PreviewEditor(state, modifier, style)
    } else {
        RealEditor(state, registerStateUpdates, modifier, style, onError, mentionDisplayHandler)
    }
}

@Composable
private fun RealEditor(
    state: RichTextEditorState,
    registerStateUpdates: Boolean,
    modifier: Modifier = Modifier,
    style: RichTextEditorStyle,
    onError: (Throwable) -> Unit,
    mentionDisplayHandler: MentionDisplayHandler?,
) {
    val context = LocalContext.current
    val coroutineScope = rememberCoroutineScope()
    val textStyleTypeface = style.text.rememberTypeface()

    var previousStyle by remember { mutableStateOf<RichTextEditorStyle?>(null) }

    AndroidView(modifier = modifier, factory = {
        val view = EditorEditText(context).apply {
            if (registerStateUpdates) {
                state.activeViewKey = hashCode()
                actionStatesChangedListener =
                    EditorEditText.OnActionStatesChangedListener { actionStates ->
                        state.actions = actionStates
                    }

                selectionChangeListener = EditorEditText.OnSelectionChangeListener { start, end ->
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
                    state.internalHtml = getInternalHtml()
                    state.messageHtml = getContentAsMessageHtml()
                    state.messageMarkdown = getMarkdown()
                    state.lineCount = lineCount
                }
                val shouldRestoreFocus = state.hasFocus
                if (shouldRestoreFocus) {
                    requestFocus()
                }
                onFocusChangeListener = View.OnFocusChangeListener { view, hasFocus ->
                    state.onFocusChanged(view.hashCode(), hasFocus)
                }
            }

            applyDefaultStyle()

            // Restore the state of the view with the saved state
            setHtml(state.internalHtml)
            setSelection(state.selection.first, state.selection.second)

            // Only start listening for text changes after the initial state has been restored
            if (registerStateUpdates) {
                coroutineScope.launch(context = Dispatchers.Main) {
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
                            is ViewAction.RequestFocus -> requestFocus()
                            is ViewAction.SetLink -> setLink(it.url)
                            is ViewAction.RemoveLink -> removeLink()
                            is ViewAction.InsertLink -> insertLink(it.url, it.text)
                            is ViewAction.ReplaceSuggestionText -> replaceTextSuggestion(it.text)
                            is ViewAction.SetSuggestionLink -> setLinkSuggestion(url = it.url, text = it.text)
                        }
                    }
                }
            }
        }

        view
    }, update = { view ->
        if (style != previousStyle) {
            view.setStyleConfig(style.toStyleConfig(view.context))
            view.applyStyle(style)
            previousStyle = style
        }
        view.typeface = textStyleTypeface
        view.rustErrorCollector = RustErrorCollector(onError)
        view.mentionDisplayHandler = mentionDisplayHandler
    })
}

@Composable
private fun PreviewEditor(
    state: RichTextEditorState,
    modifier: Modifier = Modifier,
    style: RichTextEditorStyle,
) {
    if (!LocalInspectionMode.current) {
        throw IllegalStateException("PreviewEditor should only be used in preview mode")
    }

    val context = LocalContext.current

    AndroidView(modifier = modifier, factory = {
        val view = AppCompatEditText(context).apply {
            applyDefaultStyle()

            setText(state.messageHtml)
        }

        view
    }, update = { view ->
        view.applyStyle(style)
    })
}

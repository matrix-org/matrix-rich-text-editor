package io.element.android.wysiwyg.compose

import android.net.Uri
import android.view.View
import androidx.appcompat.widget.AppCompatEditText
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.remember
import androidx.compose.runtime.rememberCoroutineScope
import androidx.compose.ui.Modifier
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.platform.LocalInspectionMode
import androidx.compose.ui.viewinterop.AndroidView
import androidx.core.widget.addTextChangedListener
import io.element.android.wysiwyg.EditorEditText
import io.element.android.wysiwyg.compose.internal.ViewAction
import io.element.android.wysiwyg.compose.internal.applyDefaultStyle
import io.element.android.wysiwyg.compose.internal.applyStyleInCompose
import io.element.android.wysiwyg.compose.internal.rememberTypeface
import io.element.android.wysiwyg.compose.internal.toStyleConfig
import io.element.android.wysiwyg.display.MentionDisplayHandler
import io.element.android.wysiwyg.display.TextDisplay
import io.element.android.wysiwyg.utils.RustErrorCollector
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.flow.onStart
import kotlinx.coroutines.launch
import timber.log.Timber


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
 * @param inputType The input type for the editor. Defaults to [RichTextEditorDefaults.inputType].
 * @param resolveMentionDisplay A function to resolve the [TextDisplay] of a mention.
 * @param resolveRoomMentionDisplay A function to resolve the [TextDisplay] of an `@room` mention.
 * @param onTyping Called when the user starts or stops typing in the editor.
 * @param onError Called when an internal error occurs
 * @param onRichContentSelected Called when user uses the keyboard to send a rich content
 */
@Composable
fun RichTextEditor(
    modifier: Modifier = Modifier,
    state: RichTextEditorState = rememberRichTextEditorState(),
    registerStateUpdates: Boolean = true,
    style: RichTextEditorStyle = RichTextEditorDefaults.style(),
    inputType: Int = RichTextEditorDefaults.inputType,
    resolveMentionDisplay: (text: String, url: String) -> TextDisplay = RichTextEditorDefaults.MentionDisplay,
    resolveRoomMentionDisplay: () -> TextDisplay = RichTextEditorDefaults.RoomMentionDisplay,
    onTyping: (Boolean) -> Unit = {},
    onError: (Throwable) -> Unit = {},
    onRichContentSelected: ((Uri) -> Unit)? = null,
) {
    val isPreview = LocalInspectionMode.current

    if (isPreview) {
        PreviewEditor(state, modifier, style)
    } else {
        RealEditor(
            state = state,
            registerStateUpdates = registerStateUpdates,
            modifier = modifier,
            style = style,
            inputType = inputType,
            onError = onError,
            resolveMentionDisplay = resolveMentionDisplay,
            resolveRoomMentionDisplay = resolveRoomMentionDisplay,
            onTyping = onTyping,
            onRichContentSelected = onRichContentSelected,
        )
    }
}

@Composable
private fun RealEditor(
    state: RichTextEditorState,
    registerStateUpdates: Boolean,
    modifier: Modifier = Modifier,
    style: RichTextEditorStyle,
    inputType: Int,
    onError: (Throwable) -> Unit,
    resolveMentionDisplay: (text: String, url: String) -> TextDisplay,
    resolveRoomMentionDisplay: () -> TextDisplay,
    onTyping: (Boolean) -> Unit,
    onRichContentSelected: ((Uri) -> Unit)?,
) {
    val context = LocalContext.current
    val coroutineScope = rememberCoroutineScope()

    val typeface by style.text.rememberTypeface()

    val mentionDisplayHandler = remember(resolveMentionDisplay, resolveRoomMentionDisplay) {
        object : MentionDisplayHandler {
            override fun resolveMentionDisplay(text: String, url: String): TextDisplay {
                return resolveMentionDisplay(text, url)
            }

            override fun resolveAtRoomMentionDisplay(): TextDisplay {
                return resolveRoomMentionDisplay()
            }
        }
    }

    AndroidView(
        modifier = modifier,
        factory = {
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
                        state.messageMarkdown = getContentAsMessageMarkdown()

                        onTyping(state.internalHtml.isNotEmpty())

                        // Prevent the line count from being reset when the text Layout is not set
                        if (lineCount > 0) {
                            state.lineCount = lineCount
                        }
                    }
                    val shouldRestoreFocus = state.hasFocus
                    if (shouldRestoreFocus) {
                        performClick()
                    }
                    onFocusChangeListener = View.OnFocusChangeListener { view, hasFocus ->
                        state.onFocusChanged(view.hashCode(), hasFocus)
                    }

                    mentionsStateChangedListener = EditorEditText.OnMentionsStateChangedListener { mentionsState ->
                        state.mentionsState = mentionsState
                    }
                }

                applyDefaultStyle()

                // Set initial HTML and selection based on the provided state
                setHtml(state.internalHtml)
                setSelection(state.selection.first, state.selection.second)

                setOnRichContentSelected(onRichContentSelected)
                // Only start listening for text changes after the initial state has been restored
                if (registerStateUpdates) {
                    coroutineScope.launch(context = Dispatchers.Main) {
                        state.viewActions
                            .onStart { state.isReadyToProcessActions = true }
                            .collect {
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
                                    is ViewAction.SetMarkdown -> setMarkdown(it.markdown)
                                    is ViewAction.RequestFocus -> state.hasFocus = true
                                    is ViewAction.SetLink -> setLink(it.url)
                                    is ViewAction.RemoveLink -> removeLink()
                                    is ViewAction.InsertLink -> insertLink(it.url, it.text)
                                    is ViewAction.ReplaceSuggestionText -> replaceTextSuggestion(it.text)
                                    is ViewAction.InsertMentionAtSuggestion -> insertMentionAtSuggestion(url = it.url, text = it.text)
                                    is ViewAction.InsertAtRoomMentionAtSuggestion -> insertAtRoomMentionAtSuggestion()
                                    is ViewAction.SetSelection -> setSelection(it.start, it.end)
                                }
                            }
                    }
                }
            }
            view
        },
        update = { view ->
            Timber.i("RichTextEditor update() called")
            if (inputType != view.inputType) { view.inputType = inputType }
            view.applyStyleInCompose(style)
            view.typeface = typeface
            view.updateStyle(style.toStyleConfig(view.context), mentionDisplayHandler)
            view.rustErrorCollector = RustErrorCollector(onError)

            if (registerStateUpdates && state.hasFocus && !view.hasFocus()) {
                state.hasFocus = view.requestFocus()
            }
        },
        onRelease = {
            if (registerStateUpdates) {
                state.onRelease()
            }
        }
    )
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
        view.applyStyleInCompose(style)
    })
}

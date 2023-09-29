package io.element.android.wysiwyg.compose

import android.os.Build
import android.util.TypedValue
import android.view.ActionMode
import android.view.Menu
import android.view.MenuItem
import android.view.View
import androidx.appcompat.widget.AppCompatEditText
import androidx.compose.runtime.Composable
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
import io.element.android.wysiwyg.compose.selection.SelectionAction
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
 * @param customSelectionActions A list of custom actions to add to the selection context menu.
 * @param onCustomSelectionActionSelected Called when a custom selection action is selected.
 * @param onError Called when an internal error occurs
 */
@Composable
fun RichTextEditor(
    modifier: Modifier = Modifier,
    state: RichTextEditorState = rememberRichTextEditorState(),
    registerStateUpdates: Boolean = true,
    style: RichTextEditorStyle = RichTextEditorDefaults.style(),
    customSelectionActions: List<SelectionAction> = emptyList(),
    onCustomSelectionActionSelected: (SelectionAction) -> Unit = {},
    onError: (Throwable) -> Unit = {},
) {
    val isPreview = LocalInspectionMode.current

    if (isPreview) {
        PreviewEditor(state, modifier, style)
    } else {
        RealEditor(
            state,
            registerStateUpdates,
            modifier,
            style,
            customSelectionActions,
            onCustomSelectionActionSelected,
            onError,
        )
    }
}

@Composable
private fun RealEditor(
    state: RichTextEditorState,
    registerStateUpdates: Boolean,
    modifier: Modifier = Modifier,
    style: RichTextEditorStyle,
    customSelectionActions: List<SelectionAction>,
    onCustomSelectionActionSelected: (SelectionAction) -> Unit,
    onError: (Throwable) -> Unit,
) {
    val context = LocalContext.current
    val coroutineScope = rememberCoroutineScope()

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
                    customSelectionActionModeCallback = object : ActionMode.Callback {
                        override fun onCreateActionMode(mode: ActionMode?, menu: Menu?): Boolean {
                            customSelectionActions.forEach {
                                menu?.add(Menu.NONE, it.id, Menu.NONE, it.title)
                            }
                            return true
                        }

                        override fun onActionItemClicked(
                            mode: ActionMode?,
                            item: MenuItem?
                        ): Boolean {
                            val action = customSelectionActions.find { it.id == item?.itemId }
                                    ?: return false

                            onCustomSelectionActionSelected(action)
                            mode?.finish()
                            return true
                        }

                        override fun onDestroyActionMode(mode: ActionMode?) {}

                        override fun onPrepareActionMode(mode: ActionMode?, menu: Menu?): Boolean {
                            return false
                        }
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
                            }
                        }
                    }
                }
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
    style: RichTextEditorStyle,
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
    setTextSize(TypedValue.COMPLEX_UNIT_SP, style.text.fontSize.value)
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
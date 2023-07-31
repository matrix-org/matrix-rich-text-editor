package io.element.wysiwyg.compose.ui.components

import android.annotation.SuppressLint
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.runtime.Composable
import androidx.compose.runtime.rememberCoroutineScope
import androidx.compose.ui.Modifier
import androidx.compose.ui.viewinterop.AndroidView
import io.element.android.wysiwyg.EditorEditText
import io.element.android.wysiwyg.view.models.InlineFormat
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.emptyFlow
import kotlinx.coroutines.launch
import uniffi.wysiwyg_composer.ActionState
import uniffi.wysiwyg_composer.ComposerAction
import uniffi.wysiwyg_composer.MenuAction

// TODO: Extract this to a new library
//  io.element.android:wysiwyg-compose
@SuppressLint("SetTextI18n")
@Composable
fun RichTextEditor(
    modifier: Modifier = Modifier,
    actions: Flow<EditorAction> = emptyFlow(),
    onActionsChanged: (actions: Map<ComposerAction, ActionState>) -> Unit = { },
    onSelectionChanged: (start: Int, end: Int) -> Unit = { _, _ -> },
    onMenuActionChanged: (menuAction: MenuAction) -> Unit = { },
) {
    val scope = rememberCoroutineScope()

    AndroidView(
        modifier = modifier.fillMaxWidth(),
        factory = { context ->
            EditorEditText(context).apply {
                scope.launch {
                    actions.collect(::handleAction)
                }
            }
        },
        update = { view ->
            view.actionStatesChangedListener =
                EditorEditText.OnActionStatesChangedListener { actionStates ->
                    onActionsChanged(actionStates)
                }

            view.selectionChangeListener = EditorEditText.OnSelectionChangeListener { start, end ->
                onSelectionChanged(start, end)
            }
            view.menuActionListener = EditorEditText.OnMenuActionChangedListener { menuAction ->
                onMenuActionChanged(menuAction)
            }
        }
    )
}

private fun EditorEditText.handleAction(action: EditorAction) {
    when (action) {
        is EditorAction.SetHtml ->
            setHtml(action.html)

        is EditorAction.Bold ->
            toggleInlineFormat(InlineFormat.Bold)

        is EditorAction.Italic ->
            toggleInlineFormat(InlineFormat.Italic)
    }
}


sealed interface EditorAction {
    data object Bold : EditorAction
    data object Italic : EditorAction
    data class SetHtml(val html: String) : EditorAction
}
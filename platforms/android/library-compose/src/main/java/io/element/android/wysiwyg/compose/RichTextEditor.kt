package io.element.android.wysiwyg.compose

import android.text.Editable
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.rememberCoroutineScope
import androidx.compose.runtime.setValue
import androidx.compose.ui.Modifier
import androidx.compose.ui.viewinterop.AndroidView
import androidx.core.widget.addTextChangedListener
import io.element.android.wysiwyg.EditorEditText
import io.element.android.wysiwyg.view.models.InlineFormat
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.emptyFlow
import kotlinx.coroutines.launch
import uniffi.wysiwyg_composer.ActionState
import uniffi.wysiwyg_composer.ComposerAction
import uniffi.wysiwyg_composer.MenuAction

/**
 * A composable rich text editor.
 *
 * This composable is a wrapper around the [EditorEditText] view.
 *
 * @param modifier The modifier for the layout
 * @param actions Used to send actions to the editor
 * @param onHtmlChanged Called whenever the editor content changes with the new HTML
 * @param onMarkdownChanged Called whenever the editor content changes with the new Markdown
 * @param onActionsChanged Called whenever the editor action states change
 * @param onSelectionChanged Called whenever the editor selection changes
 * @param onMenuActionChanged Called whenever the editor menu action changes
 */
@Composable
fun RichTextEditor(
    modifier: Modifier = Modifier,
    actions: Flow<EditorAction> = emptyFlow(),
    onHtmlChanged: (html: String) -> Unit = { },
    onMarkdownChanged: (markdown: String) -> Unit = { },
    onActionsChanged: (actions: Map<ComposerAction, ActionState>) -> Unit = { },
    onSelectionChanged: (start: Int, end: Int) -> Unit = { _, _ -> },
    onMenuActionChanged: (menuAction: MenuAction) -> Unit = { },
) {
    val scope = rememberCoroutineScope()

    var textWatcher by remember { mutableStateOf<((Editable?) -> Unit)>({ }) }

    AndroidView(
        modifier = modifier.fillMaxWidth(),
        factory = { context ->
            EditorEditText(context).apply {
                scope.launch {
                    actions.collect(::handleAction)
                }
                addTextChangedListener {
                    textWatcher(it)
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

            textWatcher = {
                onHtmlChanged(view.getContentAsMessageHtml())
                onMarkdownChanged(view.getMarkdown())
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


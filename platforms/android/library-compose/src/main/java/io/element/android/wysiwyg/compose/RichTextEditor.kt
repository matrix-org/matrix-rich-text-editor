package io.element.android.wysiwyg.compose

import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.viewinterop.AndroidView
import androidx.core.widget.addTextChangedListener
import io.element.android.wysiwyg.EditorEditText

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
    AndroidView(
        modifier = modifier.fillMaxWidth(),
        factory = {
            state.view.apply {
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
            }
        },
    )
}

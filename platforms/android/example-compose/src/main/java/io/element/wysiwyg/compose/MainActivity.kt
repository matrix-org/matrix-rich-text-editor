package io.element.wysiwyg.compose

import android.os.Bundle
import android.widget.Toast
import androidx.activity.ComponentActivity
import androidx.activity.compose.setContent
import androidx.compose.foundation.BorderStroke
import androidx.compose.foundation.border
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Surface
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.rememberCoroutineScope
import androidx.compose.runtime.setValue
import androidx.compose.ui.Modifier
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.tooling.preview.Preview
import androidx.compose.ui.unit.dp
import io.element.android.wysiwyg.compose.RichTextEditor
import io.element.android.wysiwyg.compose.RichTextEditorDefaults
import io.element.android.wysiwyg.compose.rememberRichTextEditorState
import io.element.android.wysiwyg.compose.selection.SelectionAction
import io.element.android.wysiwyg.view.models.InlineFormat
import io.element.android.wysiwyg.view.models.LinkAction
import io.element.wysiwyg.compose.ui.components.FormattingButtons
import io.element.wysiwyg.compose.ui.theme.RichTextEditorTheme
import kotlinx.collections.immutable.toPersistentMap
import kotlinx.coroutines.launch
import timber.log.Timber
import uniffi.wysiwyg_composer.ComposerAction

class MainActivity : ComponentActivity() {
    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        setContent {
            RichTextEditorTheme {
                val state = rememberRichTextEditorState()

                var linkDialogAction by remember { mutableStateOf<LinkAction?>(null) }
                val coroutineScope = rememberCoroutineScope()
                val context = LocalContext.current

                linkDialogAction?.let { linkAction ->
                    LinkDialog(
                        linkAction = linkAction,
                        onRemoveLink = { coroutineScope.launch { state.removeLink() } },
                        onSetLink = { coroutineScope.launch { state.setLink(it) } },
                        onInsertLink = { url, text ->
                            coroutineScope.launch {
                                state.insertLink(
                                    url,
                                    text
                                )
                            }
                        },
                        onDismissRequest = { linkDialogAction = null }
                    )
                }
                Surface(
                    modifier = Modifier.fillMaxSize(),
                    color = MaterialTheme.colorScheme.background
                ) {
                    Column(
                        modifier = Modifier.fillMaxWidth(),
                        verticalArrangement = Arrangement.SpaceBetween
                    ) {
                        Surface(
                            modifier = Modifier
                                .padding(8.dp)
                                .border(
                                    border = BorderStroke(
                                        1.dp,
                                        MaterialTheme.colorScheme.outlineVariant
                                    ),
                                )
                                .padding(8.dp),
                            color = MaterialTheme.colorScheme.surface,
                        ) {
                            RichTextEditor(
                                state = state,
                                modifier = Modifier.fillMaxWidth(),
                                style = RichTextEditorDefaults.style(),
                                onError = Timber::e,
                                customSelectionActions = listOf(
                                    SelectionAction(R.id.custom_action, getString(R.string.custom_action))
                                ),
                                onCustomSelectionActionSelected = {
                                    Toast.makeText(context, getString(R.string.custom_action_clicked), Toast.LENGTH_SHORT).show()
                                },
                            )
                        }
                        FormattingButtons(
                            onResetText = {
                                coroutineScope.launch {
                                    state.setHtml("")
                                }
                            },
                            actionStates = state.actions.toPersistentMap(),
                            onActionClick = {
                                coroutineScope.launch {
                                    when (it) {
                                        ComposerAction.BOLD -> state.toggleInlineFormat(
                                            InlineFormat.Bold
                                        )

                                        ComposerAction.ITALIC -> state.toggleInlineFormat(
                                            InlineFormat.Italic
                                        )

                                        ComposerAction.STRIKE_THROUGH -> state.toggleInlineFormat(
                                            InlineFormat.StrikeThrough
                                        )

                                        ComposerAction.UNDERLINE -> state.toggleInlineFormat(
                                            InlineFormat.Underline
                                        )

                                        ComposerAction.INLINE_CODE -> state.toggleInlineFormat(
                                            InlineFormat.InlineCode
                                        )

                                        ComposerAction.LINK ->
                                            linkDialogAction = state.linkAction

                                        ComposerAction.UNDO -> state.undo()
                                        ComposerAction.REDO -> state.redo()
                                        ComposerAction.ORDERED_LIST -> state.toggleList(ordered = true)
                                        ComposerAction.UNORDERED_LIST -> state.toggleList(
                                            ordered = false
                                        )

                                        ComposerAction.INDENT -> state.indent()
                                        ComposerAction.UNINDENT -> state.unindent()
                                        ComposerAction.CODE_BLOCK -> state.toggleCodeBlock()
                                        ComposerAction.QUOTE -> state.toggleQuote()
                                    }
                                }
                            }
                        )
                    }
                }
            }
        }
    }
}

@Preview
@Composable
fun EditorPreview() {
    RichTextEditorTheme {
        val state = rememberRichTextEditorState("Hello, world")
        RichTextEditor(
            state = state,
            modifier = Modifier.fillMaxWidth(),
        )
    }
}


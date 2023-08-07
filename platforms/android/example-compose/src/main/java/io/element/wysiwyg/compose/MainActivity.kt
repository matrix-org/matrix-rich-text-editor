package io.element.wysiwyg.compose

import android.os.Bundle
import androidx.activity.ComponentActivity
import androidx.activity.compose.setContent
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Surface
import androidx.compose.ui.Modifier
import io.element.android.wysiwyg.compose.RichTextEditor
import io.element.android.wysiwyg.compose.rememberRichTextEditorState
import io.element.android.wysiwyg.view.models.InlineFormat
import io.element.wysiwyg.compose.ui.components.FormattingButtons
import io.element.wysiwyg.compose.ui.theme.RichTextEditorTheme
import uniffi.wysiwyg_composer.ComposerAction

class MainActivity : ComponentActivity() {
    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        setContent {
            RichTextEditorTheme {
                val state = rememberRichTextEditorState()

                Surface(
                    modifier = Modifier.fillMaxSize(),
                    color = MaterialTheme.colorScheme.background
                ) {
                    Column(
                        modifier = Modifier.fillMaxWidth(),
                        verticalArrangement = Arrangement.SpaceBetween
                    ) {
                        RichTextEditor(
                            state = state,
                            modifier = Modifier.fillMaxWidth(),
                        )
                        FormattingButtons(
                            onResetText = {
                                state.setHtml("")
                            },
                            actionStates = state.actions,
                            onActionClick = { action ->
                                when(action) {
                                    ComposerAction.BOLD -> state.toggleInlineFormat(InlineFormat.Bold)
                                    ComposerAction.ITALIC -> state.toggleInlineFormat(InlineFormat.Italic)
                                    ComposerAction.STRIKE_THROUGH -> state.toggleInlineFormat(InlineFormat.StrikeThrough)
                                    ComposerAction.UNDERLINE -> state.toggleInlineFormat(InlineFormat.Underline)
                                    ComposerAction.INLINE_CODE -> state.toggleInlineFormat(InlineFormat.InlineCode)
                                    ComposerAction.LINK -> TODO()
                                    ComposerAction.UNDO -> state.undo()
                                    ComposerAction.REDO -> state.redo()
                                    ComposerAction.ORDERED_LIST -> state.toggleList(ordered = true)
                                    ComposerAction.UNORDERED_LIST -> state.toggleList(ordered = false)
                                    ComposerAction.INDENT -> state.indent()
                                    ComposerAction.UNINDENT -> state.unindent()
                                    ComposerAction.CODE_BLOCK -> state.toggleCodeBlock()
                                    ComposerAction.QUOTE -> state.toggleQuote()
                                }
                            },
                        )
                    }
                }
            }
        }
    }
}


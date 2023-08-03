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
import io.element.wysiwyg.compose.ui.components.FormattingButtons
import io.element.wysiwyg.compose.ui.theme.RichTextEditorTheme

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
                        RichTextEditor(state = state)
                        FormattingButtons(
                            onResetText = {
                                state.setHtml("")
                            },
                            onBoldClick = {
                                state.toggleBold()
                            },
                            onItalicClick = {
                                state.toggleItalic()
                            },
                            actionStates = state.actions,
                        )
                    }
                }
            }
        }
    }
}


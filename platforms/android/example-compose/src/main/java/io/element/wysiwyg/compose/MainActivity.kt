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
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.rememberCoroutineScope
import androidx.compose.runtime.setValue
import androidx.compose.ui.Modifier
import io.element.android.wysiwyg.compose.EditorAction
import io.element.android.wysiwyg.compose.RichTextEditor
import io.element.wysiwyg.compose.ui.components.FormattingButtons
import io.element.wysiwyg.compose.ui.theme.RichTextEditorTheme
import kotlinx.coroutines.flow.MutableSharedFlow
import kotlinx.coroutines.launch
import uniffi.wysiwyg_composer.ActionState
import uniffi.wysiwyg_composer.ComposerAction


class MainActivity : ComponentActivity() {
    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        setContent {
            RichTextEditorTheme {
                var actionStates by remember { mutableStateOf(emptyMap<ComposerAction, ActionState>()) }
                val scope = rememberCoroutineScope()

                val actions = MutableSharedFlow<EditorAction>()

                Surface(
                    modifier = Modifier.fillMaxSize(),
                    color = MaterialTheme.colorScheme.background
                ) {
                    Column(
                        modifier = Modifier.fillMaxWidth(),
                        verticalArrangement = Arrangement.SpaceBetween
                    ) {
                        RichTextEditor(
                            actions = actions,
                            onActionsChanged = { actionStates = it },
                        )
                        FormattingButtons(
                            onResetText = {
                                scope.launch { actions.emit(EditorAction.SetHtml("")) }
                            },
                            onBoldClick = {
                                scope.launch { actions.emit(EditorAction.Bold) }
                            },
                            onItalicClick = {
                                scope.launch { actions.emit(EditorAction.Italic) }
                            },
                            actionStates = actionStates,
                        )
                    }
                }
            }
        }
    }
}


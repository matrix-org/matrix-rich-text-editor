package io.element.wysiwyg.compose

import android.os.Bundle
import androidx.activity.ComponentActivity
import androidx.activity.compose.setContent
import androidx.compose.foundation.BorderStroke
import androidx.compose.foundation.border
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.heightIn
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.items
import androidx.compose.material3.Divider
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Surface
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateListOf
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.rememberCoroutineScope
import androidx.compose.runtime.setValue
import androidx.compose.ui.ExperimentalComposeUiApi
import androidx.compose.ui.Modifier
import androidx.compose.ui.tooling.preview.Preview
import androidx.compose.ui.unit.dp
import io.element.android.wysiwyg.compose.EditorStyledText
import io.element.android.wysiwyg.compose.RichTextEditor
import io.element.android.wysiwyg.compose.RichTextEditorDefaults
import io.element.android.wysiwyg.compose.StyledHtmlConverter
import io.element.android.wysiwyg.compose.rememberRichTextEditorState
import io.element.android.wysiwyg.view.models.InlineFormat
import io.element.android.wysiwyg.view.models.LinkAction
import io.element.wysiwyg.compose.matrix.Mention
import io.element.wysiwyg.compose.ui.components.FormattingButtons
import io.element.wysiwyg.compose.ui.theme.RichTextEditorTheme
import kotlinx.collections.immutable.toPersistentMap
import kotlinx.coroutines.launch
import timber.log.Timber
import uniffi.wysiwyg_composer.ComposerAction
import uniffi.wysiwyg_composer.MenuAction
import uniffi.wysiwyg_composer.PatternKey

class MainActivity : ComponentActivity() {

    private val roomMemberSuggestions = mutableStateListOf<Mention>()

    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        val mentionDisplayHandler = DefaultMentionDisplayHandler()
        val htmlConverter = StyledHtmlConverter(this, mentionDisplayHandler)
        setContent {
            val style = RichTextEditorDefaults.style()
            htmlConverter.configureWith(style = style)
            RichTextEditorTheme {
                val state = rememberRichTextEditorState(initialFocus = true)

                LaunchedEffect(state.menuAction) {
                    processMenuAction(state.menuAction)
                }

                var linkDialogAction by remember { mutableStateOf<LinkAction?>(null) }
                val coroutineScope = rememberCoroutineScope()

                val htmlText = htmlConverter.fromHtmlToSpans(state.messageHtml)

                linkDialogAction?.let { linkAction ->
                    LinkDialog(linkAction = linkAction,
                        onRemoveLink = { coroutineScope.launch { state.removeLink() } },
                        onSetLink = { coroutineScope.launch { state.setLink(it) } },
                        onInsertLink = { url, text ->
                            coroutineScope.launch {
                                state.insertLink(
                                    url, text
                                )
                            }
                        },
                        onDismissRequest = { linkDialogAction = null })
                }
                Surface(
                    modifier = Modifier.fillMaxSize(), color = MaterialTheme.colorScheme.background
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
                                        1.dp, MaterialTheme.colorScheme.outlineVariant
                                    ),
                                )
                                .padding(8.dp),
                            color = MaterialTheme.colorScheme.surface,
                        ) {
                            RichTextEditor(
                                state = state,
                                modifier = Modifier.fillMaxWidth().padding(10.dp),
                                style = RichTextEditorDefaults.style(),
                                onError = Timber::e,
                                mentionDisplayHandler = mentionDisplayHandler
                            )
                        }
                        EditorStyledText(
                            text = htmlText,
                            modifier = Modifier
                                .fillMaxWidth()
                                .padding(16.dp),
                        )

                        Spacer(modifier = Modifier.weight(1f))
                        LazyColumn(
                            modifier = Modifier.fillMaxWidth()
                                .heightIn(max = 320.dp)
                        ) {
                            items(roomMemberSuggestions) { item ->
                                Column {
                                    Text(
                                        text = item.display,
                                        modifier = Modifier.fillMaxWidth()
                                            .padding(10.dp)
                                            .clickable {
                                                if (item == Mention.NotifyEveryone) {
                                                    coroutineScope.launch { state.replaceSuggestion(item.text) }
                                                } else {
                                                    coroutineScope.launch { state.setLinkSuggestion(item.text, item.link) }
                                                }
                                            })
                                    Divider(modifier = Modifier.fillMaxWidth())
                                }
                            }
                        }

                        FormattingButtons(onResetText = {
                            coroutineScope.launch {
                                state.setHtml("")
                            }
                        }, actionStates = state.actions.toPersistentMap(), onActionClick = {
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

                                    ComposerAction.LINK -> linkDialogAction = state.linkAction

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
                        })
                    }
                }
            }
        }
    }

    private fun processMenuAction(menuAction: MenuAction?) {
        when (menuAction) {
            is MenuAction.Suggestion -> {
                processSuggestion(menuAction)
            }
            else -> {
                roomMemberSuggestions.clear()
            }
        }
    }

    private fun processSuggestion(suggestion: MenuAction.Suggestion) {
        val text = suggestion.suggestionPattern.text
        val people = listOf("alice", "bob", "carol", "dan").map(Mention::User)
        val rooms = listOf("matrix", "element").map(Mention::Room)
        val everyone = Mention.NotifyEveryone
        val names = when (suggestion.suggestionPattern.key) {
            PatternKey.AT -> people + everyone
            PatternKey.HASH -> rooms
            PatternKey.SLASH ->
                emptyList() // TODO
        }
        val suggestions = names
            .filter { it.display.contains(text) }
        roomMemberSuggestions.clear()
        roomMemberSuggestions.addAll(suggestions)
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


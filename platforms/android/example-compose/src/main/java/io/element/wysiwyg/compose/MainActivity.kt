package io.element.wysiwyg.compose

import android.os.Build
import android.os.Bundle
import androidx.activity.ComponentActivity
import androidx.activity.compose.setContent
import androidx.compose.foundation.BorderStroke
import androidx.compose.foundation.background
import androidx.compose.foundation.border
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.heightIn
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.foundation.text.InlineTextContent
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
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.platform.LocalDensity
import androidx.compose.ui.text.Placeholder
import androidx.compose.ui.text.PlaceholderVerticalAlign
import androidx.compose.ui.text.rememberTextMeasurer
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.tooling.preview.Preview
import androidx.compose.ui.unit.TextUnit
import androidx.compose.ui.unit.TextUnitType
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.isSpecified
import io.element.android.wysiwyg.compose.EditorStyledText
import io.element.android.wysiwyg.compose.RichTextEditor
import io.element.android.wysiwyg.compose.RichTextEditorDefaults
import io.element.android.wysiwyg.compose.StyledHtmlConverter
import io.element.android.wysiwyg.compose.internal.Mention
import io.element.android.wysiwyg.compose.rememberRichTextEditorState
import io.element.android.wysiwyg.compose.text.HtmlToComposeTextParser
import io.element.android.wysiwyg.compose.text.RichText
import io.element.android.wysiwyg.display.TextDisplay
import io.element.android.wysiwyg.view.models.InlineFormat
import io.element.android.wysiwyg.view.models.LinkAction
import io.element.wysiwyg.compose.ui.components.FormattingButtons
import io.element.wysiwyg.compose.ui.theme.RichTextEditorTheme
import kotlinx.collections.immutable.toPersistentMap
import kotlinx.coroutines.launch
import timber.log.Timber
import uniffi.wysiwyg_composer.ComposerAction
import uniffi.wysiwyg_composer.newMentionDetector

class MainActivity : ComponentActivity() {

    private val roomMemberSuggestions = mutableStateListOf<Mention>()

    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        val mentionDisplayHandler = DefaultMentionDisplayHandler()
        val mentionDetector = if (window.decorView.isInEditMode) null else newMentionDetector()
        setContent {
            val context = LocalContext.current
            RichTextEditorTheme {
                val style = RichTextEditorDefaults.style()
                val htmlConverter = remember(style) {
                    StyledHtmlConverter(
                        context = context,
                        mentionDisplayHandler = mentionDisplayHandler,
                        isMention = mentionDetector?.let { detector ->
                            { _, url ->
                                detector.isMention(url)
                            }
                        }
                    ).apply {
                        configureWith(style = style)
                    }
                }

                val state = rememberRichTextEditorState(initialFocus = true)

                LaunchedEffect(state.menuAction) {
                    processMenuAction(state.menuAction, roomMemberSuggestions)
                }

                var linkDialogAction by remember { mutableStateOf<LinkAction?>(null) }
                val coroutineScope = rememberCoroutineScope()

                val htmlText = htmlConverter.fromHtmlToSpans(state.messageHtml)
                val htmlToComposeTextParser = HtmlToComposeTextParser(
                    richTextEditorStyle = style,
                    getLinkMention = { text, url ->
                        mentionDetector?.let { detector ->
                            if (detector.isMention(url)) {
                                Mention.User(text, url)
                            } else {
                                null
                            }
                        }
                    }
                )

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
                        modifier = Modifier.fillMaxSize(),
                        verticalArrangement = Arrangement.SpaceBetween
                    ) {
                        var isTyping by remember { mutableStateOf(false) }
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
                                modifier = Modifier
                                    .fillMaxWidth()
                                    .padding(10.dp),
                                style = RichTextEditorDefaults.style(),
                                onError = { Timber.e(it) },
                                resolveMentionDisplay = { _,_ -> TextDisplay.Pill },
                                resolveRoomMentionDisplay = { TextDisplay.Pill },
                                onTyping = { isTyping = it }
                            )
                        }
                        if (isTyping) {
                            Text(
                                text = "Typing...",
                                style = MaterialTheme.typography.labelSmall,
                                modifier = Modifier
                                    .height(32.dp)
                                    .padding(horizontal = 8.dp)
                            )
                        } else {
                            Spacer(Modifier.height(32.dp))
                        }

                        val density = LocalDensity.current
                        val textMeasurer = rememberTextMeasurer()
                        val result = htmlToComposeTextParser.parse(state.messageHtml)
                        val inlineContent = buildMap<String, InlineTextContent> {
                            val extraPadding = 10.dp
                            for (mention in result.mentions) {
                                val width = with(density) {
                                    val measuredWidth = textMeasurer.measure(
                                        text = mention.text,
                                        style = MaterialTheme.typography.bodyLarge,
                                        overflow = TextOverflow.Clip,
                                    ).size.width
                                    (measuredWidth + extraPadding.toPx()).toSp()
                                }
                                val content = InlineTextContent(
                                    placeholder = Placeholder(
                                        width = TextUnit(width.value, TextUnitType.Sp),
                                        height = style.text.lineHeight.takeIf { it.isSpecified } ?: style.text.fontSize,
                                        placeholderVerticalAlign = PlaceholderVerticalAlign.Center
                                    ),
                                    children = {
                                        Text(
                                            text = mention.text,
                                            style = MaterialTheme.typography.bodyLarge,
                                            modifier = Modifier
                                                .clip(RoundedCornerShape(10.dp))
                                                .background(Color.LightGray)
                                                .clickable { Timber.i("Clicked $mention") }
                                                .padding(horizontal = extraPadding / 2)
                                        )
                                    }
                                )
                                when (mention) {
                                    is Mention.User -> put("mention:${mention.link}", content)
                                    is Mention.Room -> put("mention:${mention.link}", content)
                                    is Mention.NotifyEveryone -> put("mention:@room", content)
                                    else -> Unit
                                }
                            }
                        }
                        RichText(
                            text = result.annotatedString,
                            inlineContent = inlineContent,
                            modifier = Modifier
                                .fillMaxWidth()
                                .padding(16.dp),
                        )

//                        EditorStyledText(
//                            text = htmlText,
//                            modifier = Modifier
//                                .fillMaxWidth()
//                                .padding(16.dp),
//                            resolveMentionDisplay = { _,_ -> TextDisplay.Pill },
//                            resolveRoomMentionDisplay = { TextDisplay.Pill },
//                            onLinkClickedListener = { url ->
//                                Toast.makeText(this@MainActivity, "Clicked: $url", Toast.LENGTH_SHORT).show()
//                            }
//                        )

                        Spacer(modifier = Modifier.weight(1f))
                        SuggestionView(
                            modifier = Modifier.heightIn(max = 320.dp),
                            roomMemberSuggestions = roomMemberSuggestions,
                            onReplaceSuggestion = { text ->
                                coroutineScope.launch {
                                    state.replaceSuggestion(text)
                                }
                            },
                            onInsertAtRoomMentionAtSuggestion = {
                                coroutineScope.launch {
                                    state.insertAtRoomMentionAtSuggestion()
                                }
                            },
                            onInsertMentionAtSuggestion = { text, link ->
                                coroutineScope.launch {
                                    state.insertMentionAtSuggestion(text, link)
                                }
                            },
                        )

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


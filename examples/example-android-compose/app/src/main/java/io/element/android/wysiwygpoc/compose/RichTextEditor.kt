package io.element.android.wysiwygpoc.compose

import android.graphics.Typeface
import android.text.Html
import android.text.Spanned
import android.text.style.ForegroundColorSpan
import android.text.style.StyleSpan
import android.text.style.UnderlineSpan
import android.view.KeyEvent.*
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.material.TextField
import androidx.compose.runtime.*
import androidx.compose.ui.ExperimentalComposeUiApi
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.input.key.*
import androidx.compose.ui.platform.LocalTextInputService
import androidx.compose.ui.text.AnnotatedString
import androidx.compose.ui.text.SpanStyle
import androidx.compose.ui.text.TextRange
import androidx.compose.ui.text.buildAnnotatedString
import androidx.compose.ui.text.font.FontStyle
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.input.*
import androidx.compose.ui.text.style.TextDecoration
import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewmodel.compose.viewModel
import uniffi.wysiwyg_composer.ComposerModel
import uniffi.wysiwyg_composer.ComposerUpdate
import uniffi.wysiwyg_composer.TextUpdate

@Composable
fun RichTextEditor(
    composer: ComposerModel,
    viewModel: RichTextEditorViewModel = viewModel(),
    modifier: Modifier = Modifier,
) {
    val textFieldValue = viewModel.textFieldValue
    viewModel.composer = composer
    TextField(
        value = textFieldValue,
        modifier = Modifier.fillMaxSize(),
        // This modifier can be used to handle hardware keyboard input
        // (it has a different API than IME)
//            .onPreviewKeyEvent { keyEvent ->
////                viewModel.onKeyInput(keyEvent)
//                false
//            },
        onValueChange = {
            viewModel.textFieldValue = it
            //viewModel.updateTextFieldValue(it)
        })
    // FIXME: https://issuetracker.google.com/issues/199768107
    //  The way to intercept software keyboard input is using InputService.startInput
    //  However, due to a bug in Compose's [EditProcessor.applyTo] function, every time a change
    //  is made to the TextField (changing cursor position, adding or deleting characters), all spans
    //  in the TextFieldValue will be removed, making creating a rich text editor very difficult at
    //  the moment.

//    val inputService = LocalTextInputService.current
//    inputService?.startInput(textFieldValue, ImeOptions.Default, onEditCommand = { commands ->
//        println(commands)
//        commands.forEach { viewModel.onImeCommand(it) }
//    }, onImeActionPerformed = { action ->
//        println(action)
//    })
}

class RichTextEditorViewModel(

) : ViewModel() {

    lateinit var composer: ComposerModel

    val firstContents = Html.fromHtml("<strong>This</strong> lo<i>oks</i> <del>nice</del>.").toAnnotatedString()
    var textFieldValue by mutableStateOf(TextFieldValue(firstContents))

    private val commandsQueue = mutableListOf<EditorInputAction>()

    @OptIn(ExperimentalComposeUiApi::class)
    fun onKeyInput(keyEvent: KeyEvent) {
        if (keyEvent.nativeKeyEvent.action != ACTION_DOWN) return
        val action = when {
            keyEvent.isTypedEvent -> {
                val char = StringBuilder().appendCodePoint(keyEvent.utf16CodePoint).toString()
                EditorInputAction.InsertText(char)
            }
            keyEvent.key == Key.Backspace -> EditorInputAction.BackPress
            keyEvent.key == Key.Enter -> EditorInputAction.InsertParagraph
            else -> null
        }
        action?.let { commandsQueue.add(it) }
    }

    fun onImeCommand(command: EditCommand) {
        when (command) {
            is SetComposingTextCommand -> {
                commandsQueue.add(EditorInputAction.InsertText(command.text))
            }
            is FinishComposingTextCommand -> {}
        }
    }

    fun updateTextFieldValue(newValue: TextFieldValue) {
        if (commandsQueue.isNotEmpty()) {
            val updates = commandsQueue.map { action ->
                when (action) {
                    is EditorInputAction.InsertText -> composer.replaceText(action.value.toString())
                    is EditorInputAction.BackPress -> composer.backspace()
                    is EditorInputAction.InsertParagraph -> composer.enter()
                    is EditorInputAction.ApplyInlineFormat -> null
                }?.textUpdate() ?: TextUpdate.Keep
            }

            val lastReplaceAll = updates.lastOrNull { it is TextUpdate.ReplaceAll } as? TextUpdate.ReplaceAll
            if (lastReplaceAll != null) {
                val contents = Html.fromHtml(lastReplaceAll.replacementHtml, 0).toAnnotatedString()
                textFieldValue = TextFieldValue(contents, textFieldValue.selection, textFieldValue.composition)
            } else {
                textFieldValue = newValue
            }
            commandsQueue.clear()
        } else {
            composer.select(newValue.selection.start.toUInt(), newValue.selection.end.toUInt())
            textFieldValue = newValue
        }
    }
}

/**
 * Converts a [Spanned] into an [AnnotatedString] trying to keep as much formatting as possible.
 *
 * Currently supports `bold`, `italic`, `underline` and `color`.
 */
fun Spanned.toAnnotatedString(): AnnotatedString = buildAnnotatedString {
    val spanned = this@toAnnotatedString
    append(spanned.toString())
    getSpans(0, spanned.length, Any::class.java).forEach { span ->
        val start = getSpanStart(span)
        val end = getSpanEnd(span)
        when (span) {
            is StyleSpan -> when (span.style) {
                Typeface.BOLD -> addStyle(SpanStyle(fontWeight = FontWeight.Bold), start, end)
                Typeface.ITALIC -> addStyle(SpanStyle(fontStyle = FontStyle.Italic), start, end)
                Typeface.BOLD_ITALIC -> addStyle(SpanStyle(fontWeight = FontWeight.Bold, fontStyle = FontStyle.Italic), start, end)
            }
            is UnderlineSpan -> addStyle(SpanStyle(textDecoration = TextDecoration.Underline), start, end)
            is ForegroundColorSpan -> addStyle(SpanStyle(color = Color(span.foregroundColor)), start, end)
        }
    }
}

private val KeyEvent.isTypedEvent: Boolean
    get() = nativeKeyEvent.action == ACTION_DOWN &&
            nativeKeyEvent.unicodeChar != 0

private val KeyEvent.isMovementEvent: Boolean
    get() = nativeKeyEvent.action == ACTION_DOWN &&
            nativeKeyEvent.keyCode in listOf(KEYCODE_DPAD_UP, KEYCODE_DPAD_DOWN, KEYCODE_DPAD_RIGHT, KEYCODE_DPAD_LEFT)

sealed interface EditorInputAction {
    data class InsertText(val value: CharSequence): EditorInputAction
//    data class ReplaceAll(val value: CharSequence): EditorInputAction
    object InsertParagraph: EditorInputAction
    object BackPress: EditorInputAction
    data class ApplyInlineFormat(val format: InlineFormat): EditorInputAction
}

sealed interface InlineFormat {
    object Bold: InlineFormat
}

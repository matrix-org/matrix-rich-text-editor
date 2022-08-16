package io.element.android.wysiwyg

import android.text.Spanned
import androidx.core.text.HtmlCompat
import io.element.android.wysiwyg.extensions.log
import io.element.android.wysiwyg.extensions.string
import uniffi.wysiwyg_composer.ComposerModel
import uniffi.wysiwyg_composer.InlineFormatType
import uniffi.wysiwyg_composer.TextUpdate

class InputProcessor(
    private val composer: ComposerModel,
) {

    fun updateSelection(start: Int, end: Int) {
        if (start < 0 || end < 0) return
        composer.select(start.toUInt(), end.toUInt())
        composer.log()
    }

    fun processInput(action: EditorInputAction): TextUpdate? {
        return when (action) {
            is EditorInputAction.InsertText -> {
                // This conversion to a plain String might be too simple
                composer.replaceText(action.value.toString())
            }
            is EditorInputAction.InsertParagraph -> {
                composer.enter()
            }
            is EditorInputAction.BackPress -> {
                composer.backspace()
            }
            is EditorInputAction.ApplyInlineFormat -> {
                composer.format(action.format.toBindings())
            }
            is EditorInputAction.Delete -> {
                composer.deleteIn(action.start.toUInt(), action.end.toUInt())
            }
            is EditorInputAction.ReplaceAll -> null
            is EditorInputAction.Undo -> composer.undo()
            is EditorInputAction.Redo -> composer.redo()
        }?.textUpdate().also {
            composer.log()
        }
    }

    fun processUpdate(update: TextUpdate): ReplaceTextResult? {
        return when (update) {
            is TextUpdate.Keep -> null
            is TextUpdate.ReplaceAll -> {
                ReplaceTextResult(
                    text = stringToSpans(update.replacementHtml.string()),
                    selection = update.startUtf16Codeunit.toInt()..update.endUtf16Codeunit.toInt(),
                )
            }
        }
    }

    private fun stringToSpans(string: String): Spanned {
        // TODO: Check parsing flags
        val preparedString = string.replace(" ", "&nbsp;")
        return HtmlCompat.fromHtml(preparedString, 0)
    }
}

sealed interface EditorInputAction {
    data class InsertText(val value: CharSequence): EditorInputAction
    data class ReplaceAll(val value: CharSequence): EditorInputAction
    data class Delete(val start: Int, val end: Int): EditorInputAction
    object InsertParagraph: EditorInputAction
    object BackPress: EditorInputAction
    data class ApplyInlineFormat(val format: InlineFormat): EditorInputAction
    object Undo: EditorInputAction
    object Redo: EditorInputAction
}

sealed interface InlineFormat {
    object Bold: InlineFormat
    object Italic: InlineFormat
    object Underline: InlineFormat
    object Strikethrough: InlineFormat

    fun toBindings(): InlineFormatType = when (this) {
        Bold -> InlineFormatType.Bold
        Italic -> InlineFormatType.Italic
        Underline -> InlineFormatType.Underline
        Strikethrough -> InlineFormatType.Strikethrough
    }
}

data class ReplaceTextResult(
    val text: CharSequence,
    val selection: IntRange,
)

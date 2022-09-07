package io.element.android.wysiwyg

import android.content.Context
import android.text.*
import androidx.core.text.getSpans
import io.element.android.wysiwyg.extensions.log
import io.element.android.wysiwyg.extensions.string
import io.element.android.wysiwyg.spans.HtmlToSpansParser
import uniffi.wysiwyg_composer.ComposerModel
import uniffi.wysiwyg_composer.InlineFormatType
import uniffi.wysiwyg_composer.TextUpdate
import kotlin.math.absoluteValue

class InputProcessor(
    private val context: Context,
    private val composer: ComposerModel,
) {

    fun updateSelection(editable: Editable, start: Int, end: Int) {
        if (start < 0 || end < 0) return
        val zeroWidthLineBreaksBefore = editable.getSpans<HtmlToSpansParser.ZeroWidthLineBreak>(0, start)
            .sumOf { (editable.getSpanEnd(it) - editable.getSpanStart(it)).absoluteValue }

        val newStart = (start - zeroWidthLineBreaksBefore).toUInt()
        val newEnd = (end - zeroWidthLineBreaksBefore).toUInt()

        composer.select(newStart, newEnd)
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
            is EditorInputAction.SetLink -> composer.setLink(action.link)
            is EditorInputAction.ReplaceAll -> null
            is EditorInputAction.Undo -> composer.undo()
            is EditorInputAction.Redo -> composer.redo()
            is EditorInputAction.ToggleList -> {
                if (action.ordered) composer.orderedList() else composer.unorderedList()
            }
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
        return HtmlToSpansParser(context, string).convert()
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
    data class SetLink(val link: String): EditorInputAction
    data class ToggleList(val ordered: Boolean): EditorInputAction
}

sealed interface InlineFormat {
    object Bold: InlineFormat
    object Italic: InlineFormat
    object Underline: InlineFormat
    object StrikeThrough: InlineFormat
    object InlineCode: InlineFormat

    fun toBindings(): InlineFormatType = when (this) {
        Bold -> InlineFormatType.Bold
        Italic -> InlineFormatType.Italic
        Underline -> InlineFormatType.Underline
        StrikeThrough -> InlineFormatType.StrikeThrough
        InlineCode -> InlineFormatType.InlineCode
    }
}

data class ReplaceTextResult(
    val text: CharSequence,
    val selection: IntRange,
)


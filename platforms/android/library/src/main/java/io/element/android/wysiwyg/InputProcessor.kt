package io.element.android.wysiwyg

import android.content.Context
import android.text.Editable
import android.text.Spanned
import androidx.core.text.getSpans
import io.element.android.wysiwyg.extensions.log
import io.element.android.wysiwyg.extensions.string
import io.element.android.wysiwyg.spans.HtmlToSpansParser
import uniffi.wysiwyg_composer.ComposerModelInterface
import uniffi.wysiwyg_composer.MenuState
import uniffi.wysiwyg_composer.TextUpdate
import kotlin.math.absoluteValue

class InputProcessor(
    private val context: Context,
    private val menuStateCallback: (MenuState) -> Unit,
    private val composer: ComposerModelInterface?,
) {

    fun updateSelection(editable: Editable, start: Int, end: Int) {
        if (start < 0 || end < 0) return
        val zeroWidthLineBreaksBefore = editable.getSpans<HtmlToSpansParser.ZeroWidthLineBreak>(0, start)
            .sumOf { (editable.getSpanEnd(it) - editable.getSpanStart(it)).absoluteValue }

        val newStart = (start - zeroWidthLineBreaksBefore).toUInt()
        val newEnd = (end - zeroWidthLineBreaksBefore).toUInt()

        val update = composer?.select(newStart, newEnd)
        val menuState = update?.menuState()
        if (menuState is MenuState.Update) {
            menuStateCallback(menuState)
        }
        composer?.log()
    }

    fun processInput(action: EditorInputAction): TextUpdate? {
        val update = runCatching {
            when (action) {
                is EditorInputAction.InsertText -> {
                    // This conversion to a plain String might be too simple
                    composer?.replaceText(action.value.toString())
                }
                is EditorInputAction.InsertParagraph -> {
                    composer?.enter()
                }
                is EditorInputAction.BackPress -> {
                    composer?.backspace()
                }
                is EditorInputAction.ApplyInlineFormat -> {
                    when (action.format) {
                        InlineFormat.Bold -> composer?.bold()
                        InlineFormat.Italic -> composer?.italic()
                        InlineFormat.Underline -> composer?.underline()
                        InlineFormat.StrikeThrough -> composer?.strikeThrough()
                        InlineFormat.InlineCode -> composer?.inlineCode()
                    }
                }
                is EditorInputAction.Delete -> {
                    composer?.deleteIn(action.start.toUInt(), action.end.toUInt())
                }
                is EditorInputAction.SetLink -> composer?.setLink(action.link)
                is EditorInputAction.ReplaceAllHtml -> composer?.replaceAllHtml(action.html)
                is EditorInputAction.Undo -> composer?.undo()
                is EditorInputAction.Redo -> composer?.redo()
                is EditorInputAction.ToggleList -> {
                    if (action.ordered) composer?.orderedList() else composer?.unorderedList()
                }
            }
        }.onFailure {
            if (BuildConfig.DEBUG) {
                throw it
            } else {
                it.printStackTrace()
            }
        }.getOrNull()

        update?.menuState()?.let { menuStateCallback(it) }

        return update?.textUpdate().also {
            composer?.log()
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
            is TextUpdate.Select -> null
        }
    }

    fun getHtml(): String {
        return composer?.let { it.dumpState().html.string() }.orEmpty()
    }

    private fun stringToSpans(string: String): Spanned {
        return HtmlToSpansParser(context, string).convert()
    }
}

sealed interface EditorInputAction {
    data class InsertText(val value: CharSequence): EditorInputAction
    data class ReplaceAllHtml(val html: String): EditorInputAction
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
}

data class ReplaceTextResult(
    val text: CharSequence,
    val selection: IntRange,
)

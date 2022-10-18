package io.element.android.wysiwyg.utils

import android.text.Editable
import android.text.Spanned
import androidx.core.text.getSpans
import io.element.android.wysiwyg.spans.ExtraCharacterSpan
import kotlin.math.absoluteValue
import kotlin.math.min
import uniffi.wysiwyg_composer.ComposerModel

/**
 * The indexes in the [Editable] text and [ComposerModel] may differ if we take into account
 * [ExtraCharacterSpan] that are added for lists.
 *
 * These spans are used to mark extra characters that are not present in the original HTML output,
 * mainly extra line breaks between list items.
 */
object EditorIndexMapper {

    /**
     * Translates the [start], [end] indexes that come from the [ComposerModel] into indexes that
     * can be safely used in the [editableText].
     *
     * For this, [ExtraCharacterSpan] spans are used: we look for them in the [editableText], then
     * sum their lengths and add this extra length to the [start] and [end] original indexes.
     */
    fun fromComposerToEditor(start: Int, end: Int, editableText: Spanned): Pair<Int, Int> {
        // Invalid indexes
        if (start < 0 || end < 0) return -1 to -1

        val extraCharacters = editableText.getSpans<ExtraCharacterSpan>()
        // Extra offset to add to the start index
        val before = extraCharacters.filter { editableText.getSpanStart(it) <= start }
            .sumOf { editableText.getSpanLength(it) }
        // Extra offset to add to the end index
        val during = extraCharacters.filter { editableText.getSpanStart(it) <= end }
            .sumOf { editableText.getSpanLength(it) }
        val newStart = min(start + before, editableText.length)
        val newEnd = min(end + during, editableText.length)
        return newStart to newEnd
    }

    /**
     * Translates the [start], [end] indexes that come from the [editableText] into indexes that
     * can be safely used in the [ComposerModel].
     *
     * For this, [ExtraCharacterSpan] spans are used: we look for them in the [editableText], then
     * sum their lengths and subtract these lengths to the [start] and [end] original indexes.
     */
    fun fromEditorToComposer(start: Int, end: Int, editableText: Spanned): Pair<UInt, UInt>? {
        // Invalid indexes
        if (start < 0 || end < 0) return null

        val extraCharactersBeforeStart = editableText.getSpans<ExtraCharacterSpan>(0, start)
            .sumOf { editableText.getSpanLength(it) }

        val newStart = (start - extraCharactersBeforeStart).toUInt()
        val newEnd = (end - extraCharactersBeforeStart).toUInt()

        return newStart to newEnd
    }

    /**
     * Translates the [index] coming from the [editableText] into one that can be safely used
     * in the [ComposerModel].
     */
    fun composerIndexForEditable(index: Int, editableText: Spanned): Int {
        var consumed = 0
        var i = 0
        while (index > consumed) {
            val spans = editableText.getSpans<ExtraCharacterSpan>(start = i, end = i+1)
            if (spans.isEmpty()) {
                consumed++
                i++
            } else {
                i += spans.count()
            }
        }
        return i
    }

}

private fun Spanned.getSpanLength(span: Any): Int {
    return (getSpanEnd(span) - getSpanStart(span)).absoluteValue
}

package io.element.android.wysiwyg.utils

import android.text.Editable
import android.text.Spanned
import androidx.core.text.getSpans
import io.element.android.wysiwyg.spans.ZeroWidthLineBreak
import kotlin.math.absoluteValue
import kotlin.math.min
import uniffi.wysiwyg_composer.ComposerModel

/**
 * The indexes in the [Editable] text and [ComposerModel] may differ if we take into account
 * [ZeroWidthLineBreak] that are added for lists.
 *
 * These spans are used to mark extra characters that are not present in the original HTML output,
 * mainly extra line breaks between list items.
 */
object EditorIndexMapper {

    /**
     * Translates the [start], [end] indexes that come from the [ComposerModel] into indexes that
     * can be safely used in the [editableText].
     *
     * For this, [ZeroWidthLineBreak] spans are used: we look for them in the [editableText], then
     * sum their lengths and add this extra length to the [start] and [end] original indexes.
     */
    fun fromComposerToEditText(start: Int, end: Int, editableText: Editable): Pair<Int, Int> {
        // Invalid indexes
        if (start < 0 || end < 0) return -1 to -1

        val zeroWidthLineBreaks = editableText.getSpans<ZeroWidthLineBreak>()
        // Extra offset to add to the start index
        val before = zeroWidthLineBreaks.filter { editableText.getSpanStart(it) <= start }
            .sumOf { editableText.getSpanLength(it) }
        // Extra offset to add to the end index
        val during = zeroWidthLineBreaks.filter { editableText.getSpanStart(it) <= end }
            .sumOf { editableText.getSpanLength(it) }
        val newStart = min(start + before, editableText.length)
        val newEnd = min(end + during, editableText.length)
        return newStart to newEnd
    }

    /**
     * Translates the [start], [end] indexes that come from the [editableText] into indexes that
     * can be safely used in the [ComposerModel].
     *
     * For this, [ZeroWidthLineBreak] spans are used: we look for them in the [editableText], then
     * sum their lengths and subtract these lengths to the [start] and [end] original indexes.
     */
    fun fromEditorToComposer(start: Int, end: Int, editableText: Editable): Pair<UInt, UInt>? {
        // Invalid indexes
        if (start < 0 || end < 0) return null

        val zeroWidthLineBreaksBefore = editableText.getSpans<ZeroWidthLineBreak>(0, start)
            .sumOf { editableText.getSpanLength(it) }

        val newStart = (start - zeroWidthLineBreaksBefore).toUInt()
        val newEnd = (end - zeroWidthLineBreaksBefore).toUInt()

        return newStart to newEnd
    }

    /**
     * Translates the [index] coming from the [editableText] into one that can be safely used
     * in the [ComposerModel].
     */
    fun composerIndexForEditable(index: Int, editableText: Editable): Int {
        var consumed = 0
        var i = 0
        while (index > consumed) {
            val spans = editableText.getSpans<ZeroWidthLineBreak>(start = i, end = i+1)
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

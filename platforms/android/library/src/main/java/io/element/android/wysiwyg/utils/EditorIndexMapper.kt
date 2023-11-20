package io.element.android.wysiwyg.utils

import android.text.Editable
import android.text.Spanned
import android.text.style.ReplacementSpan
import androidx.core.text.getSpans
import io.element.android.wysiwyg.view.spans.ExtraCharacterSpan
import kotlin.math.absoluteValue
import kotlin.math.min
import uniffi.wysiwyg_composer.ComposerModel
import kotlin.math.max

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

        val newStart = editorIndexFromComposer(start, editableText)
        val newEnd = editorIndexFromComposer(end, editableText)
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

        val extraCharactersBeforeStart = editableText.getTotalSpanLengthInRange<ExtraCharacterSpan>(0, start)
        val extraCharactersDuring = editableText.getTotalSpanLengthInRange<ExtraCharacterSpan>(start, end)

        val newStart = (start - extraCharactersBeforeStart).toUInt()
        val newEnd = (end - (extraCharactersBeforeStart + extraCharactersDuring)).toUInt()

        return newStart to newEnd
    }

    /**
     * Translates the [composerIndex] coming from the [ComposerModel] into one that can be safely
     * used in the [editableText].
     */
    fun editorIndexFromComposer(composerIndex: Int, editableText: Spanned): Int {
        // Usually we could just use `editableText.getSpans<ExtraCharacterSpan>(0, 0)` and iterate
        // through its contents until the desired index, but the index from the ComposerModel can be
        // smaller than the one in the editableText and every span found means an extra character
        // to take into account and to add to the index actual position.

        // Actual indexes in the Editable we've iterated through
        var consumed = 0
        // How many extra characters we've found
        var extraOffset = 0

        while (consumed < composerIndex) {
            val indexInEditor = consumed + extraOffset

            val extraCharSpans = editableText.getSpans<ExtraCharacterSpan>(
                start = indexInEditor,
                end = indexInEditor + 1
            )
            if (extraCharSpans.isNotEmpty()) {
                // Found some extra character spans, add them to the extra offset
                // Don't increment the consumed positions
                extraOffset += extraCharSpans.count()
            } else {
                // Increment the consumed positions by default
                consumed++

                val foundReplacementSpan = editableText.getSpans<ReplacementSpan>(
                    start = indexInEditor,
                    end = indexInEditor + 1
                ).isNotEmpty()
                if (foundReplacementSpan) {
                    // If a ReplacementSpan is found, cancel the increment of consumed positions
                    // so we get an extra iteration
                    consumed--
                    // Increment the extra offset instead to keep advancing positions
                    extraOffset++
                }
            }
        }
        return consumed + extraOffset
    }

}

private inline fun <reified T: Any> Spanned.getTotalSpanLengthInRange(start: Int, end: Int): Int =
    getSpans<T>(start, end)
        .sumOf { span ->
            // Ignore any part of the span not within the range
            val clampedStart = max(start, getSpanStart(span))
            val clampedEnd = min(end, getSpanEnd(span))

            (clampedEnd - clampedStart).absoluteValue
        }

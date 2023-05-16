package io.element.android.wysiwyg.internal.utils

import android.text.Spanned
import android.text.style.ReplacementSpan
import kotlin.math.max
import kotlin.math.min

internal object TextRangeHelper {
    /**
     * Return a new range that covers the given range and extends it to cover
     * any replacement spans at either end.
     *
     * The range is returned as a pair of integers where the first is less than the last
     */
    fun extendRangeToReplacementSpans(
        spanned: Spanned,
        start: Int,
        length: Int,
    ): Pair<Int, Int> {
        require(length > 0)
        val end = start + length
        val spans = spanned.getSpans(start, end, ReplacementSpan::class.java)
        val firstReplacementSpanStart = spans.minOfOrNull { spanned.getSpanStart(it) }
        val lastReplacementSpanEnd = spans.maxOfOrNull { spanned.getSpanEnd(it) }
        val newStart = min(start, firstReplacementSpanStart ?: end)
        val newEnd = max(end, lastReplacementSpanEnd ?: end)
        return newStart to newEnd
    }
}
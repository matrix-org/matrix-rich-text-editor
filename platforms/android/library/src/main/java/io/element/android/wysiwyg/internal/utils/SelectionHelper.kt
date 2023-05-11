package io.element.android.wysiwyg.internal.utils

import android.text.Spanned
import android.text.style.ReplacementSpan
import kotlin.math.max
import kotlin.math.min

internal object SelectionHelper {
    /**
     * Return a new selection that covers the given range and extends it to cover
     * any replacement spans at either end
     */
    fun extendSelectionToReplacementSpans(
        spanned: Spanned,
        start: Int,
        end: Int,
    ): Pair<Int, Int> {
        val spans = spanned.getSpans(start, end, ReplacementSpan::class.java)
        val firstReplacementSpanStart = spans.minOfOrNull { spanned.getSpanStart(it) }
        val lastReplacementSpanEnd = spans.maxOfOrNull { spanned.getSpanEnd(it) }
        val newStart = min(start, firstReplacementSpanStart ?: end)
        val newEnd = max(end, lastReplacementSpanEnd ?: end)
        return newStart to newEnd
    }
}
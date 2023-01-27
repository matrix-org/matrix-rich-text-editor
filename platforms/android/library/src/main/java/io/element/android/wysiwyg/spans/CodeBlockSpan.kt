package io.element.android.wysiwyg.spans

import android.graphics.Canvas
import android.graphics.Paint
import android.text.Layout
import android.text.Spanned
import android.text.TextPaint
import android.text.style.*
import androidx.annotation.FloatRange
import androidx.annotation.Px

/**
 * Code block (```some code``` in Markdown, <pre> in HTML) Span that applies a monospaced font style
 * and adds an extra padding to the top and bottom of the paragraph.
 */
class CodeBlockSpan(
    @Px private val leadingMargin: Int,
    @Px private val verticalPadding: Int,
    @FloatRange(from = 0.0) relativeSizeProportion: Float =
        CodeSpanConstants.DEFAULT_RELATIVE_SIZE_PROPORTION,
) : MetricAffectingSpan(), BlockSpan, LeadingMarginSpan, LineHeightSpan, UpdateAppearance {

    private val monoTypefaceSpan = TypefaceSpan("monospace")
    private val relativeSizeSpan = RelativeSizeSpan(relativeSizeProportion)

    override fun updateDrawState(tp: TextPaint) {
        relativeSizeSpan.updateDrawState(tp)
        monoTypefaceSpan.updateDrawState(tp)
    }

    override fun updateMeasureState(textPaint: TextPaint) {
        relativeSizeSpan.updateMeasureState(textPaint)
        monoTypefaceSpan.updateMeasureState(textPaint)
    }

    override fun getLeadingMargin(first: Boolean): Int {
        return leadingMargin
    }

    override fun chooseHeight(
        text: CharSequence,
        start: Int,
        end: Int,
        spanStart: Int,
        lineHeight: Int,
        fm: Paint.FontMetricsInt,
    ) {
        val spanned = text as Spanned
        val spanEnd = spanned.getSpanEnd(this)
        // Add top padding to first line if needed
        if (start == spanStart) {
            fm.ascent -= verticalPadding
            fm.top -= verticalPadding
        }
        // Add bottom padding to last line if needed
        if (end >= spanEnd) {
            fm.descent += verticalPadding
            fm.bottom += verticalPadding
        }
    }

    override fun drawLeadingMargin(
        c: Canvas,
        p: Paint,
        x: Int,
        dir: Int,
        top: Int,
        baseline: Int,
        bottom: Int,
        text: CharSequence?,
        start: Int,
        end: Int,
        first: Boolean,
        layout: Layout?
    ) = Unit
}

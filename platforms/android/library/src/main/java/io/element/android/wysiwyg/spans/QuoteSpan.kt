package io.element.android.wysiwyg.spans

import android.graphics.Canvas
import android.graphics.Paint
import android.graphics.Rect
import android.os.Parcel
import android.text.Layout
import android.text.TextPaint
import android.text.style.LeadingMarginSpan
import android.text.style.MetricAffectingSpan

/**
 * Quote ("> a quote" in Markdown, <blockquote> in HTML) Span that applies margin and an indicator
 * on the start of the paragraph.
 */
class QuoteSpan : MetricAffectingSpan, LeadingMarginSpan {

    private val indicatorColor: Int
    private val indicatorWidth: Int
    private val indicatorPadding: Int
    private val margin: Int

    private val paint = Paint()
    private var rect = Rect()

    constructor(
        indicatorColor: Int,
        indicatorWidth: Int,
        indicatorPadding: Int,
        margin: Int
    ): super() {
        this.margin = margin
        this.indicatorWidth = indicatorWidth
        this.indicatorPadding = indicatorPadding
        this.indicatorColor = indicatorColor
    }

    constructor(parcel: Parcel): super() {
        indicatorColor = parcel.readInt()
        indicatorWidth = parcel.readInt()
        indicatorPadding = parcel.readInt()
        margin = parcel.readInt()
    }

    override fun updateDrawState(tp: TextPaint) {}

    override fun updateMeasureState(textPaint: TextPaint) {}

    override fun getLeadingMargin(first: Boolean): Int {
        return margin + indicatorWidth + indicatorPadding
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
    ) {
        paint.style = Paint.Style.FILL
        paint.color = indicatorColor

        val left: Int
        val right: Int

        if (dir > 0) {
            // Left to right
            left = x + margin
            right = x + margin + indicatorWidth
        } else {
            // Right to left
            left = x + margin - c.width - indicatorWidth
            right = x + margin
        }

        rect = Rect(left, top, right, bottom)
        c.drawRect(rect, paint)
    }
}

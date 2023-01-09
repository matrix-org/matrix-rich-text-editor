package io.element.android.wysiwyg.spans

import android.content.Context
import android.graphics.Canvas
import android.graphics.Paint
import android.graphics.Rect
import android.os.Parcel
import android.text.Layout
import android.text.ParcelableSpan
import android.text.TextPaint
import android.text.style.BackgroundColorSpan
import android.text.style.LeadingMarginSpan
import android.text.style.MetricAffectingSpan
import android.text.style.ParagraphStyle
import android.text.style.TypefaceSpan
import android.text.style.UpdateAppearance
import androidx.core.content.ContextCompat

/**
 * Code block (```some code``` in Markdown, <pre> in HTML) Span that applies a monospaced font style
 * and adds a background color to a whole paragraph.
 */
internal class CodeBlockSpan : MetricAffectingSpan, LeadingMarginSpan, UpdateAppearance {

    private val monoTypefaceSpan: TypefaceSpan
    private val backgroundColor: Int
    private val margin: Int

    private val paint = Paint()
    private var rect = Rect()

    constructor(backgroundColor: Int, margin: Int): super() {
        monoTypefaceSpan = TypefaceSpan("monospace")
        this.margin = margin
        this.backgroundColor = backgroundColor
    }

    constructor(parcel: Parcel): super() {
        monoTypefaceSpan = requireNotNull(parcel.readParcelable(TypefaceSpan::class.java.classLoader))
        backgroundColor = parcel.readInt()
        margin = parcel.readInt()
    }

    override fun updateDrawState(tp: TextPaint) {
        monoTypefaceSpan.updateDrawState(tp)
    }

    override fun updateMeasureState(textPaint: TextPaint) {
        monoTypefaceSpan.updateMeasureState(textPaint)
    }

    override fun getLeadingMargin(first: Boolean): Int {
        return margin
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
        paint.color = backgroundColor

        val left: Int
        val right: Int

        if (dir > 0) {
            // Left to right
            left = x + margin
            right = c.width
        } else {
            // Right to left
            left = x + margin - c.width
            right = x + margin
        }

        rect = Rect(left, top, right, bottom)
        c.drawRect(rect, paint)
    }
}

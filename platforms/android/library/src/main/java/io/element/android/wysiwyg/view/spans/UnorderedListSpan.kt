package io.element.android.wysiwyg.view.spans

import android.graphics.Canvas
import android.graphics.Paint
import android.graphics.Rect
import android.text.style.LeadingMarginSpan
import androidx.annotation.Px
import android.text.Layout
import android.text.Spanned
import androidx.annotation.IntRange

class UnorderedListSpan(
    @Px
    private val gapWidth: Int,
    @Px
    @IntRange(from = 0)
    private val bulletRadius: Int,
) : LeadingMarginSpan, BlockSpan {
    override fun getLeadingMargin(first: Boolean): Int {
        return 2 * bulletRadius + gapWidth
    }

    override fun drawLeadingMargin(
        canvas: Canvas, paint: Paint, x: Int, dir: Int,
        top: Int, baseline: Int, bottom: Int,
        text: CharSequence, start: Int, end: Int,
        first: Boolean, layout: Layout?
    ) {
        if ((text as Spanned).getSpanStart(this) != start) {
            return
        }

        val style = paint.style
        paint.style = Paint.Style.FILL

        val bounds = Rect().also {
            paint.getTextBounds("1", 0, 1, it)
        }

        val yPosition = (baseline - bounds.height() / 2f)
        val xPosition = (x + dir * bulletRadius).toFloat()

        canvas.drawCircle(xPosition, yPosition, bulletRadius.toFloat(), paint)
        paint.style = style
    }
}

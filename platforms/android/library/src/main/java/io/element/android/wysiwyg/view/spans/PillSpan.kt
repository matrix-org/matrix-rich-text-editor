package io.element.android.wysiwyg.view.spans

import android.graphics.Canvas
import android.graphics.Paint
import android.graphics.RectF
import android.text.style.ReplacementSpan
import androidx.annotation.ColorInt
import kotlin.math.roundToInt

internal class PillSpan(
    @ColorInt
    val backgroundColor: Int,
    val url: String? = null,
) : ReplacementSpan() {
    override fun getSize(
        paint: Paint,
        text: CharSequence?,
        start: Int,
        end: Int,
        fm: Paint.FontMetricsInt?
    ): Int {
        return paint.measureText(text, start, end).roundToInt() + 40
    }

    override fun draw(
        canvas: Canvas,
        text: CharSequence?,
        start: Int,
        end: Int,
        x: Float,
        top: Int,
        y: Int,
        bottom: Int,
        paint: Paint
    ) {
        val paintColor = paint.color
        val textSize = paint.measureText(text, start, end)
        val rect = RectF(x, top.toFloat(), x + textSize + 40, bottom.toFloat())
        paint.color = backgroundColor
        canvas.drawRoundRect(rect, rect.height() / 2, rect.height() / 2, paint)
        paint.color = paintColor
        canvas.drawText(text!!, start, end, x + 20, y.toFloat(), paint)
    }
}

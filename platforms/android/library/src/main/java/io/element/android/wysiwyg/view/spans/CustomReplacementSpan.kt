package io.element.android.wysiwyg.view.spans

import android.graphics.Canvas
import android.graphics.Paint
import android.text.style.ReplacementSpan

/**
 * Wrapper for a [ReplacementSpan] which does nothing except delegate to an
 * underlying span.
 * It is used to allow reuse of the same underlying span across multiple ranges
 * of a spanned text.
 */
internal class CustomReplacementSpan(
    private val providedSpan: ReplacementSpan
) : ReplacementSpan() {
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
    ) = providedSpan.draw(
        canvas, text, start, end, x, top, y, bottom, paint
    )

    override fun getSize(
        paint: Paint,
        text: CharSequence?,
        start: Int,
        end: Int,
        fm: Paint.FontMetricsInt?
    ): Int = providedSpan.getSize(
        paint, text, start, end, fm
    )
}
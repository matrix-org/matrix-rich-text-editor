package io.element.android.wysiwyg.inlinebg

import android.graphics.Canvas
import android.graphics.drawable.Drawable
import android.text.Layout
import android.text.Spanned
import android.text.style.LeadingMarginSpan
import androidx.core.text.getSpans

/**
 * Helper class to render a single 'block' with a bordered round rectangle as its background.
 */
internal class BlockRenderer(
    private val drawable: Drawable,
    horizontalPadding: Int,
    verticalPadding: Int,
): SpanBackgroundRenderer(horizontalPadding, verticalPadding) {

    override fun draw(
        canvas: Canvas,
        layout: Layout,
        startLine: Int,
        endLine: Int,
        startOffset: Int,
        endOffset: Int,
        leadingMargin: Int,
        text: Spanned,
        spanType: Class<*>,
    ) {
        val top = layout.getLineTop(startLine)
        val bottom = layout.getLineBottom(endLine)
        drawable.setBounds(
            if (leadingMargin > 0) leadingMargin - horizontalPadding else horizontalPadding,
            top + verticalPadding,
            layout.width - horizontalPadding * 2,
            bottom - verticalPadding * 2
        )
        drawable.draw(canvas)
    }

}

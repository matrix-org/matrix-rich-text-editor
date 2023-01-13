package io.element.android.wysiwyg.inlinebg

import android.graphics.Canvas
import android.graphics.drawable.Drawable
import android.text.Layout

/**
 * Helper class to render a single 'block' with a bordered round rectangle as its background.
 */
internal class BlockRenderer(
    private val drawable: Drawable,
    horizontalPadding: Int,
    verticalPadding: Int
): SpanBackgroundRenderer(horizontalPadding, verticalPadding) {

    override fun draw(
        canvas: Canvas,
        layout: Layout,
        startLine: Int,
        endLine: Int,
        startOffset: Int,
        endOffset: Int
    ) {
        val top = layout.getLineTop(startLine)
        val bottom = layout.getLineBottom(endLine)
        drawable.setBounds(
            horizontalPadding,
            top + verticalPadding,
            layout.width - horizontalPadding * 2,
            bottom - verticalPadding * 2
        )
        drawable.draw(canvas)
    }

}

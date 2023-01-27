/*
 * Copyright (C) 2018 The Android Open Source Project
 * Modifications Copyright 2022 New Vector Ltd
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *      http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
*/
package io.element.android.wysiwyg.inlinebg

import android.graphics.Canvas
import android.graphics.drawable.Drawable
import android.text.Layout
import android.text.Spanned
import android.text.style.LeadingMarginSpan
import androidx.core.text.getSpans
import io.element.android.wysiwyg.spans.BlockSpan

/**
 * Helper class to draw multi-line rounded background to certain parts of a text. The start/end
 * positions of the backgrounds are annotated with [android.text.Annotation] class. Each annotation
 * should have the annotation key set to **rounded**.
 *
 * i.e.:
 * ```
 *    <!--without the quotes at the begining and end Android strips the whitespace and also starts
 *        the annotation at the wrong position-->
 *    <string name="ltr">"this is <annotation key="rounded">a regular</annotation> paragraph."</string>
 * ```
 *
 * **Note:** BiDi text is not supported.
 *
 * @param horizontalPadding the padding to be applied to left & right of the background
 * @param verticalPadding the padding to be applied to top & bottom of the background
 * @param drawable the drawable used to draw the background
 * @param drawableLeft the drawable used to draw left edge of the background
 * @param drawableMid the drawable used to draw for whole line
 * @param drawableRight the drawable used to draw right edge of the background
 */
class SpanBackgroundHelper(
    private val spanType: Class<*>,
    val horizontalPadding: Int,
    val verticalPadding: Int,
    drawable: Drawable? = null,
    drawableLeft: Drawable? = null,
    drawableMid: Drawable? = null,
    drawableRight: Drawable? = null,
) {
    private var cache = mutableMapOf<SpanPosition, DrawPosition>()

    private val singleLineRenderer: SpanBackgroundRenderer by lazy {
        SingleLineRenderer(
            horizontalPadding = horizontalPadding,
            verticalPadding = verticalPadding,
            drawable = requireNotNull(drawable),
        )
    }

    private val multiLineRenderer: SpanBackgroundRenderer by lazy {
        MultiLineRenderer(
            horizontalPadding = horizontalPadding,
            verticalPadding = verticalPadding,
            drawableLeft = requireNotNull(drawableLeft),
            drawableMid = requireNotNull(drawableMid),
            drawableRight = requireNotNull(drawableRight),
        )
    }

    private val blockRenderer: SpanBackgroundRenderer by lazy {
        BlockRenderer(
            horizontalPadding = horizontalPadding,
            verticalPadding = verticalPadding,
            drawable = requireNotNull(drawable),
        )
    }

    /**
     * Call this function during onDraw of another widget such as TextView.
     *
     * @param canvas Canvas to draw onto
     * @param text
     * @param layout Layout that contains the text
     */
    fun draw(canvas: Canvas, text: Spanned, layout: Layout) {
        val spanPositions = getSpanPositions(text)
        val drawPositions = getOrCalculateDrawPositions(text, layout, spanPositions)

        drawPositions.forEach {
            val renderer = if (BlockSpan::class.java.isAssignableFrom(spanType)) {
                blockRenderer
            } else {
                if (it.startLine == it.endLine) singleLineRenderer else multiLineRenderer
            }
            renderer.draw(
                canvas,
                layout,
                it.startLine,
                it.endLine,
                it.startOffset,
                it.endOffset,
                it.leadingMargin,
                text,
                spanType
            )
        }
    }

    fun clearCachedPositions() {
        cache.clear()
    }

    private fun getSpanPositions(text: Spanned): Set<SpanPosition> {
        val spans = text.getSpans(0, text.length, spanType)
        return spans.map { SpanPosition(text.getSpanStart(it), text.getSpanEnd(it), spanType) }.toSet()
    }

    /**
     * Calculate the positions at which to draw backgrounds if they are not already cached
     */
    private fun getOrCalculateDrawPositions(
        text: Spanned,
        layout: Layout,
        spanPositions: Set<SpanPosition>
    ): Collection<DrawPosition> {
        // Remove old positions
        cache = cache.filterKeys { spanPositions.contains(it) }.toMutableMap()

        // Calculate draw positions for any new keys
        spanPositions.forEach { spanPosition ->
            cache.getOrPut(spanPosition) { calculateDrawPosition(text, layout, spanPosition) }
        }

        return cache.values
    }

    private fun calculateDrawPosition(
        text: Spanned,
        layout: Layout,
        spanPosition: SpanPosition
    ): DrawPosition {
        val (spanStart, spanEnd) = spanPosition
        val startLine = layout.getLineForOffset(spanStart)
        val endLine = layout.getLineForOffset(spanEnd)

        // start can be on the left or on the right depending on the language direction.
        val startOffset = (layout.getPrimaryHorizontal(spanStart)
                - layout.getParagraphDirection(startLine) * horizontalPadding).toInt()
        // end can be on the left or on the right depending on the language direction.
        val endOffset = (layout.getPrimaryHorizontal(spanEnd)
                + layout.getParagraphDirection(endLine) * horizontalPadding).toInt()

        val startIndex = layout.getOffsetForHorizontal(startLine, 0f)
        val endIndex = layout.getOffsetForHorizontal(endLine, 0f)
        val leadingMarginSpans = text.getSpans<LeadingMarginSpan>(startIndex, endIndex)
            .filter { !spanType.isInstance(it) }
        val leadingMargin = leadingMarginSpans.sumOf { it.getLeadingMargin(true) }

        return DrawPosition(startLine, endLine, startOffset, endOffset, leadingMargin)
    }
}

internal data class SpanPosition(
    val spanStart: Int,
    val spanEnd: Int,
    val spanType: Class<*>,
)

internal data class DrawPosition(
    val startLine: Int,
    val endLine: Int,
    val startOffset: Int,
    val endOffset: Int,
    val leadingMargin: Int,
)

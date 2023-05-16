package io.element.android.wysiwyg.internal.utils

import android.graphics.Canvas
import android.graphics.Paint
import android.text.SpannableStringBuilder
import android.text.Spanned
import android.text.style.ReplacementSpan
import org.junit.Assert.assertEquals
import org.junit.Test
import org.junit.runner.RunWith
import org.robolectric.RobolectricTestRunner

@RunWith(RobolectricTestRunner::class)
class TextRangeHelperTest {
    @Test(expected = IllegalArgumentException::class)
    fun `given negative length, when extend to cover ReplacementSpans, it throws`() {
        val text = SpannableStringBuilder("0123456789")

        TextRangeHelper.extendRangeToReplacementSpans(
            text, 3, -1
        )
    }
    @Test
    fun `given plain text, when extend to cover ReplacementSpans, selection is not extended`() {
        val text = SpannableStringBuilder("0123456789")

        val newSelection = TextRangeHelper.extendRangeToReplacementSpans(
            text, 3, 4
        )

        assertEquals(3 to 7, newSelection)
    }

    @Test
    fun `given ReplacementSpan at end, when extend to cover ReplacementSpans, selection extended`() {
        val text = SpannableStringBuilder("0123456789")
        text.setSpan(MyReplacementSpan(), 6, 10, Spanned.SPAN_EXCLUSIVE_EXCLUSIVE)

        val newSelection = TextRangeHelper.extendRangeToReplacementSpans(
            text, 3, 4
        )

        assertEquals(3 to 10, newSelection)
    }

    @Test
    fun `given ReplacementSpan at start, when extend to cover ReplacementSpans, selection extended`() {
        val text = SpannableStringBuilder("0123456789")
        text.setSpan(MyReplacementSpan(), 0, 4, Spanned.SPAN_EXCLUSIVE_EXCLUSIVE)

        val newSelection = TextRangeHelper.extendRangeToReplacementSpans(
            text, 3, 4
        )

        assertEquals(0 to 7, newSelection)
    }

    @Test
    fun `given ReplacementSpan immediately before and after, when extend to cover ReplacementSpans, selection not extended`() {
        val text = SpannableStringBuilder("0123456789")
        text.setSpan(MyReplacementSpan(), 0, 3, Spanned.SPAN_EXCLUSIVE_EXCLUSIVE)
        text.setSpan(MyReplacementSpan(), 7, 10, Spanned.SPAN_EXCLUSIVE_EXCLUSIVE)

        val newSelection = TextRangeHelper.extendRangeToReplacementSpans(
            text, 3, 4
        )

        assertEquals(3 to 7, newSelection)
    }

    @Test
    fun `given ReplacementSpan at start and end, when extend to cover ReplacementSpans, selection extended`() {
        val text = SpannableStringBuilder("0123456789")
        text.setSpan(MyReplacementSpan(), 0, 4, Spanned.SPAN_EXCLUSIVE_EXCLUSIVE)
        text.setSpan(MyReplacementSpan(), 6, 10, Spanned.SPAN_EXCLUSIVE_EXCLUSIVE)

        val newSelection = TextRangeHelper.extendRangeToReplacementSpans(
            text, 3, 4
        )

        assertEquals(0 to 10, newSelection)
    }
}

class MyReplacementSpan : ReplacementSpan() {
    override fun getSize(
        paint: Paint,
        text: CharSequence?,
        start: Int,
        end: Int,
        fm: Paint.FontMetricsInt?
    ): Int = 10

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
    }
}
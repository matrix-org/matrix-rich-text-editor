package io.element.android.wysiwyg.utils

import androidx.core.text.toSpanned
import io.mockk.every
import io.mockk.mockk
import org.hamcrest.MatcherAssert.assertThat
import org.hamcrest.Matchers.equalTo
import org.junit.Test
import org.junit.runner.RunWith
import org.robolectric.RobolectricTestRunner

@RunWith(RobolectricTestRunner::class)
class AndroidHtmlConverterTest {
    private val htmlToSpansParser = mockk<HtmlToSpansParser>()
    private val androidHtmlConverter = AndroidHtmlConverter(
        provideHtmlToSpansParser = { htmlToSpansParser }
    )

    @Test
    fun testToPlainText_removesTags() {
        val result = androidHtmlConverter.fromHtmlToPlainText(
            "<b>Hello</b> <i>world</i>"
        )

        assertThat(result, equalTo("Hello world"))
    }

    @Test
    fun testToPlainText_handlesLineBreaks() {
        val result = androidHtmlConverter.fromHtmlToPlainText(
            "<p><b>Hello</b><br /><i>world</i></p>"
        )

        assertThat(result, equalTo("Hello\nworld\n\n"))
    }

    @Test
    fun testToSpans() {
        val expectedParserOutput = "mock parser output".toSpanned()
        every { htmlToSpansParser.convert() } returns expectedParserOutput

        val result = androidHtmlConverter.fromHtmlToSpans("input")

        assertThat(result, equalTo(expectedParserOutput))
    }
}
package io.element.android.wysiwyg.test.utils

import androidx.test.ext.junit.runners.AndroidJUnit4
import io.element.android.wysiwyg.utils.AndroidHtmlConverter
import org.hamcrest.MatcherAssert.assertThat
import org.hamcrest.Matchers.equalTo
import org.junit.Test
import org.junit.runner.RunWith

@RunWith(AndroidJUnit4::class)
class AndroidHtmlConverterTest {
    private val androidHtmlConverter = AndroidHtmlConverter

    @Test
    fun testToPlaintext_removesTags() {
        val result = androidHtmlConverter.fromHtmlToPlainText(
            "<b>Hello</b> <i>world</i>"
        )

        assertThat(result, equalTo("Hello world"))
    }

    @Test
    fun testToPlaintext_handlesParagraphAndLineBreaks() {
        val result = androidHtmlConverter.fromHtmlToPlainText(
            "<p><b>Hello</b><br /><i>world</i></p>"
        )

        assertThat(result, equalTo("Hello\nworld\n\n"))
    }
}
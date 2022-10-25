package io.element.android.wysiwyg.test.utils

import android.text.TextUtils
import androidx.test.core.app.ApplicationProvider
import androidx.test.ext.junit.runners.AndroidJUnit4
import io.element.android.wysiwyg.utils.AndroidResourcesProvider
import io.element.android.wysiwyg.utils.HtmlToSpansParser
import org.hamcrest.MatcherAssert.assertThat
import org.hamcrest.Matchers.equalTo
import org.junit.Test
import org.junit.runner.RunWith

@RunWith(AndroidJUnit4::class)
class HtmlToSpansParserTest {
    @Test
    fun testStyles() {
        val html = "<b>bold</b>" +
                "<i>italic</i>" +
                "<u>underline</u>" +
                "<strong>strong</strong>" +
                "<em>emphasis</em>" +
                "<del>strikethrough</del>" +
                "<code>code</code>"
        val spanned = convertHtml(html)

        assertThat(
            spanned.dumpSpans(), equalTo(
                listOf(
                    "bold: android.text.style.StyleSpan (0-4) fl=#33",
                    "italic: android.text.style.StyleSpan (4-10) fl=#33",
                    "underline: android.text.style.UnderlineSpan (10-19) fl=#33",
                    "strong: android.text.style.StyleSpan (19-25) fl=#33",
                    "emphasis: android.text.style.StyleSpan (25-33) fl=#33",
                    "strikethrough: android.text.style.StrikethroughSpan (33-46) fl=#33",
                    "code: io.element.android.wysiwyg.spans.InlineCodeSpan (46-50) fl=#33",
                )
            )
        )
    }

    @Test
    fun testLists() {
        val html = """
            <ol>
                <li>ordered1</li>
                <li>ordered2</li>
            </ol>
            <ul>
                <li>bullet1</li>
                <li>bullet2</li>
            </ul>
        """.trimIndent()
        val spanned = convertHtml(html)


        assertThat(
            spanned.dumpSpans(), equalTo(
                listOf(
                    "\u200B: io.element.android.wysiwyg.spans.ExtraCharacterSpan (0-1) fl=#33",
                    "\u200Bordered1: io.element.android.wysiwyg.spans.OrderedListSpan (0-9) fl=#33",
                    "\n: io.element.android.wysiwyg.spans.ExtraCharacterSpan (9-10) fl=#33",
                    "ordered2: io.element.android.wysiwyg.spans.OrderedListSpan (10-18) fl=#33",
                    "bullet1: android.text.style.BulletSpan (19-26) fl=#33",
                    "\n: io.element.android.wysiwyg.spans.ExtraCharacterSpan (26-27) fl=#33",
                    "bullet2: android.text.style.BulletSpan (27-34) fl=#33"
                )
            )
        )
    }

    @Test
    fun testLineBreaks() {
        val html = "Hello<br>world"
        val spanned = convertHtml(html)
        assertThat(
            spanned.dumpSpans(), equalTo(
                emptyList()
            )
        )
        assertThat(
            spanned.toString(), equalTo("Hello\nworld")
        )
    }

    @Test
    fun testIgnoresParagraphs() {
        val html = "<p>Hello</p><br /><p>world</p>"
        val spanned = convertHtml(html)
        assertThat(
            spanned.dumpSpans(), equalTo(
                emptyList()
            )
        )
        assertThat(
            spanned.toString(), equalTo("Hello\nworld")
        )
    }

    private fun CharSequence.dumpSpans(): List<String> {
        val spans = mutableListOf<String>()
        TextUtils.dumpSpans(
            this, { span ->
                val spanWithoutHash = span.split(" ").filterIndexed { index, _ ->
                    index != 1
                }.joinToString(" ")

                spans.add(spanWithoutHash)
            }, ""
        )
        return spans
    }


    private fun convertHtml(html: String) =
        HtmlToSpansParser(
            resourcesProvider = AndroidResourcesProvider(application = ApplicationProvider.getApplicationContext()),
            html = html
        ).convert()
}
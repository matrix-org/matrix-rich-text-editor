package io.element.android.wysiwyg.test.utils

import android.app.Application
import android.text.Spanned
import androidx.test.core.app.ApplicationProvider
import androidx.test.ext.junit.runners.AndroidJUnit4
import io.element.android.wysiwyg.fakes.createFakeStyleConfig
import io.element.android.wysiwyg.suggestions.MatrixMentionUrlFilter
import io.element.android.wysiwyg.suggestions.MentionUrlFilter
import io.element.android.wysiwyg.utils.AndroidResourcesHelper
import io.element.android.wysiwyg.utils.HtmlToSpansParser
import io.element.android.wysiwyg.utils.NBSP
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
            spanned.dumpSpans().joinToString(",\n"), equalTo(
                """
                    ordered1: io.element.android.wysiwyg.spans.OrderedListSpan (0-8) fl=#34,
                    ordered2: io.element.android.wysiwyg.spans.OrderedListSpan (9-17) fl=#34,
                    bullet1: io.element.android.wysiwyg.spans.UnorderedListSpan (18-25) fl=#34,
                    bullet2: io.element.android.wysiwyg.spans.UnorderedListSpan (26-33) fl=#34
                """.trimIndent()
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
    fun testParagraphs() {
        val html = "<p>Hello</p><p>world</p>"
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
    fun testEmptyParagraphs() {
        val html = "<p></p><p></p>"
        val spanned = convertHtml(html)
        assertThat(
            spanned.dumpSpans(), equalTo(
                listOf(
                    "$NBSP: io.element.android.wysiwyg.spans.ExtraCharacterSpan (0-1) fl=#17",
                    "$NBSP: io.element.android.wysiwyg.spans.ExtraCharacterSpan (2-3) fl=#17"
                )
            )
        )
        assertThat(
            spanned.toString(), equalTo("$NBSP\n$NBSP")
        )
    }

    @Test
    fun testLineBreakCanWorkWithParagraphs() {
        val html = "<p>Hello</p><br /><p>world</p>"
        val spanned = convertHtml(html)
        assertThat(
            spanned.dumpSpans(), equalTo(
                emptyList()
            )
        )
        assertThat(
            spanned.toString(), equalTo("Hello\n\nworld")
        )
    }

    @Test
    fun testMentionsWithMatrixFilter() {
        val html = """
            <a href="https://element.io">link</a>
            <a href="https://matrix.to/#/@jonny.andrew:matrix.org">jonny</a>
        """.trimIndent()
        val spanned = convertHtml(html, mentionUrlFilter = MatrixMentionUrlFilter())
        assertThat(
            spanned.dumpSpans(), equalTo(
                listOf(
                    "link: io.element.android.wysiwyg.spans.LinkSpan (0-4) fl=#33",
                    "jonny: io.element.android.wysiwyg.spans.MentionSpan (5-10) fl=#33"
                )
            )
        )
        assertThat(
            spanned.toString(), equalTo("link\njonny")
        )
    }

    @Test
    fun testMentionsWithCustomFilter() {
        val html = """
            <a href="https://element.io">link</a>
            <a href="https://matrix.to/#/@jonny.andrew:matrix.org">jonny</a>
        """.trimIndent()
        val spanned = convertHtml(html, mentionUrlFilter = MentionUrlFilter {
            it.contains("element.io")
        })
        assertThat(
            spanned.dumpSpans(), equalTo(
                listOf(
                    "link: io.element.android.wysiwyg.spans.MentionSpan (0-4) fl=#33",
                    "jonny: io.element.android.wysiwyg.spans.LinkSpan (5-10) fl=#33"
                )
            )
        )
        assertThat(
            spanned.toString(), equalTo("link\njonny")
        )
    }

    private fun convertHtml(
        html: String,
        mentionUrlFilter: MentionUrlFilter? = MatrixMentionUrlFilter()
    ): Spanned {
        val app = ApplicationProvider.getApplicationContext<Application>()
        return HtmlToSpansParser(
            resourcesHelper = AndroidResourcesHelper(application = app),
            html = html,
            styleConfig = createFakeStyleConfig(),
            mentionUrlFilter = mentionUrlFilter,
        ).convert()
    }
}

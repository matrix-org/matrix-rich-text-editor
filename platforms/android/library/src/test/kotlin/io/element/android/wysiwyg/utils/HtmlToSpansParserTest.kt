package io.element.android.wysiwyg.utils

import android.text.Spanned
import io.element.android.wysiwyg.display.KeywordDisplayHandler
import io.element.android.wysiwyg.display.TextDisplay
import io.element.android.wysiwyg.display.LinkDisplayHandler
import io.element.android.wysiwyg.test.fakes.createFakeStyleConfig
import io.element.android.wysiwyg.test.utils.dumpSpans
import io.element.android.wysiwyg.view.spans.CustomReplacementSpan
import io.element.android.wysiwyg.view.spans.PillSpan
import org.hamcrest.MatcherAssert.assertThat
import org.hamcrest.Matchers.equalTo
import org.junit.Test
import org.junit.runner.RunWith
import org.robolectric.RobolectricTestRunner
import org.robolectric.RuntimeEnvironment

@RunWith(RobolectricTestRunner::class)
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
                    "code: io.element.android.wysiwyg.view.spans.InlineCodeSpan (46-50) fl=#33",
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
                    ordered1: io.element.android.wysiwyg.view.spans.OrderedListSpan (0-8) fl=#34,
                    ordered2: io.element.android.wysiwyg.view.spans.OrderedListSpan (9-17) fl=#34,
                    bullet1: io.element.android.wysiwyg.view.spans.UnorderedListSpan (18-25) fl=#34,
                    bullet2: io.element.android.wysiwyg.view.spans.UnorderedListSpan (26-33) fl=#34
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
                    "$NBSP: io.element.android.wysiwyg.view.spans.ExtraCharacterSpan (0-1) fl=#17",
                    "$NBSP: io.element.android.wysiwyg.view.spans.ExtraCharacterSpan (2-3) fl=#17"
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
    fun testLinkDisplayWithCustomLinkDisplayHandler() {
        val html = """
            <a href="https://element.io">link</a>
            <a href="https://matrix.to/#/@test:matrix.org">jonny</a>
        """.trimIndent()
        val spanned = convertHtml(html, linkDisplayHandler = { _, url ->
            if(url.contains("element.io")) {
                TextDisplay.Pill
            } else {
                TextDisplay.Plain
            }
        })
        assertThat(
            spanned.dumpSpans(), equalTo(
                listOf(
                    "link: io.element.android.wysiwyg.view.spans.PillSpan (0-4) fl=#33",
                    "jonny: io.element.android.wysiwyg.view.spans.LinkSpan (5-10) fl=#33"
                )
            )
        )
        assertThat(
            spanned.toString(), equalTo("link\njonny")
        )
    }

    @Test
    fun testKeywordDisplayWithCustomKeywordDisplayHandler() {
        val keyword1 = "\$hello"
        val keyword2 = "anotherkeyword"
        val keyword3 = "plainkeyword"
        val html = "$keyword1 $keyword2 $keyword3"
        val spanned = convertHtml(html, keywordDisplayHandler = object: KeywordDisplayHandler {
            override val keywords: List<String> = listOf(keyword1, keyword2)
            override fun resolveKeywordDisplay(text: String): TextDisplay =
                when(text) {
                    keyword1 -> TextDisplay.Pill
                    keyword2 -> TextDisplay.Custom(PillSpan(0))
                    keyword3 -> TextDisplay.Plain
                    else -> TextDisplay.Plain
                }
        })
        assertThat(
            spanned.dumpSpans(), equalTo(
                listOf(
                    "\$hello: io.element.android.wysiwyg.view.spans.PillSpan (0-6) fl=#33",
                    "anotherkeyword: io.element.android.wysiwyg.view.spans.CustomReplacementSpan (7-21) fl=#33"
                )
            )
        )
        assertThat(
            spanned.toString(), equalTo("\$hello anotherkeyword plainkeyword")
        )
    }

    private fun convertHtml(
        html: String,
        linkDisplayHandler: LinkDisplayHandler? = null,
        keywordDisplayHandler: KeywordDisplayHandler? = null,
    ): Spanned {
        val app = RuntimeEnvironment.getApplication()
        return HtmlToSpansParser(
            resourcesHelper = AndroidResourcesHelper(application = app),
            html = html,
            styleConfig = createFakeStyleConfig(),
            linkDisplayHandler = linkDisplayHandler,
            keywordDisplayHandler = keywordDisplayHandler,
        ).convert()
    }
}

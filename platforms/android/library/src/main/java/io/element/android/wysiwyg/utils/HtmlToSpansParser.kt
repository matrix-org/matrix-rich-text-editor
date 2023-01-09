package io.element.android.wysiwyg.utils

import android.graphics.Typeface
import android.text.Editable
import android.text.SpannableString
import android.text.SpannableStringBuilder
import android.text.Spanned
import android.text.style.ParagraphStyle
import android.text.style.StrikethroughSpan
import android.text.style.StyleSpan
import android.text.style.UnderlineSpan
import androidx.core.text.getSpans
import io.element.android.wysiwyg.BuildConfig
import io.element.android.wysiwyg.inputhandlers.models.InlineFormat
import io.element.android.wysiwyg.spans.CodeBlockSpan
import io.element.android.wysiwyg.spans.ExtraCharacterSpan
import io.element.android.wysiwyg.spans.InlineCodeSpan
import io.element.android.wysiwyg.spans.LinkSpan
import io.element.android.wysiwyg.spans.OrderedListSpan
import io.element.android.wysiwyg.spans.UnorderedListSpan
import io.element.android.wysiwyg.spans.QuoteSpan
import org.ccil.cowan.tagsoup.Parser
import org.xml.sax.Attributes
import org.xml.sax.ContentHandler
import org.xml.sax.InputSource
import org.xml.sax.Locator
import java.io.StringReader
import kotlin.math.roundToInt

/**
 * Custom HTML to Span parser so we can customise what each HTML tag will be represented with in the
 * formatted text.
 *
 * This is specially important for lists, since they not only use custom spans, but they also need
 * to create [ExtraCharacterSpan] spans to work properly.
 */
internal class HtmlToSpansParser(
    private val resourcesProvider: ResourcesProvider,
    private val html: String,
    private val styleConfig: StyleConfig,
) : ContentHandler {

    // Spans created to be used as 'marks' while parsing
    private data class Hyperlink(val link: String)
    private object OrderedListBlock
    private object UnorderedListBlock
    private object CodeBlock
    private object Quote
    private data class ListItem(val ordered: Boolean, val order: Int? = null)

    private val parser = Parser().also { it.contentHandler = this }
    private val text = SpannableStringBuilder()

    fun convert(): Spanned {
        parser.parse(InputSource(StringReader(html)))
        if (BuildConfig.DEBUG) text.assertOnlyAllowedSpans()
        return text
    }

    override fun setDocumentLocator(locator: Locator?) {}

    override fun startDocument() {}

    override fun endDocument() {}

    override fun startPrefixMapping(prefix: String?, uri: String?) {}

    override fun endPrefixMapping(prefix: String?) {}

    override fun startElement(uri: String?, localName: String, qName: String?, atts: Attributes?) {
        handleStartTag(localName, atts)
    }

    override fun endElement(uri: String?, localName: String, qName: String?) {
        handleEndTag(localName)
    }

    override fun characters(ch: CharArray, start: Int, length: Int) {
        for (i in start until start + length) {
            val char = ch[i]
            text.append(char)
        }
    }

    override fun ignorableWhitespace(ch: CharArray, start: Int, length: Int) {}

    override fun processingInstruction(target: String?, data: String?) {}

    override fun skippedEntity(name: String?) {}

    private fun handleStartTag(name: String, attrs: Attributes?) {
        when (name) {
            "b", "strong" -> handleFormatStartTag(InlineFormat.Bold)
            "i", "em" -> handleFormatStartTag(InlineFormat.Italic)
            "u" -> handleFormatStartTag(InlineFormat.Underline)
            "del" -> handleFormatStartTag(InlineFormat.StrikeThrough)
            "code" -> handleFormatStartTag(InlineFormat.InlineCode)
            "a" -> {
                val url = attrs?.getValue("href") ?: return
                handleHyperlinkStart(url)
            }
            "ul", "ol" -> {
                val mark: Any = if (name == "ol") OrderedListBlock else UnorderedListBlock
                text.setSpan(mark, text.length, text.length, Spanned.SPAN_INCLUSIVE_EXCLUSIVE)
            }
            "li" -> {
                val lastListBlock =
                    getLast<OrderedListBlock>() ?: getLast<UnorderedListBlock>() ?: return
                val start = text.getSpanStart(lastListBlock)
                val newItem = when (lastListBlock) {
                    is OrderedListBlock -> {
                        val lastListItem = getLast<OrderedListSpan>(from = start)
                        val order = (lastListItem?.order ?: 0) + 1
                        ListItem(true, order)
                    }
                    is UnorderedListBlock -> ListItem(false)
                    else -> return
                }
                text.setSpan(newItem, text.length, text.length, Spanned.SPAN_INCLUSIVE_EXCLUSIVE)
            }
            "pre" -> {
                text.setSpan(CodeBlock, text.length, text.length, Spanned.SPAN_INCLUSIVE_EXCLUSIVE)
            }
            "blockquote" -> {
                text.setSpan(Quote, text.length, text.length, Spanned.SPAN_INCLUSIVE_EXCLUSIVE)
            }
        }
    }

    private fun handleEndTag(name: String) {
        when (name) {
            "br" -> text.append("\n")
            "b", "strong" -> handleFormatEndTag(InlineFormat.Bold)
            "i", "em" -> handleFormatEndTag(InlineFormat.Italic)
            "u" -> handleFormatEndTag(InlineFormat.Underline)
            "del" -> handleFormatEndTag(InlineFormat.StrikeThrough)
            "code" -> handleFormatEndTag(InlineFormat.InlineCode)
            "a" -> handleHyperlinkEnd()
            "ul", "ol" -> {
                val mark: Any = if (name == "ol") OrderedListBlock else UnorderedListBlock
                val last = getLast(mark::class.java) ?: return
                text.removeSpan(last)
            }
            "li" -> {
                val last = getLast<ListItem>() ?: return
                var start = text.getSpanStart(last)
                // We only add line breaks *after* a previous <li> element if there is not already a line break
                if (start > 0 && start <= text.length && text[start - 1] != '\n') {
                    // We add a line break to actually display the list item
                    val extraLineBreakSpan = SpannableString("\n").apply {
                        setSpan(ExtraCharacterSpan(), 0, 1, Spanned.SPAN_EXCLUSIVE_EXCLUSIVE)
                    }
                    text.insert(start, extraLineBreakSpan)
                    start += 1
                }

                val span = createListSpan(last = last)

                text.setSpan(span, start, text.length, Spanned.SPAN_EXCLUSIVE_EXCLUSIVE)
                text.removeSpan(last)
            }
            "pre" -> {
                val last = getLast<CodeBlock>() ?: return
                val start = addLeadingLineBreakIfNeeded(text.getSpanStart(last))
                text.removeSpan(last)

                val codeSpan = CodeBlockSpan(
                    0xC0A0A0A0.toInt(),
                    10.dpToPx().toInt(),
                )

                addZWSP(text.length)

                text.setSpan(codeSpan, start, text.length, Spanned.SPAN_INCLUSIVE_EXCLUSIVE)
            }
            "blockquote" -> {
                val last = getLast<Quote>() ?: return
                val start = addLeadingLineBreakIfNeeded(text.getSpanStart(last))
                text.removeSpan(last)

                val quoteSpan = QuoteSpan(
                    indicatorColor = 0xC0A0A0A0.toInt(),
                    indicatorWidth = 4.dpToPx().toInt(),
                    indicatorPadding = 6.dpToPx().toInt(),
                    margin = 10.dpToPx().toInt(),
                )

                addZWSP(text.length)

                text.setSpan(quoteSpan, start, text.length, Spanned.SPAN_INCLUSIVE_EXCLUSIVE)
            }
        }
    }

    private fun addLeadingLineBreakIfNeeded(start: Int): Int {
        return if (start > 0) {
            if (text[start] == ZWSP) {
                text.replace(start, start + 1, "\n")
            } else {
                text.insert(start, "\n")
            }
            start + 1
        } else start
    }

    private fun addZWSP(pos: Int) {
        text.insert(pos, "$ZWSP")
        text.setSpan(ExtraCharacterSpan(), pos, pos+1, Spanned.SPAN_INCLUSIVE_EXCLUSIVE)
    }

    private fun handleFormatStartTag(format: InlineFormat) {
        text.setSpan(format, text.length, text.length, Spanned.SPAN_INCLUSIVE_EXCLUSIVE)
    }

    private fun handleFormatEndTag(format: InlineFormat) {
        val span = when (format) {
            InlineFormat.Bold -> StyleSpan(Typeface.BOLD)
            InlineFormat.Italic -> StyleSpan(Typeface.ITALIC)
            InlineFormat.Underline -> UnderlineSpan()
            InlineFormat.StrikeThrough -> StrikethroughSpan()
            InlineFormat.InlineCode ->
                InlineCodeSpan(resourcesProvider.getColor(android.R.color.darker_gray))
        }
        setSpanFromMark(format, span)
    }

    private fun handleHyperlinkStart(url: String) {
        val hyperlink = Hyperlink(url)
        text.setSpan(hyperlink, text.length, text.length, Spanned.SPAN_INCLUSIVE_EXCLUSIVE)
    }

    private fun handleHyperlinkEnd() {
        val last = getLast<Hyperlink>() ?: return
        val span = LinkSpan(last.link)
        setSpanFromMark(last, span)
    }

    private fun setSpanFromMark(mark: Any, vararg spans: Any) {
        val lastTag = getLast(mark::class.java) ?: return
        val startIndex = text.getSpanStart(lastTag)
        for (span in spans) {
            text.setSpan(span, startIndex, text.length, Spanned.SPAN_EXCLUSIVE_EXCLUSIVE)
        }
        text.removeSpan(lastTag)
    }

    private fun createListSpan(last: ListItem): ParagraphStyle {
        val gapWidth = styleConfig.bulletGapWidth.roundToInt()
        val bulletRadius = styleConfig.bulletRadius.roundToInt()

        return if (last.ordered) {
            // TODO: provide typeface and textSize somehow
            val typeface = Typeface.defaultFromStyle(Typeface.NORMAL)
            val textSize = 16.dpToPx()
            OrderedListSpan(typeface, textSize, last.order ?: 1, gapWidth)
        } else {
            UnorderedListSpan(gapWidth, bulletRadius)
        }
    }

    private inline fun <reified T : Any> getLast(from: Int = 0, to: Int = text.length): T? {
        val spans = text.getSpans<T>(from, to)
        return spans.lastOrNull()
    }

    private fun <T : Any> getLast(kind: Class<T>, from: Int = 0, to: Int = text.length): T? {
        val spans = text.getSpans(from, to, kind)
        return spans.lastOrNull()
    }

    private fun Int.dpToPx(): Float {
        return resourcesProvider.dpToPx(this)
    }

    companion object FormattingSpans {
        /**
         * This list keeps track of the spans used by the editor.
         *
         * This is needed because the editor currently uses Editable.replace
         * to replace the entire contents of the editor when the model changes.
         * This method does not replace spans that are not contained within the
         * range resulting in spans which cover the whole range being duplicated.
         *
         * @see android.text.Editable.replace(int, int, CharSequence)
         * to
         */
        private val spans: List<Class<out Any>> = listOf(
            // Formatting
            StyleSpan::class.java,
            UnderlineSpan::class.java,
            StrikethroughSpan::class.java,
            InlineCodeSpan::class.java,

            // Links
            LinkSpan::class.java,

            // Lists
            UnorderedListSpan::class.java,
            OrderedListSpan::class.java,

            ExtraCharacterSpan::class.java,

            // Blocks
            CodeBlockSpan::class.java,
            QuoteSpan::class.java,
        )

        fun Editable.removeFormattingSpans() =
            spans.flatMap { type ->
                getSpans(0, length, type).toList()
            }.forEach {
                removeSpan(it)
            }

        fun Spanned.assertOnlyAllowedSpans() =
            assert(getSpans(0, length, Any::class.java).all {
                spans.contains(it.javaClass)
            })
    }
}

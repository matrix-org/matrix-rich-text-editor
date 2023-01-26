package io.element.android.wysiwyg.utils

import android.graphics.Typeface
import android.text.Editable
import android.text.SpannableStringBuilder
import android.text.Spanned
import android.text.style.ParagraphStyle
import android.text.style.StrikethroughSpan
import android.text.style.StyleSpan
import android.text.style.UnderlineSpan
import androidx.core.text.getSpans
import io.element.android.wysiwyg.BuildConfig
import io.element.android.wysiwyg.inputhandlers.models.InlineFormat
import io.element.android.wysiwyg.spans.BlockSpan
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
import kotlin.math.max
import kotlin.math.roundToInt

/**
 * Custom HTML to Span parser so we can customise what each HTML tag will be represented with in the
 * formatted text.
 *
 * This is specially important for lists, since they not only use custom spans, but they also need
 * to create [ExtraCharacterSpan] spans to work properly.
 */
internal class HtmlToSpansParser(
    private val resourcesHelper: ResourcesHelper,
    private val html: String,
    private val styleConfig: StyleConfig,
) : ContentHandler {

    private class AddSpan(
        val span: Any,
        val start: Int,
        val end: Int,
        val flags: Int,
        val isPlaceholder: Boolean,
    )

    // Spans created to be used as 'marks' while parsing
    private sealed interface PlaceholderSpan {
        data class Hyperlink(val link: String): PlaceholderSpan
        sealed interface ListBlock: PlaceholderSpan {
            class Ordered: ListBlock
            class Unordered: ListBlock
        }
        class CodeBlock: PlaceholderSpan
        class Quote: PlaceholderSpan
        class Paragraph : BlockSpan, PlaceholderSpan
        data class ListItem(
            val ordered: Boolean,
            val order: Int? = null
        ) : BlockSpan, PlaceholderSpan
    }

    /**
     * Child tags are parsed before their parents,
     * causing them to be added to the [text] also in reversed order.
     * In example, in:
     * ```
     * <li><blockquote>text</blockquote></li>
     * ```
     * The `blockquote` tag will be parsed and its span will be added first to the text, then the
     * `li` one will be parsed and its span will be added. However, as they were added in reversed
     * order (`quote > li` instead of `li > quote`), it will appear as a list item inside a quote
     * when Android renders the resulting text.
     *
     * To fix that, we're creating this list of Spans to be added to the text:
     * 1. When we parse the start tag, we set a placeholder span to 'book' the position.
     * 2. When we parse the end tag, we replace that placeholder with the real span, keeping the
     * starting position.
     * 3. Once we've parsed the whole HTML, we apply these spans to the text in order.
     *
     * *Note*: this is only needed for block spans, as inline spans can be rendered in any order.
     */
    private val spansToAdd = mutableListOf<AddSpan>()

    private val parser = Parser().also { it.contentHandler = this }
    private val text = SpannableStringBuilder()

    fun convert(): Spanned {
        spansToAdd.clear()
        parser.parse(InputSource(StringReader(html)))
        for (spanToAdd in spansToAdd) {
            text.setSpan(spanToAdd.span, spanToAdd.start, spanToAdd.end, spanToAdd.flags)
        }
        text.removePlaceholderSpans()
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
                addLeadingLineBreakIfNeeded(text.length)
                val mark: PlaceholderSpan = if (name == "ol") {
                    PlaceholderSpan.ListBlock.Ordered()
                } else {
                    PlaceholderSpan.ListBlock.Unordered()
                }
                addPlaceHolderSpan(mark)
                text.setSpan(mark, text.length, text.length, Spanned.SPAN_INCLUSIVE_EXCLUSIVE)
            }
            "li" -> {
                addLeadingLineBreakIfNeeded(text.length)
                val lastListBlock = getLast<PlaceholderSpan.ListBlock>() ?: return
                val start = text.getSpanStart(lastListBlock)
                val newItem = when (lastListBlock) {
                    is PlaceholderSpan.ListBlock.Ordered -> {
                        val lastListItem = spansToAdd.findLast {
                            it.span is OrderedListSpan && it.start >= start
                        }?.span as? OrderedListSpan
                        val order = (lastListItem?.order ?: 0) + 1
                        PlaceholderSpan.ListItem(true, order)
                    }
                    is PlaceholderSpan.ListBlock.Unordered -> PlaceholderSpan.ListItem(false)
                    else -> return
                }
                addPlaceHolderSpan(newItem)
                text.setSpan(newItem, text.length, text.length, Spanned.SPAN_INCLUSIVE_EXCLUSIVE)
            }
            "pre" -> {
                addLeadingLineBreakIfNeeded(text.length)
                val placeholder = PlaceholderSpan.CodeBlock()
                addPlaceHolderSpan(placeholder)
                text.setSpan(placeholder, text.length, text.length, Spanned.SPAN_INCLUSIVE_EXCLUSIVE)
            }
            "blockquote" -> {
                addLeadingLineBreakIfNeeded(text.length)
                val placeholder = PlaceholderSpan.Quote()
                addPlaceHolderSpan(placeholder)
                text.setSpan(placeholder, text.length, text.length, Spanned.SPAN_INCLUSIVE_EXCLUSIVE)
            }
            "p" -> {
                addLeadingLineBreakIfNeeded(text.length)
                val placeholder = PlaceholderSpan.Paragraph()
                addPlaceHolderSpan(placeholder)
                text.setSpan(placeholder, text.length, text.length, Spanned.SPAN_INCLUSIVE_EXCLUSIVE)
            }
        }
    }

    private fun handleEndTag(name: String) {
        when (name) {
            "br" -> {
                addLeadingLineBreakIfNeeded(text.length)
                text.append("\n")
            }
            "b", "strong" -> handleFormatEndTag(InlineFormat.Bold)
            "i", "em" -> handleFormatEndTag(InlineFormat.Italic)
            "u" -> handleFormatEndTag(InlineFormat.Underline)
            "del" -> handleFormatEndTag(InlineFormat.StrikeThrough)
            "code" -> handleFormatEndTag(InlineFormat.InlineCode)
            "a" -> handleHyperlinkEnd()
            "ul", "ol" -> {
                val mark: Any = if (name == "ol") {
                    PlaceholderSpan.ListBlock.Ordered()
                } else {
                    PlaceholderSpan.ListBlock.Unordered()
                }
                val last = getLast(mark::class.java) ?: return
                text.removeSpan(last)
            }
            "li" -> {
                val last = getLast<PlaceholderSpan.ListItem>() ?: return
                val start = text.getSpanStart(last)
                text.removeSpan(last)

                addNBSP(start)

                val span = createListSpan(last = last)
                replacePlaceholderSpanWith(last, span, start, text.length, Spanned.SPAN_EXCLUSIVE_INCLUSIVE)
            }
            "pre" -> {
                val last = getLast<PlaceholderSpan.CodeBlock>() ?: return
                val start = text.getSpanStart(last)
                text.removeSpan(last)

                val lastNewLine = max(text.lastIndexOf('\n') + 1, start);
                addNBSP(lastNewLine)

                if (text.lastOrNull() == '\n') {
                    // Extra char to properly render empty new lines in code blocks
                    addNBSP(text.length)
                }

                val codeSpan = CodeBlockSpan(styleConfig.codeBlock.leadingMargin, styleConfig.codeBlock.verticalPadding)
                replacePlaceholderSpanWith(last, codeSpan, start, text.length, Spanned.SPAN_EXCLUSIVE_EXCLUSIVE)
            }
            "blockquote" -> {
                val last = getLast<PlaceholderSpan.Quote>() ?: return
                val start = text.getSpanStart(last)
                text.removeSpan(last)

                addNBSP(start)

                val quoteSpan = QuoteSpan(
                    indicatorColor = 0xC0A0A0A0.toInt(),
                    indicatorWidth = 4.dpToPx().toInt(),
                    indicatorPadding = 6.dpToPx().toInt(),
                    margin = 10.dpToPx().toInt(),
                )
                replacePlaceholderSpanWith(last, quoteSpan, start, text.length, Spanned.SPAN_EXCLUSIVE_EXCLUSIVE)
            }
            "p" -> {
                val last = getLast<PlaceholderSpan.Paragraph>() ?: return
                val start = text.getSpanStart(last)
                text.removeSpan(start)

                addNBSP(start)

                replacePlaceholderSpanWith(last, last, start, text.length, Spanned.SPAN_EXCLUSIVE_EXCLUSIVE)
            }
        }
    }

    // region: Handle parsing of tags into spans

    private fun handleFormatStartTag(format: InlineFormat) {
        text.setSpan(format, text.length, text.length, Spanned.SPAN_INCLUSIVE_EXCLUSIVE)
    }

    private fun handleFormatEndTag(format: InlineFormat) {
        val span: Any = when (format) {
            InlineFormat.Bold -> StyleSpan(Typeface.BOLD)
            InlineFormat.Italic -> StyleSpan(Typeface.ITALIC)
            InlineFormat.Underline -> UnderlineSpan()
            InlineFormat.StrikeThrough -> StrikethroughSpan()
            InlineFormat.InlineCode -> InlineCodeSpan()
        }
        setSpanFromMark(format, span)
    }

    private fun handleHyperlinkStart(url: String) {
        val hyperlink = PlaceholderSpan.Hyperlink(url)
        text.setSpan(hyperlink, text.length, text.length, Spanned.SPAN_INCLUSIVE_EXCLUSIVE)
    }

    private fun handleHyperlinkEnd() {
        val last = getLast<PlaceholderSpan.Hyperlink>() ?: return
        val span = LinkSpan(last.link)
        setSpanFromMark(last, span)
    }

    private fun setSpanFromMark(mark: Any, vararg spans: Any) {
        val lastTag = getLast(mark::class.java) ?: return
        val startIndex = text.getSpanStart(lastTag)
        for (span in spans) {
            addSpan(span, startIndex, text.length, Spanned.SPAN_EXCLUSIVE_EXCLUSIVE)
        }
        text.removeSpan(lastTag)
    }

    private fun createListSpan(last: PlaceholderSpan.ListItem): ParagraphStyle {
        val gapWidth = styleConfig.bulletList.bulletGapWidth.roundToInt()
        val bulletRadius = styleConfig.bulletList.bulletRadius.roundToInt()

        return if (last.ordered) {
            // TODO: provide typeface and textSize somehow
            val typeface = Typeface.defaultFromStyle(Typeface.NORMAL)
            val textSize = 16.dpToPx()
            OrderedListSpan(typeface, textSize, last.order ?: 1, gapWidth)
        } else {
            UnorderedListSpan(gapWidth, bulletRadius)
        }
    }

    // endregion

    // region: Utils for whitespaces and indexes

    private fun addLeadingLineBreakIfNeeded(start: Int): Int {
        val previousBlock = spansToAdd.filter {
            it.span is BlockSpan && !it.isPlaceholder && (it.start >= start -1 || it.end <= start)
        }.maxByOrNull { it.start }
        return if (previousBlock == null) {
            start
        } else {
            val previousBlockEnd = previousBlock.end
            if (previousBlockEnd == start) {
                text.insert(start, "\n")
                start + 1
            } else {
                start
            }
        }
    }

    private fun addNBSP(pos: Int) {
        if (pos == text.length) {
            // If there was no NBSP char, add a new one as an extra character
            text.append(NBSP)
            text.setSpan(ExtraCharacterSpan(), pos, pos+1, Spanned.SPAN_INCLUSIVE_EXCLUSIVE)
        } else if (text.length - pos == 1 && text.last() == NBSP) {
            // If there was one, set it as an extra character
            text.setSpan(ExtraCharacterSpan(), pos, text.length, Spanned.SPAN_INCLUSIVE_EXCLUSIVE)
        }
    }

    // endregion

    // region: Handle span placeholders for block spans

    private fun addSpan(span: Any, start: Int, end: Int, flags: Int) {
        spansToAdd.add(AddSpan(span, start, end, flags, false))
    }

    private fun addPlaceHolderSpan(span: Any) {
        val addSpan = AddSpan(span, 0, 0, 0, true)
        spansToAdd.add(addSpan)
    }

    private fun replacePlaceholderSpanWith(
        placeholder: Any,
        span: Any,
        start: Int,
        end: Int,
        flags: Int
    ) {
        val index = spansToAdd.indexOfFirst { it.span === placeholder }
        if (index >= 0) {
            spansToAdd[index] = AddSpan(span, start, end, flags, false)
        }
    }

    // endregion

    private inline fun <reified T : Any> getLast(from: Int = 0, to: Int = text.length): T? {
        val spans = text.getSpans<T>(from, to)
        return spans.lastOrNull()
    }

    private fun <T : Any> getLast(kind: Class<T>, from: Int = 0, to: Int = text.length): T? {
        val spans = text.getSpans(from, to, kind)
        return spans.lastOrNull()
    }

    private fun Int.dpToPx(): Float {
        return resourcesHelper.dpToPx(this)
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

        fun Editable.removePlaceholderSpans() =
            spans.flatMap { type ->
                getSpans<PlaceholderSpan>(0, length).toList()
            }.forEach {
                removeSpan(it)
            }

        fun Spanned.assertOnlyAllowedSpans() =
            assert(getSpans(0, length, Any::class.java).all {
                spans.contains(it.javaClass)
            })
    }
}

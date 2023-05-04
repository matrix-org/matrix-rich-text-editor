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
import io.element.android.wysiwyg.spans.*
import io.element.android.wysiwyg.spans.LinkSpan
import io.element.android.wysiwyg.spans.MentionSpan
import io.element.android.wysiwyg.suggestions.MentionUrlFilter
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
    private val resourcesHelper: ResourcesHelper,
    private val html: String,
    private val styleConfig: StyleConfig,
    private val mentionUrlFilter: MentionUrlFilter?,
) : ContentHandler {

    /**
     * Class used to hold information about what spans should be added to the text, keeping the
     * natural order of insertion that would otherwise be broken by the parsing order.
     */
    private class PendingSpan<T: Any>(
        val span: T,
        val start: Int,
        val end: Int,
        val flags: Int,
        val isPlaceholder: Boolean,
    )

    // Spans created to be used as 'marks' while parsing
    private sealed interface PlaceholderSpan {
        data class Hyperlink(val link: String): PlaceholderSpan
        data class Mention(val link: String): PlaceholderSpan
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
     * Child tags are parsed before their parents, causing them to be added to the [text]
     * also in reversed order.
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
    private val spansToAdd = mutableListOf<PendingSpan<*>>()

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
            "code" -> {
                if(getLastPending<PlaceholderSpan.CodeBlock>() != null) return
                handleFormatStartTag(InlineFormat.InlineCode)
            }
            "a" -> {
                val url = attrs?.getValue("href") ?: return
                if (mentionUrlFilter?.isMention(url) == true) {
                    handleMentionStart(url)
                } else {
                    handleHyperlinkStart(url)
                }
            }
            "ul", "ol" -> {
                addLeadingLineBreakIfNeeded(text.length)
                val mark: PlaceholderSpan = if (name == "ol") {
                    PlaceholderSpan.ListBlock.Ordered()
                } else {
                    PlaceholderSpan.ListBlock.Unordered()
                }
                addPlaceHolderSpan(mark)
            }
            "li" -> {
                addLeadingLineBreakIfNeeded(text.length)
                val lastListBlock = getLastPending<PlaceholderSpan.ListBlock>() ?: return
                val start = text.getSpanStart(lastListBlock)
                val newItem = when (lastListBlock.span) {
                    is PlaceholderSpan.ListBlock.Ordered -> {
                        val lastListItem = spansToAdd.findLast {
                            it.span is OrderedListSpan && it.start >= start
                        }?.span as? OrderedListSpan
                        val order = (lastListItem?.order ?: 0) + 1
                        PlaceholderSpan.ListItem(true, order)
                    }
                    is PlaceholderSpan.ListBlock.Unordered -> PlaceholderSpan.ListItem(false)
                }
                addPlaceHolderSpan(newItem)
            }
            "pre" -> {
                addLeadingLineBreakIfNeeded(text.length)
                val placeholder = PlaceholderSpan.CodeBlock()
                addPlaceHolderSpan(placeholder)
            }
            "blockquote" -> {
                addLeadingLineBreakIfNeeded(text.length)
                val placeholder = PlaceholderSpan.Quote()
                addPlaceHolderSpan(placeholder)
            }
            "p" -> {
                addLeadingLineBreakIfNeeded(text.length)
                val placeholder = PlaceholderSpan.Paragraph()
                addPlaceHolderSpan(placeholder)
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
            "a" -> {
                handleMentionEnd()
                handleHyperlinkEnd()
            }
            "li" -> {
                val last = getLastPending<PlaceholderSpan.ListItem>() ?: return
                val start = last.start

                handleNbspInBlock(start)

                val span = createListSpan(last = last.span)
                replacePlaceholderWithPendingSpan(
                    placeholder = last.span,
                    span = span,
                    start = start,
                    flags = Spanned.SPAN_EXCLUSIVE_INCLUSIVE
                )
            }
            "pre" -> {
                val last = getLastPending<PlaceholderSpan.CodeBlock>() ?: return
                val start = last.start

                handleNbspInBlock(start)
                for (i in start+1 until text.length) {
                    if (text[i] == NBSP) {
                        // Extra char to properly render empty new lines in code blocks
                        handleNbspInBlock(i)
                    }
                }

                val codeSpan = CodeBlockSpan(
                    leadingMargin = styleConfig.codeBlock.leadingMargin,
                    verticalPadding = styleConfig.codeBlock.verticalPadding,
                    relativeSizeProportion = styleConfig.codeBlock.relativeTextSize,
                )
                replacePlaceholderWithPendingSpan(
                    placeholder = last.span,
                    span = codeSpan,
                    start = start,
                    flags = Spanned.SPAN_EXCLUSIVE_EXCLUSIVE
                )
            }
            "blockquote" -> {
                val last = getLastPending<PlaceholderSpan.Quote>() ?: return
                val start = last.start

                handleNbspInBlock(start)

                val quoteSpan = QuoteSpan(
                    indicatorColor = 0xC0A0A0A0.toInt(),
                    indicatorWidth = 4.dpToPx().toInt(),
                    indicatorPadding = 6.dpToPx().toInt(),
                    margin = 10.dpToPx().toInt(),
                )
                replacePlaceholderWithPendingSpan(
                    placeholder = last.span,
                    span = quoteSpan,
                    start = start,
                    flags = Spanned.SPAN_EXCLUSIVE_EXCLUSIVE
                )
            }
            "p" -> {
                val last = getLastPending<PlaceholderSpan.Paragraph>() ?: return
                val start = last.start

                handleNbspInBlock(start)

                replacePlaceholderWithPendingSpan(
                    placeholder = last.span,
                    start = start,
                    flags = Spanned.SPAN_EXCLUSIVE_EXCLUSIVE
                )
            }
        }
    }

    // region: Handle parsing of tags into spans

    private fun handleFormatStartTag(format: InlineFormat) {
        addPlaceHolderSpan(format)
    }

    private fun handleFormatEndTag(format: InlineFormat) {
        val last = getLastPending(format::class.java) ?: return
        val span: Any = when (format) {
            InlineFormat.Bold -> StyleSpan(Typeface.BOLD)
            InlineFormat.Italic -> StyleSpan(Typeface.ITALIC)
            InlineFormat.Underline -> UnderlineSpan()
            InlineFormat.StrikeThrough -> StrikethroughSpan()
            InlineFormat.InlineCode -> InlineCodeSpan(
                relativeSizeProportion = styleConfig.inlineCode.relativeTextSize
            )
        }
        replacePlaceholderWithPendingSpan(last.span, span, last.start, text.length, Spanned.SPAN_EXCLUSIVE_EXCLUSIVE)
    }

    private fun handleHyperlinkStart(url: String) {
        val hyperlink = PlaceholderSpan.Hyperlink(url)
        addPlaceHolderSpan(hyperlink)
    }

    private fun handleHyperlinkEnd() {
        val last = getLastPending<PlaceholderSpan.Hyperlink>() ?: return
        val span = LinkSpan((last.span).link)
        replacePlaceholderWithPendingSpan(last.span, span, last.start, text.length, Spanned.SPAN_EXCLUSIVE_EXCLUSIVE)
    }

    private fun handleMentionStart(url: String) {
        val hyperlink = PlaceholderSpan.Mention(url)
        addPlaceHolderSpan(hyperlink)
    }

    private fun handleMentionEnd() {
        val last = getLastPending<PlaceholderSpan.Mention>() ?: return
        val span = MentionSpan((last.span).link, resourcesHelper.getColor(styleConfig.mention.backgroundColor))
        replacePlaceholderWithPendingSpan(last.span, span, last.start, text.length, Spanned.SPAN_EXCLUSIVE_EXCLUSIVE)
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

    /**
     * Add a line break for the current block element if needed.
     */
    private fun addLeadingLineBreakIfNeeded(start: Int): Int {
        val previousBlock = spansToAdd.findLast {
            it.span is BlockSpan && !it.isPlaceholder && (it.start >= start -1 || it.end <= start)
        }
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

    /**
     * Either add an extra NBSP character if missing in the current block element, or, if a NBSP
     * character exists, set it as extra character so it's ignored when mapping indexes.
     */
    private fun handleNbspInBlock(pos: Int) {
        if (pos == text.length) {
            // If there was no NBSP char, add a new one as an extra character
            text.append(NBSP)
            addPendingSpan(ExtraCharacterSpan(), pos, pos + 1, Spanned.SPAN_INCLUSIVE_EXCLUSIVE)
        } else if (text.length > pos && text[pos] == NBSP) {
            // If there was one, set it as an extra character
            addPendingSpan(ExtraCharacterSpan(), pos, pos+1, Spanned.SPAN_INCLUSIVE_EXCLUSIVE)
        }
    }

    // endregion

    // region: Handle span placeholders for block spans

    /**
     * Used to add a placeholder span to be replaced in the future with its real version.
     * This is used to 'book' a position inside the list of pending spans when we read a starting
     * HTML tag so we can replace it later with its real version and range in the text.
     */
    private fun <T: Any> addPlaceHolderSpan(
        span: T,
        start: Int = text.length,
        end: Int = text.length
    ) {
        val pendingSpan = PendingSpan(span, start, end, 0, true)
        spansToAdd.add(pendingSpan)
    }

    /**
     * Adds the final version of a span to the list of pending spans to be inserted in the text,
     * with valid type, [start], [end], and [flags].
     */
    private fun <T: Any> addPendingSpan(
        span: T,
        start: Int = text.length,
        end: Int = text.length,
        flags: Int
    ) {
        spansToAdd.add(PendingSpan(span, start, end, flags, false))
    }

    /**
     * Replaces a [placeholder] span in the list of pending spans with its final version, including
     * the real type, the [start] and [end] indexes and the real [flags] to apply.
     */
    private fun replacePlaceholderWithPendingSpan(
        placeholder: Any,
        span: Any = placeholder,
        start: Int = text.length,
        end: Int = text.length,
        flags: Int
    ) {
        val index = spansToAdd.indexOfFirst { it.span === placeholder }
        if (index >= 0) {
            spansToAdd[index] = PendingSpan(span, start, end, flags, false)
        }
    }

    // endregion

    /**
     * Looks for the last span of the type [T] in the list of pending spans
     * in the range ([from], [to]).
     */
    private inline fun <reified T : Any> getLastPending(
        from: Int = 0,
        to: Int = text.length
    ): PendingSpan<T>? {
        return getLastPending(T::class.java, from, to)
    }

    /**
     * Looks for the last span of the [type] in the list of pending spans
     * in the range ([from], [to]).
     */
    @Suppress("UNCHECKED_CAST")
    private fun <T : Any> getLastPending(
        type: Class<T>,
        from: Int = 0,
        to: Int = text.length
    ): PendingSpan<T>? {
        return spansToAdd.findLast {
            type.isInstance(it.span) && from <= it.start && to >= it.end
        } as? PendingSpan<T>
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
        private val spans: HashSet<Class<out Any>> = hashSetOf(
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

            // Pills
            MentionSpan::class.java,
        )

        fun Editable.removeFormattingSpans() =
            spans.flatMap { type ->
                getSpans(0, length, type).toList()
            }.forEach {
                removeSpan(it)
            }

        fun Editable.removePlaceholderSpans() =
            spans.flatMap { _ ->
                getSpans<PlaceholderSpan>(0, length).toList()
            }.forEach {
                removeSpan(it)
            }

        fun Spanned.assertOnlyAllowedSpans() {
            val textSpans = getSpans(0, length, Any::class.java)
            assert(textSpans.all { spans.contains(it.javaClass) }) {
                "Spans in text contain invalid spans.\n\n${textSpans.joinToString("\n")}"
            }
        }
    }
}

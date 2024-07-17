package io.element.android.wysiwyg.utils

import android.graphics.Typeface
import android.text.Editable
import android.text.SpannableStringBuilder
import android.text.Spanned
import android.text.style.StrikethroughSpan
import android.text.style.StyleSpan
import android.text.style.UnderlineSpan
import androidx.core.text.buildSpannedString
import androidx.core.text.inSpans
import io.element.android.wysiwyg.BuildConfig
import io.element.android.wysiwyg.display.MentionDisplayHandler
import io.element.android.wysiwyg.display.TextDisplay
import io.element.android.wysiwyg.view.StyleConfig
import io.element.android.wysiwyg.view.models.InlineFormat
import io.element.android.wysiwyg.view.spans.CodeBlockSpan
import io.element.android.wysiwyg.view.spans.CustomMentionSpan
import io.element.android.wysiwyg.view.spans.ExtraCharacterSpan
import io.element.android.wysiwyg.view.spans.InlineCodeSpan
import io.element.android.wysiwyg.view.spans.LinkSpan
import io.element.android.wysiwyg.view.spans.OrderedListSpan
import io.element.android.wysiwyg.view.spans.PillSpan
import io.element.android.wysiwyg.view.spans.PlainAtRoomMentionDisplaySpan
import io.element.android.wysiwyg.view.spans.QuoteSpan
import io.element.android.wysiwyg.view.spans.UnorderedListSpan
import org.jsoup.Jsoup
import org.jsoup.nodes.Element
import org.jsoup.nodes.TextNode
import timber.log.Timber
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
    private val mentionDisplayHandler: MentionDisplayHandler?,
    private val isMention: ((text: String, url: String) -> Boolean)? = null,
) {
    /**
     * Convert the HTML string into a [Spanned] text.
     */
    fun convert(): Spanned {
        val dom = Jsoup.parse(html)
        val text = buildSpannedString {
            val body = dom.body()
            parseChildren(body)
            addAtRoomSpans()
        }
        if (BuildConfig.DEBUG) text.assertOnlyAllowedSpans()
        return text
    }

    private fun SpannableStringBuilder.parseChildren(element: Element) {
        for (child in element.childNodes()) {
            when (child) {
                is Element -> parseElement(child)
                is TextNode -> parseTextNode(child)
            }
        }
    }

    private fun SpannableStringBuilder.parseElement(element: Element) {
        when (element.tagName()) {
            "a" -> parseLink(element)
            "b", "strong" -> parseInlineFormatting(element, InlineFormat.Bold)
            "i", "em" -> parseInlineFormatting(element, InlineFormat.Italic)
            "u" -> parseInlineFormatting(element, InlineFormat.Underline)
            "del" -> parseInlineFormatting(element, InlineFormat.StrikeThrough)
            "code" -> parseInlineCode(element)
            "ul", "ol" -> parseChildren(element)
            "li" -> parseListItem(element)
            "pre" -> parseCodeBlock(element)
            "blockquote" -> parseQuote(element)
            "p" -> parseParagraph(element)
            "br" -> parseLineBreak(element)
            else -> if (LoggingConfig.enableDebugLogs) {
                Timber.d("Unsupported tag: ${element.tagName()}")
            }
        }
    }

    // region: Handle parsing of tags into spans

    private fun SpannableStringBuilder.parseTextNode(child: TextNode) {
        val text = child.wholeText
        if (text.isEmpty()) return

        val previousSibling = child.previousSibling() as? Element
        if (previousSibling != null && previousSibling.isBlock) {
            append('\n')
        }
        append(text)
    }

    private fun SpannableStringBuilder.parseInlineFormatting(element: Element, inlineFormat: InlineFormat) {
        val span = when (inlineFormat) {
            InlineFormat.Bold -> StyleSpan(Typeface.BOLD)
            InlineFormat.Italic -> StyleSpan(Typeface.ITALIC)
            InlineFormat.Underline -> UnderlineSpan()
            InlineFormat.StrikeThrough -> StrikethroughSpan()
            InlineFormat.InlineCode -> return
        }
        inSpans(span) {
            parseChildren(element)
        }
    }

    private fun SpannableStringBuilder.parseLineBreak(element: Element) {
        if (element.previousElementSibling()?.isBlock == true) {
            append('\n')
        }
        append('\n')
    }

    private fun SpannableStringBuilder.parseParagraph(element: Element) {
        addLeadingLineBreakForBlockNode(element)
        val start = this.length
        parseChildren(element)
        handleNbspInBlock(element, start, length)
    }

    private fun SpannableStringBuilder.parseQuote(element: Element) {
        addLeadingLineBreakForBlockNode(element)
        val start = this.length
        inSpans(
            QuoteSpan(
                // TODO provide these values from the style config
                indicatorColor = 0xC0A0A0A0.toInt(),
                indicatorWidth = 4.dpToPx().toInt(),
                indicatorPadding = 6.dpToPx().toInt(),
                margin = 10.dpToPx().toInt(),
            )
        ) {
            parseChildren(element)
        }

        handleNbspInBlock(element, start, length)
    }

    private fun SpannableStringBuilder.parseCodeBlock(element: Element) {
        addLeadingLineBreakForBlockNode(element)
        val start = this.length
        inSpans(
            CodeBlockSpan(
                leadingMargin = styleConfig.codeBlock.leadingMargin,
                verticalPadding = styleConfig.codeBlock.verticalPadding,
                relativeSizeProportion = styleConfig.codeBlock.relativeTextSize,
            )
        ) {
            append(element.wholeText())
        }

        handleNbspInBlock(element, start, length)
        // Handle NBSPs for new lines inside the preformatted text
        for (i in start + 1 until length) {
            if (this[i] == NBSP) {
                setSpan(ExtraCharacterSpan(), i, i + 1, Spanned.SPAN_INCLUSIVE_EXCLUSIVE)
            }
        }
    }

    private fun SpannableStringBuilder.parseListItem(element: Element) {
        val gapWidth = styleConfig.bulletList.bulletGapWidth.roundToInt()
        val bulletRadius = styleConfig.bulletList.bulletRadius.roundToInt()

        val listParent = element.parents().find { it.tagName() == "ul" || it.tagName() == "ol" }
        val span = when (listParent?.tagName()) {
            "ul" -> UnorderedListSpan(gapWidth, bulletRadius)
            "ol" -> {
                val typeface = Typeface.defaultFromStyle(Typeface.NORMAL)
                val textSize = 16.dpToPx()
                val order = (element.parent()?.select("li")?.indexOf(element) ?: 0) + 1
                OrderedListSpan(typeface, textSize, order, gapWidth)
            }

            else -> return
        }
        addLeadingLineBreakForBlockNode(element)
        inSpans(span) {
            parseChildren(element)
        }
    }

    private fun SpannableStringBuilder.parseInlineCode(element: Element) {
        if (element.parents().none { it.tagName() == "pre" }) {
            inSpans(InlineCodeSpan(relativeSizeProportion = styleConfig.inlineCode.relativeTextSize)) {
                parseChildren(element)
            }
        } else {
            parseChildren(element)
        }
    }

    private fun SpannableStringBuilder.parseLink(element: Element) {
        val start = this.length
        val innerText = element.text()
        if (innerText.isEmpty()) return

        val url = element.attr("href") ?: return
        val data = buildMap<String, String> {
            for (attr in element.attributes()) {
                set(attr.key, attr.value)
            }
        }
        val isMention = isMention?.invoke(innerText, url) == true || data.containsKey("data-mention-type")

        val textDisplay = if (isMention) {
            mentionDisplayHandler?.resolveMentionDisplay(innerText, url) ?: TextDisplay.Plain
        } else {
            TextDisplay.Plain
        }

        val span = when (textDisplay) {
            is TextDisplay.Custom ->  CustomMentionSpan(textDisplay.customSpan, url)
            TextDisplay.Pill -> {
                val pillBackground = styleConfig.pill.backgroundColor
                PillSpan(pillBackground, url)
            }
            TextDisplay.Plain -> LinkSpan(url)
        }
        inSpans(span) {
            parseChildren(element)

            // If the link is a mention, tag all but the first character of the anchor text with
            // ExtraCharacterSpans. These characters will then be taken into account when translating
            // between editor and composer model indices (see [EditorIndexMapper]).
            if (isMention && this.length > 1) {
                setSpan(
                    ExtraCharacterSpan(), start + 1, this.length, Spanned.SPAN_EXCLUSIVE_EXCLUSIVE
                )
            }
        }
    }

    // endregion

    // region: Utils for whitespaces and indexes

    /**
     * Either add an extra NBSP character if missing in the current block element, or, if a NBSP
     * character exists, set it as extra character so it's ignored when mapping indexes.
     */
    private fun SpannableStringBuilder.handleNbspInBlock(element: Element, start: Int, end: Int) {
        if (!element.isBlock) return

        if (element.childNodes().isEmpty()) {
            this.append(NBSP)
            setSpan(ExtraCharacterSpan(), end - 1, end, Spanned.SPAN_INCLUSIVE_EXCLUSIVE)
        } else if (end - start == 1 && this.getOrNull(start) in listOf(' ', NBSP)) {
            setSpan(ExtraCharacterSpan(), start, end, Spanned.SPAN_INCLUSIVE_EXCLUSIVE)
        }
    }

    private fun SpannableStringBuilder.addLeadingLineBreakForBlockNode(element: Element) {
        if (element.isBlock && element.previousElementSibling() != null) {
            append('\n')
        }
    }

    // endregion

    private fun Int.dpToPx(): Float {
        return resourcesHelper.dpToPx(this)
    }

    private fun Editable.addAtRoomSpans() {
        val display = mentionDisplayHandler?.resolveAtRoomMentionDisplay() ?: return
        Regex(Regex.escape("@room")).findAll(this).forEach eachMatch@{ match ->
            val start = match.range.first
            val end = match.range.last + 1
            if (getSpans(start, end, PlainAtRoomMentionDisplaySpan::class.java).isNotEmpty()) {
                return@eachMatch
            }
            val span = when (display) {
                is TextDisplay.Custom -> CustomMentionSpan(display.customSpan)
                TextDisplay.Pill -> PillSpan(
                    styleConfig.pill.backgroundColor
                )

                TextDisplay.Plain -> null
            }
            setSpan(span, start, end, Spanned.SPAN_EXCLUSIVE_EXCLUSIVE)
        }
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
            PillSpan::class.java,
            CustomMentionSpan::class.java,

            // Lists
            UnorderedListSpan::class.java,
            OrderedListSpan::class.java,

            ExtraCharacterSpan::class.java,

            // Blocks
            CodeBlockSpan::class.java,
            QuoteSpan::class.java,
        )

        fun Editable.removeFormattingSpans() = spans.flatMap { type ->
            getSpans(0, length, type).toList()
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

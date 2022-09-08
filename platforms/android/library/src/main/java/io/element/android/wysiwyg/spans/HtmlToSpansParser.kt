package io.element.android.wysiwyg.spans

import android.content.Context
import android.graphics.Typeface
import android.text.NoCopySpan
import android.text.SpannableString
import android.text.SpannableStringBuilder
import android.text.Spanned
import android.text.style.BulletSpan
import android.text.style.StrikethroughSpan
import android.text.style.StyleSpan
import android.text.style.UnderlineSpan
import androidx.core.text.getSpans
import io.element.android.wysiwyg.InlineFormat
import org.ccil.cowan.tagsoup.Parser
import org.xml.sax.Attributes
import org.xml.sax.ContentHandler
import org.xml.sax.InputSource
import org.xml.sax.Locator
import java.io.StringReader
import kotlin.math.roundToInt

class HtmlToSpansParser(
    private val context: Context,
    private val html: String,
): ContentHandler {

    data class Hyperlink(val link: String)
    object OrderedListBlock
    object UnorderedListBlock
    class ZeroWidthLineBreak: NoCopySpan
    data class ListItem(val ordered: Boolean, val order: Int? = null)

    private val parser = Parser().also { it.contentHandler = this }
    private val text = SpannableStringBuilder()

    fun convert(): Spanned {
        parser.parse(InputSource(StringReader(html)))
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
        for (i in start until start+length) {
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
                val mark = if (name == "ol") OrderedListBlock else UnorderedListBlock
                text.setSpan(mark, text.length, text.length, Spanned.SPAN_INCLUSIVE_EXCLUSIVE)
            }
            "li" -> {
                val lastListBlock = getLast<OrderedListBlock>() ?: getLast<UnorderedListBlock>() ?: return
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
                val mark = if (name == "ol") OrderedListBlock else UnorderedListBlock
                val last = getLast(mark::class.java) ?: return
                text.removeSpan(last)
            }
            "li" -> {
                val last = getLast<ListItem>() ?: return
                val start = text.getSpanStart(last)
                var lineBreakAdded = false
                // We only add line breaks *after* a previous <li> element if there is not already a line break
                if (start == 0) {
                    val zeroWidthSpan = SpannableString("\u200b").apply {
                        setSpan(ZeroWidthLineBreak(), 0, 1, Spanned.SPAN_EXCLUSIVE_EXCLUSIVE)
                    }
                    text.insert(0, zeroWidthSpan)
                } else if (start > 0 && start <= text.length && text[start-1] != '\n') {
                    // We add a line break and an zero width character to actually display the list item
                    val zeroWidthLineBreakSpan = SpannableString("\n").apply {
                        setSpan(ZeroWidthLineBreak(), 0, 1, Spanned.SPAN_EXCLUSIVE_EXCLUSIVE)
                    }
                    text.insert(start, zeroWidthLineBreakSpan)
                    lineBreakAdded = true
                }
                val newStart = if (lineBreakAdded) start+1 else start
                // TODO: provide gap width, typeface and textSize somehow
                val gapWidth = (10f * context.resources.displayMetrics.density).roundToInt()
                val span = if (last.ordered) {
                    val typeface = Typeface.defaultFromStyle(Typeface.NORMAL)
                    val textSize = 16f * context.resources.displayMetrics.scaledDensity
                    OrderedListSpan(typeface, textSize, last.order ?: 1, gapWidth)
                } else {
                    BulletSpan(gapWidth)
                }
                text.setSpan(span, newStart, text.length, Spanned.SPAN_EXCLUSIVE_EXCLUSIVE)
                text.removeSpan(last)
            }
        }
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
            InlineFormat.InlineCode -> InlineCodeSpan(context)
        }
        setSpanFromMark(format, span)
    }

    private fun handleHyperlinkStart(url: String) {
        val hyperlink = Hyperlink(url)
        text.setSpan(hyperlink, text.length, text.length, Spanned.SPAN_INCLUSIVE_EXCLUSIVE)
    }
    private fun handleHyperlinkEnd() {
        val last = getLast<Hyperlink>() ?: return
        // TODO: use custom link span maybe
        val span = UnderlineSpan()
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

    private inline fun <reified T: Any> getLast(from: Int = 0, to: Int = text.length): T? {
        val spans = text.getSpans<T>(from, to)
        return spans.lastOrNull()
    }

    private fun <T: Any> getLast(kind: Class<T>, from: Int = 0, to: Int = text.length): T? {
        val spans = text.getSpans(from, to, kind)
        return spans.lastOrNull()
    }

}

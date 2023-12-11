package io.element.android.wysiwyg.compose.text

import androidx.compose.ui.text.AnnotatedString
import androidx.compose.ui.text.ExperimentalTextApi
import androidx.compose.ui.text.ParagraphStyle
import androidx.compose.ui.text.SpanStyle
import androidx.compose.ui.text.UrlAnnotation
import androidx.compose.ui.text.font.FontFamily
import androidx.compose.ui.text.font.FontStyle
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.LineBreak
import androidx.compose.ui.text.style.TextDecoration
import androidx.compose.ui.text.style.TextIndent
import androidx.compose.ui.unit.TextUnit
import androidx.compose.ui.unit.TextUnitType
import io.element.android.wysiwyg.compose.RichTextEditorStyle
import io.element.android.wysiwyg.compose.internal.Mention
import kotlinx.collections.immutable.ImmutableList
import kotlinx.collections.immutable.toImmutableList
import org.jsoup.Jsoup
import org.jsoup.nodes.Element
import org.jsoup.nodes.Node
import org.jsoup.nodes.TextNode

class HtmlToComposeTextParser(
    private val richTextEditorStyle: RichTextEditorStyle,
    private val getLinkMention: (text: String, url: String) -> Mention?,
) {

    private val mentions = mutableListOf<Mention>()
    private var currentIndentation: TextUnit = TextUnit(0f, TextUnitType.Sp)

    private var parseChildren = false

    fun parse(html: String): ParsingResult {
        val document = Jsoup.parse(html)
        val annotatedString = buildSafeAnnotatedString {
            for (child in document.children()) {
                processNode(child)
            }
        }
        return ParsingResult(annotatedString, mentions.toImmutableList())
    }

    private fun SafeAnnotatedStringBuilder.processNode(node: Node) {
        when (node) {
            is Element -> {
                processElement(node)
            }
            is TextNode -> {
                val hasPreParent = node.hasAncestorWithTagName("pre")
                val plainText = node.wholeText.apply {
                    if (!hasPreParent) {
                        replace("\n", " ")
                    }
                }
                append(plainText)
            }
            else -> Unit
        }
    }

    private fun SafeAnnotatedStringBuilder.processElement(element: Element) {
        val tagName = element.tagName().lowercase()
        parseChildren = true

        val toPop = onElementStart(element, tagName)

        if (parseChildren) {
            for (child in element.childNodes()) {
                processNode(child)
            }
        }

        onElementEnd(element, tagName, toPop)
    }

    @OptIn(ExperimentalTextApi::class)
    private fun SafeAnnotatedStringBuilder.onElementStart(element: Element, tagName: String): Int {
        var toPop = 0
        when (tagName) {
            "b", "strong" -> {
                pushStyle(SpanStyle(fontWeight = FontWeight.Bold))
                toPop += 1
            }
            "i", "em" -> {
                pushStyle(SpanStyle(fontStyle = FontStyle.Italic))
                toPop += 1
            }
            "u" -> {
                pushStyle(SpanStyle(textDecoration = TextDecoration.Underline))
                toPop += 1
            }
            "s", "strike" -> {
                pushStyle(SpanStyle(textDecoration = TextDecoration.LineThrough))
                toPop += 1
            }
            "code" -> {
                if (element.parent()?.tagName()?.lowercase() != "pre") {
                    pushStringAnnotation(tagName, "")
                    toPop += 1
                }
                pushStyle(SpanStyle(fontFamily = FontFamily.Monospace))
                toPop += 1
            }
            "p" -> {
                if (element.elementSiblingIndex() != 0) {
                    append("\n")
                }
            }
            "br" -> {
                append("\n")
            }
            "li" -> {
                val parentTag = element.parent()?.tagName()?.lowercase()
                currentIndentation += richTextEditorStyle.indentation.listItem
                pushStyle(ParagraphStyle(textIndent = TextIndent(currentIndentation, currentIndentation), lineBreak = LineBreak.Simple))
                toPop += 1
                when (parentTag) {
                    "ul" -> {
                        pushStringAnnotation(parentTag, "")
                        toPop += 1
                    }
                    "ol" -> {
                        pushStringAnnotation(parentTag, "${element.elementSiblingIndex() + 1}")
                        toPop += 1
                    }
                }
            }
            "blockquote" -> {
                pushStringAnnotation(tagName, "")
                currentIndentation += richTextEditorStyle.indentation.quote
                pushStyle(ParagraphStyle(textIndent = TextIndent(currentIndentation, currentIndentation), lineBreak = LineBreak.Simple))
                toPop += 2
            }
            "pre" -> {
                pushStringAnnotation(tagName, "")
                currentIndentation += richTextEditorStyle.indentation.codeBlock
                pushStyle(ParagraphStyle(textIndent = TextIndent(currentIndentation, currentIndentation), lineBreak = LineBreak.Simple))
                toPop += 2
            }
            "a" -> {
                val text = element.text()
                val url = element.attr("href")
                when (val mention = getLinkMention(text, url)) {
                    is Mention.User -> {
                        parseChildren = false
                        mentions += mention
                        appendInlineContent("mention:${mention.link}", mention.text)
                    }
                    is Mention.Room -> {
                        parseChildren = false
                        mentions += mention
                        appendInlineContent("mention:${mention.link}", mention.text)
                    }
                    is Mention.NotifyEveryone -> {
                        parseChildren = false
                        mentions += mention
                        appendInlineContent("mention:@room", "@room")
                    }
                    is Mention.SlashCommand -> Unit
                    null -> {
                        pushStyle(SpanStyle(textDecoration = TextDecoration.Underline, color = richTextEditorStyle.link.color))
                        pushUrlAnnotation(UrlAnnotation(url = url))
                        toPop += 2
                    }
                }
            }
        }
        return toPop
    }

    private fun SafeAnnotatedStringBuilder.onElementEnd(element: Element, tagName: String, toPop: Int) {
        fun popStyles() {
            for (i in 0 until toPop) {
                pop()
            }
        }

        when (tagName) {
            "li" -> {
                popStyles()
                currentIndentation -= richTextEditorStyle.indentation.listItem
                restoreLastParagraphStyle()
            }
            "blockquote" -> {
                popStyles()
                currentIndentation -= richTextEditorStyle.indentation.quote
                restoreLastParagraphStyle()
            }
            "pre" -> {
                popStyles()
                currentIndentation -= richTextEditorStyle.indentation.codeBlock
                restoreLastParagraphStyle()
            }
            else -> popStyles()
        }
    }
}

private fun Node.hasAncestorWithTagName(tagName: String): Boolean {
    val parent = parent() ?: return false
    return if (parent is Element && parent.tagName().lowercase() == tagName) {
        true
    } else {
        parent.hasAncestorWithTagName(tagName)
    }
}

operator fun TextUnit.plus(other: TextUnit): TextUnit {
    return when {
        this.type == TextUnitType.Sp && other.type == TextUnitType.Sp -> TextUnit(this.value + other.value, TextUnitType.Sp)
        this.type == TextUnitType.Em && other.type == TextUnitType.Em -> TextUnit(this.value + other.value, TextUnitType.Em)
        else -> error("Cannot sum TextUnits with different types")
    }
}

operator fun TextUnit.minus(other: TextUnit): TextUnit {
    return when {
        this.type == TextUnitType.Sp && other.type == TextUnitType.Sp -> TextUnit(this.value - other.value, TextUnitType.Sp)
        this.type == TextUnitType.Em && other.type == TextUnitType.Em -> TextUnit(this.value - other.value, TextUnitType.Em)
        else -> error("Cannot subtract TextUnits with different types")
    }
}

data class ParsingResult(
    val annotatedString: AnnotatedString,
    val mentions: ImmutableList<Mention>,
)

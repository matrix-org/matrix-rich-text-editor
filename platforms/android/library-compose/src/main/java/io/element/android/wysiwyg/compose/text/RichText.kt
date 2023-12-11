package io.element.android.wysiwyg.compose.text

import androidx.compose.foundation.text.InlineTextContent
import androidx.compose.foundation.text.selection.SelectionContainer
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.drawWithCache
import androidx.compose.ui.geometry.CornerRadius
import androidx.compose.ui.geometry.Offset
import androidx.compose.ui.geometry.RoundRect
import androidx.compose.ui.geometry.Size
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.Path
import androidx.compose.ui.graphics.drawscope.DrawScope
import androidx.compose.ui.graphics.drawscope.DrawStyle
import androidx.compose.ui.graphics.drawscope.Fill
import androidx.compose.ui.graphics.drawscope.Stroke
import androidx.compose.ui.text.AnnotatedString
import androidx.compose.ui.text.TextLayoutResult
import androidx.compose.ui.text.TextMeasurer
import androidx.compose.ui.text.drawText
import androidx.compose.ui.text.rememberTextMeasurer
import androidx.compose.ui.unit.TextUnit
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import io.element.android.wysiwyg.compose.RichTextEditorDefaults
import io.element.android.wysiwyg.compose.RichTextEditorStyle

@Composable
fun RichText(
    text: AnnotatedString,
    modifier: Modifier = Modifier,
    inlineContent: Map<String, InlineTextContent> = emptyMap(),
    style: RichTextEditorStyle = RichTextEditorDefaults.style(),
) {
    var textLayout by remember { mutableStateOf<TextLayoutResult?>(null) }
    val textMeasurer = rememberTextMeasurer()
    SelectionContainer {
        Text(
            text = text,
            inlineContent = inlineContent,
            onTextLayout = {
                textLayout = it
            },
            modifier = modifier.drawWithCache {
                val allAnnotations = text.getStringAnnotations(0, text.length)
                val indentSources = text.getIndentSources(style)
                onDrawWithContent {
                    val layout = textLayout
                    if (layout != null) {
                        for ((index, annotation) in allAnnotations.withIndex()) {
                            when (annotation.tag) {
                                "ul" -> {
                                    val extraIndentation = indentationToSubtract(
                                        index,
                                        annotation.start,
                                        indentSources
                                    )
                                    drawUnorderedListItem(
                                        annotation,
                                        layout,
                                        style,
                                        extraIndentation.toPx()
                                    )
                                }

                                "ol" -> {
                                    val extraIndentation = indentationToSubtract(
                                        index,
                                        annotation.start,
                                        indentSources
                                    )
                                    drawOrderedListItem(
                                        annotation,
                                        textMeasurer,
                                        layout,
                                        style,
                                        extraIndentation.toPx()
                                    )
                                }

                                "blockquote" -> {
                                    val extraIndentation = indentationToSubtract(
                                        index,
                                        annotation.start,
                                        indentSources
                                    )
                                    drawQuote(annotation, layout, style, extraIndentation.toPx())
                                }

                                "pre" -> {
                                    val extraIndentation = indentationToSubtract(
                                        index,
                                        annotation.start,
                                        indentSources
                                    )
                                    drawCodeBlock(
                                        annotation,
                                        layout,
                                        style,
                                        extraIndentation.toPx()
                                    )
                                }

                                "code" -> {
                                    drawInlineCode(annotation, layout, style)
                                }

                                else -> Unit
                            }
                        }
                    }
                    drawContent()
                }
            },
        )
    }
}

private fun DrawScope.drawRoundRect(
    color: Color,
    roundRect: RoundRect,
    style: DrawStyle,
) {
    val path = Path().apply {
        addRoundRect(roundRect)
    }
    drawPath(
        path = path,
        color = color,
        style = style,
    )
}

private fun AnnotatedString.getIndentSources(style: RichTextEditorStyle): List<IndentSource> {
    val result = mutableListOf<IndentSource>()
    val annotations = getStringAnnotations(0, length)
    for ((index, annotation) in annotations.withIndex()) {
        when (annotation.tag) {
            "ol", "ul" -> {
                result.add(IndentSource(index, annotation.start, style.indentation.listItem))
            }
            "blockquote" -> {
                result.add(IndentSource(index, annotation.start, style.indentation.quote))
            }
            "pre" -> {
                result.add(IndentSource(index, annotation.start, style.indentation.codeBlock))
            }
            else -> Unit
        }
    }
    return result
}

private fun DrawScope.drawUnorderedListItem(
    annotation: AnnotatedString.Range<String>,
    layout: TextLayoutResult,
    style: RichTextEditorStyle,
    extraIndentation: Float = 0f,
) {
    val bulletStyle = style.bulletList
    val radius = bulletStyle.bulletRadius.roundToPx()
    val gap = bulletStyle.bulletGapWidth.roundToPx()

    val rect = layout.getBoundingBox(annotation.start)
    val center = rect.left - radius - gap - extraIndentation

    drawCircle(
        color = style.text.color,
        radius = radius.toFloat(),
        center = Offset(center, rect.center.y)
    )
}

private fun DrawScope.drawOrderedListItem(
    annotation: AnnotatedString.Range<String>,
    textMeasurer: TextMeasurer,
    layout: TextLayoutResult,
    style: RichTextEditorStyle,
    extraIndentation: Float = 0f,
) {
    val rect = layout.getBoundingBox(annotation.start)
    val result = textMeasurer.measure("${annotation.item}. ")
    drawText(
        textLayoutResult = result,
        color = style.text.color,
        topLeft = Offset(rect.left - result.size.width - extraIndentation, rect.bottom - result.size.height)
    )
}

private fun DrawScope.drawQuote(
    annotation: AnnotatedString.Range<String>,
    layout: TextLayoutResult,
    style: RichTextEditorStyle,
    extraIndentation: Float = 0f,
) {
    val start = layout.getCursorRect(annotation.start).left
    val rect = layout.getPathForRange(annotation.start, annotation.end).getBounds()
    val size = Size(4.dp.roundToPx().toFloat(), rect.height) // TODO use style
    drawRect(
        color = Color.Gray, // TODO use style
        topLeft = Offset(start - 8.dp.roundToPx() - extraIndentation, rect.top),
        size = size
    )
}

private fun DrawScope.drawCodeBlock(
    annotation: AnnotatedString.Range<String>,
    layout: TextLayoutResult,
    style: RichTextEditorStyle,
    extraIndentation: Float = 0f,
) {
    val start = layout.getCursorRect(annotation.start).left
    val rect = layout.getPathForRange(annotation.start, annotation.end).getBounds()
    val codeBlockPadding = style.codeBlock.verticalPadding.roundToPx()
    val cornerRadius = style.codeBlock.background.cornerRadiusTopLeft.roundToPx().toFloat()

    val offset = Offset(start - codeBlockPadding - extraIndentation, rect.top - codeBlockPadding)
    val size = Size(layout.size.width.toFloat() + codeBlockPadding * 2 - start, rect.height + codeBlockPadding * 2)
    drawRoundRect(
        color = style.codeBlock.background.color,
        topLeft = offset,
        size = size,
        cornerRadius = CornerRadius(cornerRadius, cornerRadius),
        style = Fill,
    )

    drawRoundRect(
        color = style.codeBlock.background.borderColor,
        topLeft = offset,
        size = size,
        cornerRadius = CornerRadius(cornerRadius, cornerRadius),
        style = Stroke(
            width = style.codeBlock.background.borderWidth.roundToPx().toFloat(),
        ),
    )
}

private fun DrawScope.drawInlineCode(
    annotation: AnnotatedString.Range<String>,
    layout: TextLayoutResult,
    style: RichTextEditorStyle,
) {
    val lineStart = layout.getLineForOffset(annotation.start)
    val lineEnd = layout.getLineForOffset(annotation.end)
    val isSameLine = lineStart == lineEnd
    val start = layout.getCursorRect(annotation.start).left
    val end = layout.getCursorRect(annotation.end).right
    val radius = style.inlineCode.background.singleLine.cornerRadiusTopLeft.roundToPx().toFloat()
    if (isSameLine) {
        drawRoundRect(
            color = style.inlineCode.background.singleLine.color,
            topLeft = Offset(start, layout.getLineTop(lineStart)),
            size = Size(end - start, layout.getLineBottom(lineStart) - layout.getLineTop(lineStart)),
            cornerRadius = CornerRadius(radius, radius),
            style = Fill,
        )
        drawRoundRect(
            color = style.inlineCode.background.singleLine.borderColor,
            topLeft = Offset(start, layout.getLineTop(lineStart)),
            size = Size(end - start, layout.getLineBottom(lineStart) - layout.getLineTop(lineStart)),
            cornerRadius = CornerRadius(radius, radius),
            style = Stroke(style.inlineCode.background.singleLine.borderWidth.roundToPx().toFloat()),
        )
    } else {
        for (line in lineStart..lineEnd) {
            when (line) {
                lineStart -> {
                    val roundRect = RoundRect(
                        left = start,
                        top = layout.getLineTop(line),
                        right = layout.size.width.toFloat(),
                        bottom = layout.getLineBottom(line),
                        topLeftCornerRadius = CornerRadius(radius, radius),
                        bottomLeftCornerRadius = CornerRadius(radius, radius),
                        topRightCornerRadius = CornerRadius.Zero,
                        bottomRightCornerRadius = CornerRadius.Zero,
                    )
                    drawRoundRect(
                        roundRect = roundRect,
                        color = style.inlineCode.background.singleLine.color,
                        style = Fill,
                    )
                    drawRoundRect(
                        roundRect = roundRect,
                        color = style.inlineCode.background.singleLine.borderColor,
                        style = Stroke(style.inlineCode.background.singleLine.borderWidth.roundToPx().toFloat()),
                    )
                }
                lineEnd -> {
                    val roundRect = RoundRect(
                        left = layout.getLineLeft(line),
                        top = layout.getLineTop(line),
                        right = end,
                        bottom = layout.getLineBottom(line),
                        topLeftCornerRadius = CornerRadius.Zero,
                        bottomLeftCornerRadius = CornerRadius.Zero,
                        topRightCornerRadius = CornerRadius(radius, radius),
                        bottomRightCornerRadius = CornerRadius(radius, radius),
                    )
                    drawRoundRect(
                        roundRect = roundRect,
                        color = style.inlineCode.background.singleLine.color,
                        style = Fill,
                    )
                    drawRoundRect(
                        roundRect = roundRect,
                        color = style.inlineCode.background.singleLine.borderColor,
                        style = Stroke(style.inlineCode.background.singleLine.borderWidth.roundToPx().toFloat()),
                    )
                }
                else -> {
                    drawRect(
                        color = style.inlineCode.background.singleLine.color,
                        topLeft = Offset(
                            layout.getLineLeft(line),
                            layout.getLineTop(line)
                        ),
                        size = Size(layout.size.width.toFloat(), layout.getLineBottom(line) - layout.getLineTop(line)),
                    )
                    drawRect(
                        color = style.inlineCode.background.singleLine.borderColor,
                        topLeft = Offset(layout.getLineLeft(line), layout.getLineTop(line)),
                        size = Size(layout.size.width.toFloat(), layout.getLineBottom(line) - layout.getLineTop(line)),
                        style = Stroke(style.inlineCode.background.singleLine.borderWidth.roundToPx().toFloat()),
                    )
                }
            }
        }
    }
}

private fun indentationToSubtract(index: Int, positionInText: Int, indentSources: List<IndentSource>): TextUnit {
    val indent = indentSources.filter { it.positionInText == positionInText && it.indexInAnnotations > index }
        .fold(0f) { acc, indentSource ->
            acc + indentSource.size.value
        }
    return indent.sp
}

data class IndentSource(
    val indexInAnnotations: Int,
    val positionInText: Int,
    val size: TextUnit,
)
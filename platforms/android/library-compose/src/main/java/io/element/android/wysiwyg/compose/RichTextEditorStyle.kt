package io.element.android.wysiwyg.compose

import androidx.annotation.DrawableRes
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.unit.Dp

data class RichTextEditorStyle internal constructor(
    val bulletList: BulletListStyle,
    val codeBlock: CodeBlockStyle,
    val inlineCode: InlineCodeStyle,
    val pill: PillStyle,
    val text: TextStyle,
    val cursor: CursorStyle,
    val link: LinkStyle,
)

data class BulletListStyle internal constructor(
    val bulletGapWidth: Dp,
    val bulletRadius: Dp,
)

data class CodeBlockStyle internal constructor(
    val leadingMargin: Dp,
    val verticalPadding: Dp,
    val relativeTextSize: Float,
    @DrawableRes
    val backgroundDrawable: Int,
)

data class InlineCodeStyle internal constructor(
    val horizontalPadding: Dp,
    val verticalPadding: Dp,
    val relativeTextSize: Float,
    @DrawableRes
    val singleLineBg: Int,
    @DrawableRes
    val multiLineBgLeft: Int,
    @DrawableRes
    val multiLineBgMid: Int,
    @DrawableRes
    val multiLineBgRight: Int,
)

data class PillStyle(
    val backgroundColor: Color,
)

data class TextStyle(
    val color: Color,
)

data class CursorStyle(
    val color: Color,
)

data class LinkStyle(
    val color: Color,
)

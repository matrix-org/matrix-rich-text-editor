package io.element.android.wysiwyg.compose

import android.graphics.drawable.GradientDrawable
import androidx.compose.runtime.Immutable
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.toArgb
import androidx.compose.ui.unit.Density
import androidx.compose.ui.unit.Dp
import kotlin.math.roundToInt

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
    val background: CodeBackgroundStyle,
)

data class InlineCodeStyle internal constructor(
    val horizontalPadding: Dp,
    val verticalPadding: Dp,
    val relativeTextSize: Float,
    val background: InlineCodeBackgroundStyle,
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

@Immutable
data class CodeBackgroundStyle(
    val density: Density,
    val color: Color,
    val borderColor: Color,
    val cornerRadiusTopLeft: Dp,
    val cornerRadiusTopRight: Dp,
    val cornerRadiusBottomLeft: Dp,
    val cornerRadiusBottomRight: Dp,
    val borderWidth: Dp,
) {
    internal val drawable by lazy {
        GradientDrawable().apply {
            shape = GradientDrawable.RECTANGLE
            setColor(this@CodeBackgroundStyle.color.toArgb())
            setStroke(
                with(density) { borderWidth.toPx() }.roundToInt(),
                borderColor.toArgb()
            )
            cornerRadii = with(density) {
                floatArrayOf(
                    cornerRadiusTopLeft.toPx(), cornerRadiusTopLeft.toPx(),
                    cornerRadiusTopRight.toPx(), cornerRadiusTopRight.toPx(),
                    cornerRadiusBottomRight.toPx(), cornerRadiusBottomRight.toPx(),
                    cornerRadiusBottomLeft.toPx(), cornerRadiusBottomLeft.toPx()
                )
            }
        }
    }
}

data class InlineCodeBackgroundStyle(
    val singleLine: CodeBackgroundStyle,
    val multiLineLeft: CodeBackgroundStyle,
    val multiLineMiddle: CodeBackgroundStyle,
    val multiLineRight: CodeBackgroundStyle,
)

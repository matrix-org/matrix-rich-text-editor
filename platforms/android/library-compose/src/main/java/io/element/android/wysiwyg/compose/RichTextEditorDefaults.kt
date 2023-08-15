package io.element.android.wysiwyg.compose

import io.element.android.wysiwyg.R as BaseR
import androidx.annotation.DrawableRes
import androidx.compose.material3.MaterialTheme
import androidx.compose.runtime.Composable
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.unit.Dp
import androidx.compose.ui.unit.dp

/**
 * Default config for the [RichTextEditor] composable.
 */
object RichTextEditorDefaults {

    /**
     * Creates the default set of style customisations for [RichTextEditor].
     *
     * @param bulletList A custom style for bullet lists.
     * @param codeBlock A custom style for code blocks.
     * @param inlineCode A custom style for inline code.
     * @param pill A custom style for pills.
     * @param textColor Color of the text displayed in the editor.
     * @param cursorDrawable Color of the cursor displayed in the editor.
     *                    Only supported on API 29 and above.
     */
    @Composable
    fun style(
        bulletList: BulletListStyle = bulletListStyle(),
        codeBlock: CodeBlockStyle = codeBlockStyle(),
        inlineCode: InlineCodeStyle = inlineCodeStyle(),
        pill: PillStyle = pillStyle(),
        text: TextStyle = textStyle(),
        cursor: CursorStyle = cursorStyle(),
    ): RichTextEditorStyle = RichTextEditorStyle(
        bulletList = bulletList,
        codeBlock = codeBlock,
        inlineCode = inlineCode,
        pill = pill,
        text = text,
        cursor = cursor,
    )

    /**
     * Creates the default bullet list style for [RichTextEditor].
     *
     * @param bulletGapWidth The gap width between the bullet and the text.
     * @param bulletRadius The radius of the bullet.
     */
    fun bulletListStyle(
        bulletGapWidth: Dp = 2.dp,
        bulletRadius: Dp = 2.dp,
    ) = BulletListStyle(
        bulletGapWidth = bulletGapWidth,
        bulletRadius = bulletRadius,
    )

    /**
     * Creates the default code block style for [RichTextEditor].
     *
     * @param leadingMargin The leading margin to apply
     * @param verticalPadding The vertical padding to apply
     * @param relativeTextSize The relative font scale to apply to code text
     * @param backgroundDrawable The background drawable to use
     */
    fun codeBlockStyle(
        leadingMargin: Dp = 16.dp,
        verticalPadding: Dp = 8.dp,
        relativeTextSize: Float = 0.85f,
        @DrawableRes
        backgroundDrawable: Int = BaseR.drawable.code_block_bg,
    ) = CodeBlockStyle(
        leadingMargin = leadingMargin,
        verticalPadding = verticalPadding,
        relativeTextSize = relativeTextSize,
        backgroundDrawable = backgroundDrawable,
    )

    /**
     * Creates the default inline code style for [RichTextEditor].
     *
     * @param horizontalPadding The horizontal padding to apply
     * @param verticalPadding The vertical padding to apply
     * @param relativeTextSize The relative font scale to apply to code text
     * @param singleLineBg The background drawable to apply for single line code blocks
     * @param multiLineBgLeft The background drawable to apply for the left side of multi line inline code
     * @param multiLineBgMid The background drawable to apply for the middle of multi line inline code
     * @param multiLineBgRight The background drawable to apply for the right side of multi line inline code
     */
    fun inlineCodeStyle(
        horizontalPadding: Dp = 4.dp,
        verticalPadding: Dp = 2.dp,
        relativeTextSize: Float = 0.85f,
        @DrawableRes
        singleLineBg: Int = BaseR.drawable.inline_code_single_line_bg,
        @DrawableRes
        multiLineBgLeft: Int = BaseR.drawable.inline_code_multi_line_bg_left,
        @DrawableRes
        multiLineBgMid: Int = BaseR.drawable.inline_code_multi_line_bg_mid,
        @DrawableRes
        multiLineBgRight: Int = BaseR.drawable.inline_code_multi_line_bg_right,
    ) = InlineCodeStyle(
        horizontalPadding = horizontalPadding,
        verticalPadding = verticalPadding,
        relativeTextSize = relativeTextSize,
        singleLineBg = singleLineBg,
        multiLineBgLeft = multiLineBgLeft,
        multiLineBgMid = multiLineBgMid,
        multiLineBgRight = multiLineBgRight,
    )

    /**
     * Creates the default pill style for [RichTextEditor].
     *
     * @param backgroundColor The background color to apply
     */
    fun pillStyle(
        backgroundColor: Color = Color.Transparent,
    ) = PillStyle(
        backgroundColor = backgroundColor,
    )

    /**
     * Creates the default text style for [RichTextEditor].
     *
     * @param color The text color to apply
     */
    @Composable
    fun textStyle(
        color: Color = MaterialTheme.colorScheme.onSurface,
    ) = TextStyle(
        color = color,
    )

    /**
     * Creates the default cursor style for [RichTextEditor].
     *
     * @param color The color to apply to the cursor
     */
    @Composable
    fun cursorStyle(
        color: Color = MaterialTheme.colorScheme.primary,
    ) = CursorStyle(
        color = color,
    )
}

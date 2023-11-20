package io.element.android.wysiwyg.compose

import android.text.InputType
import androidx.compose.material3.MaterialTheme
import androidx.compose.runtime.Composable
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.platform.LocalDensity
import androidx.compose.ui.text.font.FontFamily
import androidx.compose.ui.text.font.FontStyle
import androidx.compose.ui.text.font.FontSynthesis
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.Dp
import androidx.compose.ui.unit.TextUnit
import androidx.compose.ui.unit.dp
import io.element.android.wysiwyg.display.TextDisplay

private val defaultCodeCornerRadius = 4.dp
private val defaultCodeBorderWidth = 1.dp


/**
 * Default config for the [RichTextEditor] composable.
 */
object RichTextEditorDefaults {
    internal const val initialLineCount = 1
    internal const val initialHtml = ""
    internal const val initialFocus = false
    internal val initialSelection = 0 to 0

    /**
     * Default [TextDisplay] for mentions: they will appear as plain text.
     */
    val MentionDisplay: (String, String) -> TextDisplay = { _, _ -> TextDisplay.Plain }

    /**
     * Default [TextDisplay] for `@room` mentions: they will appear as plain text.
     */
    val RoomMentionDisplay: () -> TextDisplay = { TextDisplay.Plain }

    /**
     * Creates the default set of style customisations for [RichTextEditor].
     *
     * @param bulletList A custom style for bullet lists.
     * @param codeBlock A custom style for code blocks.
     * @param inlineCode A custom style for inline code.
     * @param pill A custom style for pills.
     * @param text A custom style for text displayed in the editor.
     * @param cursor A custom style for the cursor for API 29 and above.
     * @param link A custom style for links.
     */
    @Composable
    fun style(
        bulletList: BulletListStyle = bulletListStyle(),
        codeBlock: CodeBlockStyle = codeBlockStyle(),
        inlineCode: InlineCodeStyle = inlineCodeStyle(),
        pill: PillStyle = pillStyle(),
        text: TextStyle = textStyle(),
        cursor: CursorStyle = cursorStyle(),
        link: LinkStyle = linkStyle(),
    ): RichTextEditorStyle = RichTextEditorStyle(
        bulletList = bulletList,
        codeBlock = codeBlock,
        inlineCode = inlineCode,
        pill = pill,
        text = text,
        cursor = cursor,
        link = link,
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
     * @param background The background style to apply
     */
    @Composable
    fun codeBlockStyle(
        leadingMargin: Dp = 16.dp,
        verticalPadding: Dp = 8.dp,
        relativeTextSize: Float = 0.85f,
        background: CodeBackgroundStyle = codeBlockBackgroundStyle(),
    ) = CodeBlockStyle(
        leadingMargin = leadingMargin,
        verticalPadding = verticalPadding,
        relativeTextSize = relativeTextSize,
        background = background,
    )

    /**
     * Creates the default inline code style for [RichTextEditor].
     *
     * @param horizontalPadding The horizontal padding to apply
     * @param verticalPadding The vertical padding to apply
     * @param relativeTextSize The relative font scale to apply to code text
     * @param background The background style to apply for single line code blocks
     */
    @Composable
    fun inlineCodeStyle(
        horizontalPadding: Dp = 4.dp,
        verticalPadding: Dp = 2.dp,
        relativeTextSize: Float = 0.85f,
        background: InlineCodeBackgroundStyle = inlineCodeBackgroundStyle(),
    ) = InlineCodeStyle(
        horizontalPadding = horizontalPadding,
        verticalPadding = verticalPadding,
        relativeTextSize = relativeTextSize,
        background = background,
    )

    /**
     * Creates the default pill style for [RichTextEditor].
     *
     * @param backgroundColor The background color to apply
     */
    @Composable
    fun pillStyle(
        backgroundColor: Color = MaterialTheme.colorScheme.surfaceVariant,
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
        fontSize: TextUnit = MaterialTheme.typography.bodyLarge.fontSize,
        fontFamily: FontFamily? = MaterialTheme.typography.bodyLarge.fontFamily,
        fontWeight: FontWeight? = MaterialTheme.typography.bodyLarge.fontWeight,
        fontStyle: FontStyle? = MaterialTheme.typography.bodyLarge.fontStyle,
        fontSynthesis: FontSynthesis? = MaterialTheme.typography.bodyLarge.fontSynthesis,
    ) = TextStyle(
        color = color,
        fontSize = fontSize,
        fontFamily = fontFamily,
        fontWeight = fontWeight,
        fontStyle = fontStyle,
        fontSynthesis = fontSynthesis,
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

    /**
     * Creates the default link style for [RichTextEditor].
     *
     * @param color The color to apply
     */
    @Composable
    fun linkStyle(
        color: Color = Color.Blue,
    ) = LinkStyle(
        color = color,
    )

    /**
     * Creates a default code block background.
     */
    @Composable
    fun codeBlockBackgroundStyle(
        color: Color = MaterialTheme.colorScheme.secondaryContainer,
        borderColor: Color = MaterialTheme.colorScheme.onSecondaryContainer,
        cornerRadius: Dp = defaultCodeCornerRadius,
        borderWidth: Dp = defaultCodeBorderWidth,
    ): CodeBackgroundStyle = CodeBackgroundStyle(
        density = LocalDensity.current,
        color = color,
        borderColor = borderColor,
        borderWidth = borderWidth,
        cornerRadiusTopLeft = cornerRadius,
        cornerRadiusTopRight = cornerRadius,
        cornerRadiusBottomRight = cornerRadius,
        cornerRadiusBottomLeft = cornerRadius,
    )

    /**
     * Creates a default inline code background.
     */
    @Composable
    fun inlineCodeBackgroundStyle(
        color: Color = MaterialTheme.colorScheme.secondaryContainer,
        borderColor: Color = MaterialTheme.colorScheme.onSecondaryContainer,
        cornerRadius: Dp = defaultCodeCornerRadius,
        borderWidth: Dp = defaultCodeBorderWidth
    ): InlineCodeBackgroundStyle {
        val density = LocalDensity.current
        return InlineCodeBackgroundStyle(
            singleLine = CodeBackgroundStyle(
                density = density,
                color = color,
                borderColor = borderColor,
                borderWidth = borderWidth,
                cornerRadiusTopLeft = cornerRadius,
                cornerRadiusTopRight = cornerRadius,
                cornerRadiusBottomRight = cornerRadius,
                cornerRadiusBottomLeft = cornerRadius,
            ),
            multiLineLeft = CodeBackgroundStyle(
                density = density,
                color = color,
                borderColor = borderColor,
                borderWidth = borderWidth,
                cornerRadiusTopLeft = cornerRadius,
                cornerRadiusTopRight = 0.dp,
                cornerRadiusBottomRight = 0.dp,
                cornerRadiusBottomLeft = cornerRadius,
            ),
            multiLineRight = CodeBackgroundStyle(
                density = density,
                color = color,
                borderColor = borderColor,
                borderWidth = borderWidth,
                cornerRadiusTopLeft = 0.dp,
                cornerRadiusTopRight = cornerRadius,
                cornerRadiusBottomRight = cornerRadius,
                cornerRadiusBottomLeft = 0.dp,
            ),
            multiLineMiddle = CodeBackgroundStyle(
                density = density,
                color = color,
                borderColor = borderColor,
                borderWidth = borderWidth,
                cornerRadiusTopLeft = 0.dp,
                cornerRadiusTopRight = 0.dp,
                cornerRadiusBottomRight = 0.dp,
                cornerRadiusBottomLeft = 0.dp,
            ),
        )
    }

    const val inputType: Int = InputType.TYPE_CLASS_TEXT or
            InputType.TYPE_TEXT_FLAG_MULTI_LINE or
            InputType.TYPE_TEXT_FLAG_CAP_SENTENCES or
            InputType.TYPE_TEXT_FLAG_AUTO_CORRECT or
            InputType.TYPE_TEXT_FLAG_AUTO_COMPLETE
}

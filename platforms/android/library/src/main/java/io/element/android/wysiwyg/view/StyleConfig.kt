package io.element.android.wysiwyg.view

import android.graphics.drawable.Drawable
import androidx.annotation.ColorRes
import androidx.annotation.Px
import io.element.android.wysiwyg.EditorEditText


/**
 * Style configuration for the [EditorEditText].
 *
 * @property bulletList A custom style for bullet lists.
 * @property codeBlock A custom style for code blocks.
 * @property inlineCode A custom style for inline code.
 * @property pill A custom style for pills.
 */
data class StyleConfig(
    val bulletList: BulletListStyleConfig,

    val inlineCode: InlineCodeStyleConfig,

    val codeBlock: CodeBlockStyleConfig,

    val pill: PillStyleConfig,
)

/**
 * Style configuration for the bullet list.
 *
 * @property bulletGapWidth The gap width between the bullet and the text.
 * @property bulletRadius The radius of the bullet.
 */
data class BulletListStyleConfig(
    @Px val bulletGapWidth: Float,
    @Px val bulletRadius: Float,
)

/**
 * Style configuration for the inline code.
 *
 * @property horizontalPadding The horizontal padding to apply
 * @property verticalPadding The vertical padding to apply
 * @property relativeTextSize The relative font scale to apply to code text
 * @property singleLineBg The background drawable to apply for single line code blocks
 * @property multiLineBgLeft The background drawable to apply for the left side of multi line inline code
 * @property multiLineBgMid The background drawable to apply for the middle of multi line inline code
 * @property multiLineBgRight The background drawable to apply for the right side of multi line inline code
 */
data class InlineCodeStyleConfig(
    @Px val horizontalPadding: Int,
    @Px val verticalPadding: Int,
    val relativeTextSize: Float,
    val singleLineBg: Drawable,
    val multiLineBgLeft: Drawable,
    val multiLineBgMid: Drawable,
    val multiLineBgRight: Drawable,
)

/**
 * Style configuration for code blocks.
 *
 * @property leadingMargin The leading margin to apply
 * @property verticalPadding The vertical padding to apply
 * @property relativeTextSize The relative font scale to apply to code text
 * @property backgroundDrawable The background drawable to use
 */
data class CodeBlockStyleConfig(
    @Px val leadingMargin: Int,
    @Px val verticalPadding: Int,
    val relativeTextSize: Float,
    val backgroundDrawable: Drawable,
)

/**
 * Style configuration for pills.
 *
 * @property backgroundColor The background color to apply
 */
data class PillStyleConfig(
    @ColorRes
    val backgroundColor: Int,
)

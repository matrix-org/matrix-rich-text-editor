package io.element.android.wysiwyg.view

import android.graphics.drawable.Drawable
import androidx.annotation.Px

internal data class StyleConfig(
    val bulletList: BulletListStyleConfig,

    val inlineCode: InlineCodeStyleConfig,

    val codeBlock: CodeBlockStyleConfig
)

data class BulletListStyleConfig(
    @Px val bulletGapWidth: Float,
    @Px val bulletRadius: Float,
)

data class InlineCodeStyleConfig(
    @Px val horizontalPadding: Int,
    @Px val verticalPadding: Int,
    val relativeTextSize: Float,
    val singleLineBg: Drawable,
    val multiLineBgLeft: Drawable,
    val multiLineBgMid: Drawable,
    val multiLineBgRight: Drawable,
)

data class CodeBlockStyleConfig(
    @Px val leadingMargin: Int,
    @Px val verticalPadding: Int,
    val relativeTextSize: Float,
    val backgroundDrawable: Drawable,
)
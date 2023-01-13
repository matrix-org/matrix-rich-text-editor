package io.element.android.wysiwyg.utils

import android.graphics.drawable.Drawable
import androidx.annotation.ColorInt
import androidx.annotation.Px

internal data class StyleConfig(
    // Unordered list
    @Px val bulletGapWidth: Float,
    @Px val bulletRadius: Float,

    // Inline code
    @Px val inlineCodeHorizontalPadding: Int,
    @Px val inlineCodeVerticalPadding: Int,
    val inlineCodeSingleLineBg: Drawable,
    val inlineCodeMultiLineBgLeft: Drawable,
    val inlineCodeMultiLineBgMid: Drawable,
    val inlineCodeMultiLineBgRight: Drawable,

    // Code blocks
    @Px val codeBlockLeadingMargin: Int,
    @Px val codeBlockVerticalPadding: Int,
    val codeBlockBackgroundDrawable: Drawable,
)

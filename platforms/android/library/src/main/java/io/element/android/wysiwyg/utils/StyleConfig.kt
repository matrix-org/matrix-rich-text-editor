package io.element.android.wysiwyg.utils

import android.graphics.drawable.Drawable
import androidx.annotation.Px

internal data class StyleConfig(
    @Px val bulletGapWidth: Float,
    @Px val bulletRadius: Float,
    @Px val inlineCodeHorizontalPadding: Int,
    @Px val inlineCodeVerticalPadding: Int,
    val inlineCodeSingleLineBg: Drawable,
    val inlineCodeMultiLineBgLeft: Drawable,
    val inlineCodeMultiLineBgMid: Drawable,
    val inlineCodeMultiLineBgRight: Drawable,
)
package io.element.android.wysiwyg.fakes

import android.graphics.drawable.ColorDrawable
import io.element.android.wysiwyg.utils.StyleConfig

internal fun createFakeStyleConfig() = StyleConfig(
    bulletGapWidth = 1f,
    bulletRadius = 1f,
    inlineCodeHorizontalPadding = 2,
    inlineCodeVerticalPadding = 2,
    inlineCodeSingleLineBg = ColorDrawable(),
    inlineCodeMultiLineBgLeft = ColorDrawable(),
    inlineCodeMultiLineBgMid = ColorDrawable(),
    inlineCodeMultiLineBgRight = ColorDrawable(),
)

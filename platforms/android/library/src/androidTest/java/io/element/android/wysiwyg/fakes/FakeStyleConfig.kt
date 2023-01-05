package io.element.android.wysiwyg.fakes

import android.content.Context
import android.graphics.drawable.ColorDrawable
import io.element.android.wysiwyg.utils.StyleConfig

internal fun createFakeStyleConfig(context: Context) = StyleConfig(
    bulletGapWidth = 1f,
    bulletRadius = 1f,
    inlineCodeHorizontalPadding = 2,
    inlineCodeVerticalPadding = 2,
    inlineCodeSingleLineBg = ColorDrawable(),
    inlineCodeMultiLineBgLeft = ColorDrawable(),
    inlineCodeMultiLineBgMid = ColorDrawable(),
    inlineCodeMultiLineBgRight = ColorDrawable(),
)

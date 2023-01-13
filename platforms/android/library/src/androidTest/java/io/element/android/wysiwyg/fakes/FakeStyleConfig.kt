package io.element.android.wysiwyg.fakes

import android.graphics.drawable.ColorDrawable
import io.element.android.wysiwyg.utils.StyleConfig

private val fakeDrawable = ColorDrawable()

internal fun createFakeStyleConfig() = StyleConfig(
    bulletGapWidth = 1f,
    bulletRadius = 1f,
    inlineCodeHorizontalPadding = 2,
    inlineCodeVerticalPadding = 2,
    inlineCodeSingleLineBg = fakeDrawable,
    inlineCodeMultiLineBgLeft = fakeDrawable,
    inlineCodeMultiLineBgMid = fakeDrawable,
    inlineCodeMultiLineBgRight = fakeDrawable,
    codeBlockLeadingMargin = 0,
    codeBlockVerticalPadding = 0,
    codeBlockBackgroundDrawable = fakeDrawable,
)

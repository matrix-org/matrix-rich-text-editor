package io.element.android.wysiwyg.fakes

import android.graphics.drawable.ColorDrawable
import io.element.android.wysiwyg.utils.BulletListStyleConfig
import io.element.android.wysiwyg.utils.CodeBlockStyleConfig
import io.element.android.wysiwyg.utils.InlineCodeStyleConfig
import io.element.android.wysiwyg.utils.StyleConfig

private val fakeDrawable = ColorDrawable()

internal fun createFakeStyleConfig() = StyleConfig(
    bulletList = BulletListStyleConfig(
        bulletGapWidth = 1f,
        bulletRadius = 1f,
    ),
    inlineCode = InlineCodeStyleConfig(
        horizontalPadding = 2,
        verticalPadding = 2,
        singleLineBg = fakeDrawable,
        multiLineBgLeft = fakeDrawable,
        multiLineBgMid = fakeDrawable,
        multiLineBgRight = fakeDrawable,
    ),
    codeBlock = CodeBlockStyleConfig(
        leadingMargin = 0,
        verticalPadding = 0,
        backgroundDrawable = fakeDrawable,
    )
)

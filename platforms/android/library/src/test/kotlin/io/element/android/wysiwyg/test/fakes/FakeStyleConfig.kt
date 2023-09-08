package io.element.android.wysiwyg.test.fakes

import io.element.android.wysiwyg.view.BulletListStyleConfig
import io.element.android.wysiwyg.view.CodeBlockStyleConfig
import io.element.android.wysiwyg.view.InlineCodeStyleConfig
import io.element.android.wysiwyg.view.PillStyleConfig
import io.element.android.wysiwyg.view.StyleConfig


internal fun createFakeStyleConfig() = StyleConfig(
    bulletList = BulletListStyleConfig(
        bulletGapWidth = 1f,
        bulletRadius = 1f,
    ),
    inlineCode = InlineCodeStyleConfig(
        horizontalPadding = 2,
        verticalPadding = 2,
        relativeTextSize = 1f,
        singleLineBg = 0,
        multiLineBgLeft = 0,
        multiLineBgMid = 0,
        multiLineBgRight = 0,
    ),
    codeBlock = CodeBlockStyleConfig(
        leadingMargin = 0,
        verticalPadding = 0,
        relativeTextSize = 1f,
        backgroundDrawable = 0,
    ),
    pill = PillStyleConfig(
        android.R.color.white
    )
)

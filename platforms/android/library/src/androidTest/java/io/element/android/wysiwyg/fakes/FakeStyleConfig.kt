package io.element.android.wysiwyg.fakes

import io.element.android.wysiwyg.test.R
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
        singleLineBg = R.color.fake_color,
        multiLineBgLeft = R.color.fake_color,
        multiLineBgMid = R.color.fake_color,
        multiLineBgRight = R.color.fake_color,
    ),
    codeBlock = CodeBlockStyleConfig(
        leadingMargin = 0,
        verticalPadding = 0,
        relativeTextSize = 1f,
        backgroundDrawable = R.color.fake_color,
    ),
    pill = PillStyleConfig(
        R.color.fake_color
    )
)

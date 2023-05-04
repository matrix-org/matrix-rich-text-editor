package io.element.android.wysiwyg.fakes

import android.graphics.drawable.ColorDrawable
import io.element.android.wysiwyg.utils.*
import io.element.android.wysiwyg.test.R

private val fakeDrawable = ColorDrawable()

internal fun createFakeStyleConfig() = StyleConfig(
    bulletList = BulletListStyleConfig(
        bulletGapWidth = 1f,
        bulletRadius = 1f,
    ),
    inlineCode = InlineCodeStyleConfig(
        horizontalPadding = 2,
        verticalPadding = 2,
        relativeTextSize = 1f,
        singleLineBg = fakeDrawable,
        multiLineBgLeft = fakeDrawable,
        multiLineBgMid = fakeDrawable,
        multiLineBgRight = fakeDrawable,
    ),
    codeBlock = CodeBlockStyleConfig(
        leadingMargin = 0,
        verticalPadding = 0,
        relativeTextSize = 1f,
        backgroundDrawable = fakeDrawable,
    ),
    pill = PillStyleConfig(
        R.color.fake_color
    )
)

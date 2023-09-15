package io.element.android.wysiwyg.compose.internal

import android.content.Context
import androidx.compose.ui.graphics.toArgb
import androidx.compose.ui.unit.Density
import io.element.android.wysiwyg.compose.BulletListStyle
import io.element.android.wysiwyg.compose.CodeBlockStyle
import io.element.android.wysiwyg.compose.InlineCodeStyle
import io.element.android.wysiwyg.compose.PillStyle
import io.element.android.wysiwyg.compose.RichTextEditorStyle
import io.element.android.wysiwyg.view.BulletListStyleConfig
import io.element.android.wysiwyg.view.CodeBlockStyleConfig
import io.element.android.wysiwyg.view.InlineCodeStyleConfig
import io.element.android.wysiwyg.view.PillStyleConfig
import io.element.android.wysiwyg.view.StyleConfig
import kotlin.math.roundToInt

internal fun RichTextEditorStyle.toStyleConfig(context: Context): StyleConfig = StyleConfig(
    bulletList = bulletList.toStyleConfig(context),
    inlineCode = inlineCode.toStyleConfig(context),
    codeBlock = codeBlock.toStyleConfig(context),
    pill = pill.toStyleConfig(),
)

internal fun BulletListStyle.toStyleConfig(context: Context): BulletListStyleConfig =
    with(Density(context)) {
        BulletListStyleConfig(
            bulletGapWidth = bulletGapWidth.toPx(),
            bulletRadius = bulletRadius.toPx(),
        )
    }

internal fun InlineCodeStyle.toStyleConfig(context: Context): InlineCodeStyleConfig {
    val density = Density(context)
    return InlineCodeStyleConfig(
        horizontalPadding = with(density) { horizontalPadding.toPx().roundToInt() },
        verticalPadding = with(density) { verticalPadding.toPx().roundToInt() },
        relativeTextSize = relativeTextSize,
        singleLineBg = background.singleLine.drawable,
        multiLineBgLeft = background.multiLineLeft.drawable,
        multiLineBgMid = background.multiLineMiddle.drawable,
        multiLineBgRight = background.multiLineRight.drawable
    )
}

internal fun CodeBlockStyle.toStyleConfig(context: Context): CodeBlockStyleConfig {
    val density = Density(context)
    return CodeBlockStyleConfig(
        leadingMargin = with(density) { leadingMargin.toPx().roundToInt() },
        verticalPadding = with(density) { verticalPadding.toPx().roundToInt() },
        relativeTextSize = relativeTextSize,
        backgroundDrawable = background.drawable,
    )
}

internal fun PillStyle.toStyleConfig(): PillStyleConfig =
    PillStyleConfig(
        backgroundColor = backgroundColor.toArgb(),
    )


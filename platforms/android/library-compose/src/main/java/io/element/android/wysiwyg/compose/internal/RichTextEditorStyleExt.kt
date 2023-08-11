package io.element.android.wysiwyg.compose.internal

import android.content.Context
import android.content.res.Resources.NotFoundException
import androidx.annotation.DrawableRes
import androidx.compose.ui.graphics.toArgb
import androidx.compose.ui.unit.Density
import androidx.core.content.ContextCompat
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
        singleLineBg = context.requireDrawable(singleLineBg),
        multiLineBgLeft = context.requireDrawable(multiLineBgLeft),
        multiLineBgMid = context.requireDrawable(multiLineBgMid),
        multiLineBgRight = context.requireDrawable(multiLineBgRight),
    )
}

internal fun CodeBlockStyle.toStyleConfig(context: Context): CodeBlockStyleConfig {
    val density = Density(context)
    return CodeBlockStyleConfig(
        leadingMargin = with(density) { leadingMargin.toPx().roundToInt() },
        verticalPadding = with(density) { verticalPadding.toPx().roundToInt() },
        relativeTextSize = relativeTextSize,
        backgroundDrawable = context.requireDrawable(backgroundDrawable),
    )
}

internal fun PillStyle.toStyleConfig(): PillStyleConfig =
    PillStyleConfig(
        backgroundColor = backgroundColor.toArgb(),
    )

private fun Context.requireDrawable(
    @DrawableRes drawable: Int
) = ContextCompat.getDrawable(this, drawable)
    ?: throw NotFoundException("Drawable not found")

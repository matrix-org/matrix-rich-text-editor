package io.element.android.wysiwyg.compose.internal

import android.content.Context
import android.graphics.Typeface
import android.os.Build
import android.util.TypedValue
import android.widget.TextView
import androidx.appcompat.widget.AppCompatEditText
import androidx.compose.runtime.Composable
import androidx.compose.runtime.State
import androidx.compose.runtime.remember
import androidx.compose.ui.graphics.toArgb
import androidx.compose.ui.platform.LocalFontFamilyResolver
import androidx.compose.ui.text.font.FontFamily
import androidx.compose.ui.text.font.FontStyle
import androidx.compose.ui.text.font.FontSynthesis
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.Density
import androidx.compose.ui.unit.TextUnitType
import androidx.compose.ui.unit.isSpecified
import androidx.core.content.ContextCompat
import io.element.android.wysiwyg.compose.BulletListStyle
import io.element.android.wysiwyg.compose.CodeBlockStyle
import io.element.android.wysiwyg.compose.InlineCodeStyle
import io.element.android.wysiwyg.compose.PillStyle
import io.element.android.wysiwyg.compose.R
import io.element.android.wysiwyg.compose.RichTextEditorStyle
import io.element.android.wysiwyg.compose.TextStyle
import io.element.android.wysiwyg.view.BulletListStyleConfig
import io.element.android.wysiwyg.view.CodeBlockStyleConfig
import io.element.android.wysiwyg.view.InlineCodeStyleConfig
import io.element.android.wysiwyg.view.PillStyleConfig
import io.element.android.wysiwyg.view.StyleConfig
import io.element.android.wysiwyg.view.TextConfig
import kotlin.math.roundToInt

internal fun RichTextEditorStyle.toStyleConfig(context: Context): StyleConfig = StyleConfig(
    bulletList = bulletList.toStyleConfig(context),
    inlineCode = inlineCode.toStyleConfig(context),
    codeBlock = codeBlock.toStyleConfig(context),
    pill = pill.toStyleConfig(),
    text = text.toStyleConfig(context),
)

internal fun BulletListStyle.toStyleConfig(context: Context): BulletListStyleConfig =
    with(Density(context)) {
        BulletListStyleConfig(
            bulletGapWidth = bulletGapWidth.toPx(),
            bulletRadius = bulletRadius.toPx(),
            leadingMargin = leadingMargin.toPx(),
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

internal fun PillStyle.toStyleConfig(): PillStyleConfig = PillStyleConfig(
    backgroundColor = backgroundColor.toArgb(),
)

@Composable
internal fun TextStyle.rememberTypeface(): State<Typeface> {
    val resolver: FontFamily.Resolver = LocalFontFamilyResolver.current
    @Suppress("UNCHECKED_CAST")
    return remember(resolver, this) {
        resolver.resolve(
            fontFamily = fontFamily,
            fontWeight = fontWeight ?: FontWeight.Normal,
            fontStyle = fontStyle ?: FontStyle.Normal,
            fontSynthesis = fontSynthesis ?: FontSynthesis.All,
        )
    } as State<Typeface>
}

internal fun TextStyle.toStyleConfig(context: Context): TextConfig {
    val density = Density(context)
    return TextConfig(
        typeface = Typeface.defaultFromStyle(fontStyle?.value ?: FontStyle.Normal.value),
        textSize = with(density) { fontSize.toPx() },
    )
}

internal fun TextView.applyStyleInCompose(style: RichTextEditorStyle) {
    includeFontPadding = style.text.includeFontPadding
    setTextColor(style.text.color.toArgb())
    setTextSize(TypedValue.COMPLEX_UNIT_SP, style.text.fontSize.value)
    if (style.text.lineHeight.isSpecified && style.text.lineHeight.value > 0f) {
        val lineHeightInPx = TypedValue.applyDimension(
            style.text.lineHeight.type.toTypeUnit(),
            style.text.lineHeight.value,
            context.resources.displayMetrics
        )
        if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.P) {
            lineHeight = lineHeightInPx.roundToInt()
        } else {
            val fontHeightInPx = TypedValue.applyDimension(
                style.text.fontSize.type.toTypeUnit(),
                style.text.fontSize.value,
                context.resources.displayMetrics
            )
            val extra = lineHeightInPx - fontHeightInPx
            setLineSpacing(extra, 1f)
        }
    }
    if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.Q) {
        val cursorDrawable = ContextCompat.getDrawable(context, R.drawable.cursor)
        cursorDrawable?.setTint(style.cursor.color.toArgb())
        textCursorDrawable = cursorDrawable
        setLinkTextColor(style.link.color.toArgb())
    }
}

private fun TextUnitType.toTypeUnit() = when (this) {
    TextUnitType.Sp -> TypedValue.COMPLEX_UNIT_SP
    TextUnitType.Em -> TypedValue.COMPLEX_UNIT_FRACTION_PARENT
    else -> TypedValue.COMPLEX_UNIT_PX
}

internal fun AppCompatEditText.applyDefaultStyle() {
    // Set the style closer to a BasicTextField composable
    setBackgroundDrawable(null)
    setPadding(0, 0, 0, 0)
}

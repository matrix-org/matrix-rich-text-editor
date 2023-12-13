package io.element.android.wysiwyg.internal.view

import android.content.Context
import android.graphics.Typeface
import android.util.AttributeSet
import android.util.TypedValue
import androidx.annotation.AttrRes
import androidx.core.content.res.getColorOrThrow
import androidx.core.content.res.getDimensionOrThrow
import androidx.core.content.res.getDimensionPixelSizeOrThrow
import androidx.core.content.res.getDrawableOrThrow
import androidx.core.content.res.getFloatOrThrow
import io.element.android.wysiwyg.R
import io.element.android.wysiwyg.view.BulletListStyleConfig
import io.element.android.wysiwyg.view.CodeBlockStyleConfig
import io.element.android.wysiwyg.view.InlineCodeStyleConfig
import io.element.android.wysiwyg.view.PillStyleConfig
import io.element.android.wysiwyg.view.StyleConfig
import io.element.android.wysiwyg.view.TextConfig

internal class EditorEditTextAttributeReader(context: Context, attrs: AttributeSet?) {
    val styleConfig: StyleConfig

    init {
        val typedArray = context.theme.obtainStyledAttributes(
            attrs,
            R.styleable.EditorEditText,
            0,
            R.style.EditorEditText
        )
        styleConfig = StyleConfig(
            bulletList = BulletListStyleConfig(
                bulletGapWidth = typedArray.getDimensionOrThrow(R.styleable.EditorEditText_bulletGap),
                bulletRadius = typedArray.getDimensionOrThrow(R.styleable.EditorEditText_bulletRadius),
                leadingMargin = typedArray.getDimensionOrThrow(R.styleable.EditorEditText_unorderedListLeadingMargin),
            ),
            inlineCode = InlineCodeStyleConfig(
                horizontalPadding = typedArray.getDimensionPixelSizeOrThrow(R.styleable.EditorEditText_inlineCodeHorizontalPadding),
                verticalPadding = typedArray.getDimensionPixelSizeOrThrow(R.styleable.EditorEditText_inlineCodeVerticalPadding),
                relativeTextSize = typedArray.getFloatOrThrow(R.styleable.EditorEditText_inlineCodeRelativeTextSize),
                singleLineBg = typedArray.getDrawableOrThrow(R.styleable.EditorEditText_inlineCodeSingleLineBg),
                multiLineBgLeft = typedArray.getDrawableOrThrow(R.styleable.EditorEditText_inlineCodeMultiLineBgLeft),
                multiLineBgMid = typedArray.getDrawableOrThrow(R.styleable.EditorEditText_inlineCodeMultiLineBgMid),
                multiLineBgRight = typedArray.getDrawableOrThrow(R.styleable.EditorEditText_inlineCodeMultiLineBgRight),

            ),
            codeBlock = CodeBlockStyleConfig(
                leadingMargin = typedArray.getDimensionPixelSizeOrThrow(R.styleable.EditorEditText_codeBlockLeadingMargin),
                verticalPadding = typedArray.getDimensionPixelSizeOrThrow(R.styleable.EditorEditText_codeBlockVerticalPadding),
                relativeTextSize = typedArray.getFloatOrThrow(R.styleable.EditorEditText_codeBlockRelativeTextSize),
                backgroundDrawable = typedArray.getDrawableOrThrow(R.styleable.EditorEditText_codeBlockBackgroundDrawable),
            ),
            pill = PillStyleConfig(
                backgroundColor = typedArray.getColorOrThrow(R.styleable.EditorEditText_pillBackgroundColor),
            ),
            text = TextConfig(
                typeface = Typeface.defaultFromStyle(Typeface.NORMAL),
                textSize = typedArray.getDimensionOrThrow(R.styleable.EditorEditText_android_textSize),
            ),
        )
        typedArray.recycle()
    }

    private fun getResIdFromAttr(context: Context, @AttrRes attr: Int): Int {
        val typedValue = TypedValue()
        context.theme.resolveAttribute(attr, typedValue, true)
        return typedValue.resourceId
    }
}

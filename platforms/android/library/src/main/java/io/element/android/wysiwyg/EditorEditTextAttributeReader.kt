package io.element.android.wysiwyg

import android.content.Context
import android.util.AttributeSet
import androidx.core.content.res.getDimensionOrThrow
import androidx.core.content.res.getDimensionPixelSizeOrThrow
import androidx.core.content.res.getDrawableOrThrow
import io.element.android.wysiwyg.utils.StyleConfig

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
            bulletGapWidth = typedArray.getDimensionOrThrow(R.styleable.EditorEditText_bulletGap),
            bulletRadius = typedArray.getDimensionOrThrow(R.styleable.EditorEditText_bulletRadius),
            inlineCodeHorizontalPadding = typedArray.getDimensionPixelSizeOrThrow(R.styleable.EditorEditText_inlineCodeHorizontalPadding),
            inlineCodeVerticalPadding = typedArray.getDimensionPixelSizeOrThrow(R.styleable.EditorEditText_inlineCodeVerticalPadding),
            inlineCodeSingleLineBg = typedArray.getDrawableOrThrow(R.styleable.EditorEditText_inlineCodeSingleLineBg),
            inlineCodeMultiLineBgLeft = typedArray.getDrawableOrThrow(R.styleable.EditorEditText_inlineCodeMultiLineBgLeft),
            inlineCodeMultiLineBgMid = typedArray.getDrawableOrThrow(R.styleable.EditorEditText_inlineCodeMultiLineBgMid),
            inlineCodeMultiLineBgRight = typedArray.getDrawableOrThrow(R.styleable.EditorEditText_inlineCodeMultiLineBgRight),
        )
        typedArray.recycle()
    }
}
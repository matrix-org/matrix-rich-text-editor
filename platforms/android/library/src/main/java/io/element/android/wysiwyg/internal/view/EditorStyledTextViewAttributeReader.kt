package io.element.android.wysiwyg.internal.view

import android.content.Context
import android.util.AttributeSet
import androidx.core.content.res.getDimensionPixelSizeOrThrow
import androidx.core.content.res.getDrawableOrThrow
import androidx.core.content.res.getFloatOrThrow
import io.element.android.wysiwyg.R
import io.element.android.wysiwyg.view.CodeBlockStyleConfig
import io.element.android.wysiwyg.view.InlineCodeStyleConfig

internal class EditorStyledTextViewAttributeReader(context: Context, attrs: AttributeSet?) {
    internal val inlineCodeStyleConfig: InlineCodeStyleConfig
    internal val codeBlockStyleConfig: CodeBlockStyleConfig

    init {
        val typedArray = context.theme.obtainStyledAttributes(
            attrs,
            R.styleable.EditorStyledTextView,
            0,
            R.style.EditorStyledTextView
        )
        inlineCodeStyleConfig = InlineCodeStyleConfig(
            horizontalPadding = typedArray.getDimensionPixelSizeOrThrow(R.styleable.EditorStyledTextView_inlineCodeHorizontalPadding),
            verticalPadding = typedArray.getDimensionPixelSizeOrThrow(R.styleable.EditorStyledTextView_inlineCodeVerticalPadding),
            relativeTextSize = typedArray.getFloatOrThrow(R.styleable.EditorStyledTextView_inlineCodeRelativeTextSize),
            singleLineBg = typedArray.getDrawableOrThrow(R.styleable.EditorStyledTextView_inlineCodeSingleLineBg),
            multiLineBgLeft = typedArray.getDrawableOrThrow(R.styleable.EditorStyledTextView_inlineCodeMultiLineBgLeft),
            multiLineBgMid = typedArray.getDrawableOrThrow(R.styleable.EditorStyledTextView_inlineCodeMultiLineBgMid),
            multiLineBgRight = typedArray.getDrawableOrThrow(R.styleable.EditorStyledTextView_inlineCodeMultiLineBgRight),
        )
        codeBlockStyleConfig = CodeBlockStyleConfig(
            leadingMargin = typedArray.getDimensionPixelSizeOrThrow(R.styleable.EditorStyledTextView_codeBlockLeadingMargin),
            verticalPadding = typedArray.getDimensionPixelSizeOrThrow(R.styleable.EditorStyledTextView_codeBlockVerticalPadding),
            relativeTextSize = typedArray.getFloatOrThrow(R.styleable.EditorStyledTextView_codeBlockRelativeTextSize),
            backgroundDrawable = typedArray.getDrawableOrThrow(R.styleable.EditorStyledTextView_codeBlockBackgroundDrawable),
        )
        typedArray.recycle()
    }
}

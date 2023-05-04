package io.element.android.wysiwyg

import android.content.Context
import android.util.AttributeSet
import androidx.core.content.res.*
import io.element.android.wysiwyg.utils.*
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
            bulletList = BulletListStyleConfig(
                bulletGapWidth = typedArray.getDimensionOrThrow(R.styleable.EditorEditText_bulletGap),
                bulletRadius = typedArray.getDimensionOrThrow(R.styleable.EditorEditText_bulletRadius),
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
            mention = MentionStyleConfig(
                backgroundColor = typedArray.getResourceIdOrThrow(R.styleable.EditorEditText_mentionBackgroundColor),
            )
        )
        typedArray.recycle()
    }
}

package io.element.android.wysiwyg.view.inlinebg

import android.content.Context
import androidx.core.content.ContextCompat
import io.element.android.wysiwyg.view.CodeBlockStyleConfig
import io.element.android.wysiwyg.view.InlineCodeStyleConfig
import io.element.android.wysiwyg.view.spans.CodeBlockSpan
import io.element.android.wysiwyg.view.spans.InlineCodeSpan

object SpanBackgroundHelperFactory {
    fun createInlineCodeBackgroundHelper(
        styleConfig: InlineCodeStyleConfig,
        context: Context
    ): SpanBackgroundHelper {
        return SpanBackgroundHelper(
            spanType = InlineCodeSpan::class.java,
            horizontalPadding = styleConfig.horizontalPadding,
            verticalPadding = styleConfig.verticalPadding,
            drawable = ContextCompat.getDrawable(context, styleConfig.singleLineBg),
            drawableLeft = ContextCompat.getDrawable(context, styleConfig.multiLineBgLeft),
            drawableMid = ContextCompat.getDrawable(context, styleConfig.multiLineBgMid),
            drawableRight = ContextCompat.getDrawable(context, styleConfig.multiLineBgRight),
        )
    }

    fun createCodeBlockBackgroundHelper(
        styleConfig: CodeBlockStyleConfig,
        context: Context
    ): SpanBackgroundHelper {
        return SpanBackgroundHelper(
            spanType = CodeBlockSpan::class.java,
            horizontalPadding = 0,
            verticalPadding = 0,
            drawable = ContextCompat.getDrawable(context, styleConfig.backgroundDrawable)
        )
    }
}

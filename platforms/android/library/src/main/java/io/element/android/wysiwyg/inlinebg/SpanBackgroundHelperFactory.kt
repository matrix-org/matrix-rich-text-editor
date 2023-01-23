package io.element.android.wysiwyg.inlinebg

import io.element.android.wysiwyg.spans.CodeBlockSpan
import io.element.android.wysiwyg.spans.InlineCodeSpan
import io.element.android.wysiwyg.utils.CodeBlockStyleConfig
import io.element.android.wysiwyg.utils.InlineCodeStyleConfig

object SpanBackgroundHelperFactory {
    fun createInlineCodeBackgroundHelper(styleConfig: InlineCodeStyleConfig): SpanBackgroundHelper {
        return SpanBackgroundHelper(
            spanType = InlineCodeSpan::class.java,
            horizontalPadding = styleConfig.horizontalPadding,
            verticalPadding = styleConfig.verticalPadding,
            drawable = styleConfig.singleLineBg,
            drawableLeft = styleConfig.multiLineBgLeft,
            drawableMid = styleConfig.multiLineBgMid,
            drawableRight = styleConfig.multiLineBgRight,
        )
    }

    fun createCodeBlockBackgroundHelper(styleConfig: CodeBlockStyleConfig): SpanBackgroundHelper {
        return SpanBackgroundHelper(
            spanType = CodeBlockSpan::class.java,
            horizontalPadding = 0,
            verticalPadding = 0,
            drawable = styleConfig.backgroundDrawable,
        )
    }
}

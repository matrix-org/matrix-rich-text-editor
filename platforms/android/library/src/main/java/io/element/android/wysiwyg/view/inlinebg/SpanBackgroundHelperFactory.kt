package io.element.android.wysiwyg.view.inlinebg

import io.element.android.wysiwyg.view.spans.CodeBlockSpan
import io.element.android.wysiwyg.view.spans.InlineCodeSpan
import io.element.android.wysiwyg.view.CodeBlockStyleConfig
import io.element.android.wysiwyg.view.InlineCodeStyleConfig

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

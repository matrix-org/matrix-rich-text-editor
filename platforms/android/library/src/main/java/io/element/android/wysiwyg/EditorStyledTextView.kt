package io.element.android.wysiwyg

import android.content.Context
import android.graphics.Canvas
import android.text.Spanned
import android.util.AttributeSet
import androidx.appcompat.widget.AppCompatTextView
import androidx.core.graphics.withTranslation
import io.element.android.wysiwyg.inlinebg.SpanBackgroundHelper
import io.element.android.wysiwyg.inlinebg.SpanBackgroundHelperFactory
import io.element.android.wysiwyg.spans.InlineCodeSpan
import io.element.android.wysiwyg.utils.*

/**
 * This TextView can display all spans used by the editor.
 */
class EditorStyledTextView : AppCompatTextView {
    private lateinit var inlineCodeStyleConfig: InlineCodeStyleConfig
    private val inlineCodeBgHelper: SpanBackgroundHelper<InlineCodeSpan> by lazy {
        SpanBackgroundHelperFactory.createInlineCodeBackgroundHelper(inlineCodeStyleConfig)
    }

    constructor(context: Context) : super(context)

    constructor(context: Context, attrs: AttributeSet?) : super(context, attrs) {
        inlineCodeStyleConfig =
            EditorStyledTextViewAttributeReader(context, attrs).inlineCodeStyleConfig
    }

    constructor(context: Context, attrs: AttributeSet?, defStyleAttr: Int) :
            super(context, attrs, defStyleAttr)

    override fun onDraw(canvas: Canvas) {
        // need to draw bg first so that text can be on top during super.onDraw()
        if (text is Spanned && layout != null) {
            canvas.withTranslation(totalPaddingLeft.toFloat(), totalPaddingTop.toFloat()) {
                inlineCodeBgHelper.draw(canvas, text as Spanned, layout)
            }
        }
        super.onDraw(canvas)
    }
}

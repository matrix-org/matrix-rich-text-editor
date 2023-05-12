package io.element.android.wysiwyg

import android.content.Context
import android.graphics.Canvas
import android.text.Spanned
import android.util.AttributeSet
import androidx.appcompat.widget.AppCompatTextView
import androidx.core.graphics.withTranslation
import io.element.android.wysiwyg.internal.view.EditorStyledTextViewAttributeReader
import io.element.android.wysiwyg.view.CodeBlockStyleConfig
import io.element.android.wysiwyg.view.InlineCodeStyleConfig
import io.element.android.wysiwyg.view.inlinebg.SpanBackgroundHelper
import io.element.android.wysiwyg.view.inlinebg.SpanBackgroundHelperFactory

/**
 * This TextView can display all spans used by the editor.
 */
open class EditorStyledTextView : AppCompatTextView {
    private lateinit var inlineCodeStyleConfig: InlineCodeStyleConfig
    private lateinit var codeBlockStyleConfig: CodeBlockStyleConfig
    private var styleAttributesReady = false
    private val inlineCodeBgHelper: SpanBackgroundHelper by lazy {
        SpanBackgroundHelperFactory.createInlineCodeBackgroundHelper(inlineCodeStyleConfig)
    }
    private val codeBlockBgHelper: SpanBackgroundHelper by lazy {
        SpanBackgroundHelperFactory.createCodeBlockBackgroundHelper(codeBlockStyleConfig)
    }

    constructor(context: Context) : super(context)

    constructor(context: Context, attrs: AttributeSet?) : super(context, attrs) {
        val attrReader = EditorStyledTextViewAttributeReader(context, attrs)
        inlineCodeStyleConfig = attrReader.inlineCodeStyleConfig
        codeBlockStyleConfig = attrReader.codeBlockStyleConfig
        styleAttributesReady = true
    }

    constructor(context: Context, attrs: AttributeSet?, defStyleAttr: Int) :
            super(context, attrs, defStyleAttr)

    override fun setText(text: CharSequence?, type: BufferType?) {
        super.setText(text, type)

        // setText may be called during initialisation when we're not yet
        // ready to load the background helpers
        if(!styleAttributesReady) return

        inlineCodeBgHelper.clearCachedPositions()
        codeBlockBgHelper.clearCachedPositions()
    }

    override fun onDraw(canvas: Canvas) {
        // need to draw bg first so that text can be on top during super.onDraw()
        if (text is Spanned && layout != null) {
            canvas.withTranslation(totalPaddingLeft.toFloat(), totalPaddingTop.toFloat()) {
                codeBlockBgHelper.draw(canvas, text as Spanned, layout)
                inlineCodeBgHelper.draw(canvas, text as Spanned, layout)
            }
        }
        super.onDraw(canvas)
    }
}

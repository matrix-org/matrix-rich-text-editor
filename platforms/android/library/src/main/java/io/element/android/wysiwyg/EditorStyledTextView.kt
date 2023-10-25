package io.element.android.wysiwyg

import android.content.Context
import android.graphics.Canvas
import android.text.Spanned
import android.util.AttributeSet
import androidx.appcompat.widget.AppCompatTextView
import androidx.core.graphics.withTranslation
import io.element.android.wysiwyg.display.MentionDisplayHandler
import io.element.android.wysiwyg.internal.view.EditorEditTextAttributeReader
import io.element.android.wysiwyg.utils.HtmlConverter
import io.element.android.wysiwyg.view.StyleConfig
import io.element.android.wysiwyg.view.inlinebg.SpanBackgroundHelper
import io.element.android.wysiwyg.view.inlinebg.SpanBackgroundHelperFactory
import io.element.android.wysiwyg.view.spans.ReuseSourceSpannableFactory
import uniffi.wysiwyg_composer.MentionDetector
import uniffi.wysiwyg_composer.newMentionDetector

/**
 * This TextView can display all spans used by the editor.
 */
open class EditorStyledTextView : AppCompatTextView {

    private var mentionDetector: MentionDetector? = null

    private lateinit var inlineCodeBgHelper: SpanBackgroundHelper
    private lateinit var codeBlockBgHelper: SpanBackgroundHelper

    /**
     * The [StyleConfig] used to style the spans generated from the HTML in this TextView.
     */
    lateinit var styleConfig: StyleConfig
        private set

    private var isInit = false

    private val spannableFactory = ReuseSourceSpannableFactory()

    private var mentionDisplayHandler: MentionDisplayHandler? = null
    private var htmlConverter: HtmlConverter? = null

    init {
        setSpannableFactory(spannableFactory)
        isInit = true
    }

    constructor(context: Context) : super(context, null)

    constructor(context: Context, attrs: AttributeSet?) : super(context, attrs) {
        styleConfig = EditorEditTextAttributeReader(context, attrs).styleConfig
    }

    constructor(context: Context, attrs: AttributeSet?, defStyleAttr: Int) : super(
        context, attrs, defStyleAttr
    ) {
        styleConfig = EditorEditTextAttributeReader(context, attrs).styleConfig
    }

    override fun setText(text: CharSequence?, type: BufferType?) {
        super.setText(text, type)
        // setText may be called during initialisation when we're not yet
        // ready to load the background helpers
        if (!isInit) return
        inlineCodeBgHelper.clearCachedPositions()
        codeBlockBgHelper.clearCachedPositions()
    }

    /**
     * Sets up the [HtmlConverter] used to translate HTML to Spanned text.
     * @param styleConfig The styles to use for the generated spans.
     * @param mentionDisplayHandler Used to decide how to display any mentions found in the HTML text.
     */
    fun setupHtmlConverter(styleConfig: StyleConfig, mentionDisplayHandler: MentionDisplayHandler?) {
        this.styleConfig = styleConfig
        this.mentionDisplayHandler = mentionDisplayHandler

        inlineCodeBgHelper = SpanBackgroundHelperFactory.createInlineCodeBackgroundHelper(styleConfig.inlineCode)
        codeBlockBgHelper = SpanBackgroundHelperFactory.createCodeBlockBackgroundHelper(styleConfig.codeBlock)

        htmlConverter = createHtmlConverter(styleConfig, mentionDisplayHandler)
    }

    /**
     * Set the text of the TextView with HTML formatting.
     * @param htmlText The text to display, with HTML formatting.
     * Consider using [HtmlConverter.fromHtmlToSpans] and [setText] instead.
     */
    fun setHtml(htmlText: String) {
        if (!isInit) return
        htmlConverter?.fromHtmlToSpans(htmlText)?.let { setText(it, BufferType.SPANNABLE) }
    }

    override fun onDraw(canvas: Canvas) {
        // need to draw bg first so that text can be on top during super.onDraw()
        if (text is Spanned && layout != null && isInit) {
            canvas.withTranslation(totalPaddingLeft.toFloat(), totalPaddingTop.toFloat()) {
                codeBlockBgHelper.draw(canvas, text as Spanned, layout)
                inlineCodeBgHelper.draw(canvas, text as Spanned, layout)
            }
        }
        super.onDraw(canvas)
    }

    override fun onAttachedToWindow() {
        super.onAttachedToWindow()

        mentionDetector = if (isInEditMode) null else newMentionDetector()

        setupHtmlConverter(styleConfig, mentionDisplayHandler)
    }

    override fun onDetachedFromWindow() {
        super.onDetachedFromWindow()

        mentionDetector?.destroy()
        mentionDetector = null
    }

    private fun createHtmlConverter(styleConfig: StyleConfig, mentionDisplayHandler: MentionDisplayHandler?): HtmlConverter {
        return HtmlConverter.Factory.create(
            context = context,
            styleConfig = styleConfig,
            mentionDisplayHandler = mentionDisplayHandler,
            isMention = mentionDetector?.let { detector ->
                { _, url ->
                    detector.isMention(url)
                }
            }
        )
    }
}

package io.element.android.wysiwyg.compose

import android.content.Context
import io.element.android.wysiwyg.compose.internal.toStyleConfig
import io.element.android.wysiwyg.display.MentionDisplayHandler
import io.element.android.wysiwyg.utils.HtmlConverter
import timber.log.Timber

/**
 * An [HtmlConverter] that can be configured with a [RichTextEditorStyle].
 */
class StyledHtmlConverter(
    private val context: Context,
    private val mentionDisplayHandler: MentionDisplayHandler?,
    private val isMention: ((text: String, url: String) -> Boolean)?,
) : HtmlConverter {

    private var htmlConverter: HtmlConverter? = null

    fun configureWith(style: RichTextEditorStyle) {
        Timber.d("Configure with style: $style")
        htmlConverter = HtmlConverter.Factory.create(
            context = context,
            styleConfig = style.toStyleConfig(context),
            mentionDisplayHandler = mentionDisplayHandler,
            isMention = isMention,
        )
    }

    override fun fromHtmlToSpans(html: String): CharSequence {
        return htmlConverter?.fromHtmlToSpans(html) ?: errorNotConfigured()
    }

    private fun errorNotConfigured(): Nothing {
        error("ComposableHtmlConverter must be configured with a RichTextEditorStyle before use")
    }


}

package io.element.android.wysiwyg.utils

import android.app.Application
import android.content.Context
import androidx.core.text.HtmlCompat
import io.element.android.wysiwyg.display.MentionDisplayHandler
import io.element.android.wysiwyg.view.StyleConfig

interface HtmlConverter {
    fun fromHtmlToPlainText(html: String): String
    fun fromHtmlToSpans(html: String): CharSequence

    object Factory {
        fun create(
            context: Context,
            styleConfigProvider: () -> StyleConfig,
            mentionDisplayHandlerProvider: ()->MentionDisplayHandler?,
        ): HtmlConverter {
            val resourcesProvider =
                AndroidResourcesHelper(context.applicationContext as Application)
            return AndroidHtmlConverter(provideHtmlToSpansParser = { html ->
                HtmlToSpansParser(
                    resourcesHelper = resourcesProvider,
                    html = html,
                    styleConfig = styleConfigProvider,
                    mentionDisplayHandler = mentionDisplayHandlerProvider,
                )
            })
        }
    }


}

internal class AndroidHtmlConverter(
    private val provideHtmlToSpansParser: (html: String) -> HtmlToSpansParser
) : HtmlConverter {
    /**
     * Get the content with formatting removed.
     * TODO: Return markdown formatted plaintext instead
     */
    override fun fromHtmlToPlainText(html: String): String = HtmlCompat.fromHtml(
        html, HtmlCompat.FROM_HTML_MODE_LEGACY
    ).toString()

    override fun fromHtmlToSpans(html: String): CharSequence =
        provideHtmlToSpansParser(html).convert()
}
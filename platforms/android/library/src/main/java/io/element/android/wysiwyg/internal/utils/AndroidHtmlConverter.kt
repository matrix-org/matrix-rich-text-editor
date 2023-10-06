package io.element.android.wysiwyg.internal.utils

import androidx.core.text.HtmlCompat
import io.element.android.wysiwyg.utils.HtmlConverter
import io.element.android.wysiwyg.utils.HtmlToSpansParser

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
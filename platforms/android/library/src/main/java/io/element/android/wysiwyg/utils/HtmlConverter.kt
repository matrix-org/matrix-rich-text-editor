package io.element.android.wysiwyg.utils

import androidx.core.text.HtmlCompat

internal interface HtmlConverter {
    fun fromHtmlToPlainText(html: String): String
    fun fromHtmlToSpans(html: String): CharSequence
}

internal class AndroidHtmlConverter(
    private val htmlToSpansParser: HtmlToSpansParser
): HtmlConverter {
    /**
     * Get the content with formatting removed.
     * TODO: Return markdown formatted plaintext instead
     */
    override fun fromHtmlToPlainText(html: String): String =
        HtmlCompat.fromHtml(
            html, HtmlCompat.FROM_HTML_MODE_LEGACY
        ).toString()

    override fun fromHtmlToSpans(html: String): CharSequence =
        htmlToSpansParser.convert(html)
}
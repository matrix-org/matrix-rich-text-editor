package io.element.android.wysiwyg.internal.utils

import io.element.android.wysiwyg.utils.HtmlConverter
import io.element.android.wysiwyg.utils.HtmlToSpansParser

internal class AndroidHtmlConverter(
    private val provideHtmlToSpansParser: (html: String) -> HtmlToSpansParser
) : HtmlConverter {

    override fun fromHtmlToSpans(html: String): CharSequence =
        provideHtmlToSpansParser(html).convert()

}
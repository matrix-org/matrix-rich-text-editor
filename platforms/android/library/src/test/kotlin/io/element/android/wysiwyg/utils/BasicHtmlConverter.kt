package io.element.android.wysiwyg.utils

/**
 * HTML converter that is not depend on Android, for unit tests.
 */
class BasicHtmlConverter: HtmlConverter {
    override fun fromHtmlToPlainText(html: String): String =
        html.replace("<[^>]*>".toRegex(), "")
}
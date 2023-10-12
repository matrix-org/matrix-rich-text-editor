package io.element.android.wysiwyg.utils

/**
 * HTML converter that is not depend on Android, for unit tests.
 */
class BasicHtmlConverter: HtmlConverter {

    override fun fromHtmlToSpans(html: String): CharSequence = html.replace("<[^>]*>".toRegex(), "")
}
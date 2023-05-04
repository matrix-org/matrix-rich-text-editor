package io.element.android.wysiwyg.suggestions

/**
 * Convenience implementation of a [LinkDisplayHandler] that detects Matrix mentions and
 * displays them as default pills.
 */
class MatrixMentionLinkDisplayHandler : LinkDisplayHandler {
    override fun resolveUrlDisplay(url: String): LinkDisplay =
        when (url.startsWith("https://matrix.to/#/")) {
            true -> LinkDisplay.Pill
            false -> LinkDisplay.Plain
        }
}
package io.element.android.wysiwyg.poc.matrix

import io.element.android.wysiwyg.links.LinkDisplay
import io.element.android.wysiwyg.links.LinkDisplayHandler

/**
 * Convenience implementation of a [LinkDisplayHandler] that detects Matrix mentions and
 * displays them as default pills.
 */
class MatrixMentionLinkDisplayHandler : LinkDisplayHandler {
    override fun resolveUrlDisplay(text: String, url: String): LinkDisplay =
        when (url.startsWith("https://matrix.to/#/")) {
            true -> LinkDisplay.Pill
            false -> LinkDisplay.Plain
        }
}
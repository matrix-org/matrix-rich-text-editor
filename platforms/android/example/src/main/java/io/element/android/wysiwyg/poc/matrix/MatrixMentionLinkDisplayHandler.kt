package io.element.android.wysiwyg.poc.matrix

import io.element.android.wysiwyg.display.TextDisplay
import io.element.android.wysiwyg.display.LinkDisplayHandler

/**
 * Convenience implementation of a [LinkDisplayHandler] that detects Matrix mentions and
 * displays them as default pills.
 */
class MatrixMentionLinkDisplayHandler : LinkDisplayHandler {
    override fun resolveLinkDisplay(text: String, url: String): TextDisplay =
        when (url.startsWith("https://matrix.to/#/")) {
            true -> TextDisplay.Pill
            false -> TextDisplay.Plain
        }
}
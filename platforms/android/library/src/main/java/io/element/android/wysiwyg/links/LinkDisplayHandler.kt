package io.element.android.wysiwyg.links

import android.text.style.ReplacementSpan

/**
 * Clients can implement a link display handler to let the editor
 * know how to display URLs.
 */
fun interface LinkDisplayHandler {
    /**
     * Return the method with which to display a given link
     */
    fun resolveUrlDisplay(text: String, url: String): LinkDisplay
}

/**
 * Different ways to display links
 */
sealed class LinkDisplay {
    /**
     * Display the link using a custom span
     */
    data class Custom(val customSpan: ReplacementSpan): LinkDisplay()

    /**
     * Display the link using a default pill shape
     */
    object Pill: LinkDisplay()

    /**
     * Display the link as a plain text link
     */
    object Plain: LinkDisplay()
}


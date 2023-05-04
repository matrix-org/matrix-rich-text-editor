package io.element.android.wysiwyg.suggestions

/**
 * Clients can implement a link display handler to let the editor
 * know how to display URLs.
 */
fun interface LinkDisplayHandler {
    /**
     * Return the method with which to display a given URL
     */
    fun resolveUrlDisplay(url: String): LinkDisplay
}

/**
 * Different ways to display links
 */
sealed class LinkDisplay {
    /**
     * Display the link using a custom span
     */
    data class Custom(val customSpan: Any): LinkDisplay()

    /**
     * Display the link using a default pill shape
     */
    object Pill: LinkDisplay()

    /**
     * Display the link as a plain text link
     */
    object Plain: LinkDisplay()
}


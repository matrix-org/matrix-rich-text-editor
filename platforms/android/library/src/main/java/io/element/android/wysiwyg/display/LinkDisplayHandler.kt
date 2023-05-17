package io.element.android.wysiwyg.display

/**
 * Clients can implement a link display handler to let the editor
 * know how to display links.
 */
fun interface LinkDisplayHandler {
    /**
     * Return the method with which to display a given link
     */
    fun resolveLinkDisplay(text: String, url: String): TextDisplay
}

package io.element.android.wysiwyg.display

/**
 * Clients can implement a link display handler to let the editor
 * know how to display links.
 */
interface KeywordDisplayHandler {
    val keywords: List<String>

    /**
     * Return the method with which to display a given keyword
     */
    fun resolveKeywordDisplay(text: String): TextDisplay
}


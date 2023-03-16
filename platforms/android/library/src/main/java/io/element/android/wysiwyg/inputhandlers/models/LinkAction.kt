package io.element.android.wysiwyg.inputhandlers.models

/**
 * Link related editor actions, depending on the current selection.
 */
sealed class LinkAction {
    /**
     * Insert new text with a link (only available when no text is selected)
     */
    object InsertLink : LinkAction()

    /**
     * Add a link for the current selection, without supplying text.
     */
    object SetLink : LinkAction()

    /**
     * Edit the text and the url of link for the current selection.
     */
    data class EditLink(val text: String, val url: String) : LinkAction()
}

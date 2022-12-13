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
     * Edit the link for the current selection.
     */
    data class EditLink(val currentLink: String?, val currentText: String) : LinkAction()

    /**
     * Create a new link for the current selection.
     */
    data class CreateLink(val readonlyText: String) : LinkAction()
}

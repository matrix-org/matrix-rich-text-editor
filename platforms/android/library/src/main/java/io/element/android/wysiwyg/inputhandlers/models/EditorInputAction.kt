package io.element.android.wysiwyg.inputhandlers.models

import uniffi.wysiwyg_composer.ComposerModel

/**
 * Text editing actions to be performed by the Rust code through the [ComposerModel] component.
 */
sealed interface EditorInputAction {
    /**
     * Replaces the text at the current selection with the provided [value] in plain text.
     */
    data class ReplaceText(val value: CharSequence): EditorInputAction

    /**
     * Replaces the whole contents of the editor with the passed [html], re-creating the Dom.
     */
    data class ReplaceAllHtml(val html: String): EditorInputAction

    /**
     * Deletes text in the [start]..[end] selection
     */
    data class Delete(val start: Int, val end: Int): EditorInputAction

    /**
     * Adds a new line break at the current selection.
     */
    object InsertParagraph: EditorInputAction

    /**
     * Removes text in a backwards direction given the current selection.
     */
    object BackPress: EditorInputAction

    /**
     * Applies the passed inline [format] to the current selection, either creating or extending it
     * or removing it if it was present in that selection.
     */
    data class ApplyInlineFormat(val format: InlineFormat): EditorInputAction

    /**
     * Un-does the previous action, restoring the previous editor state.
     */
    object Undo: EditorInputAction

    /**
     * Re-does the last undone action, restoring its state.
     */
    object Redo: EditorInputAction

    /**
     * Creates a link to the [link] url in the current selection.
     */
    data class SetLink(val link: String): EditorInputAction

    /**
     * Creates a list, [ordered] if true or unordered in the current selection.
     */
    data class ToggleList(val ordered: Boolean): EditorInputAction
}

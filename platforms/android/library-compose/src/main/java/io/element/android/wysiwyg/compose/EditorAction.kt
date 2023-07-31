package io.element.android.wysiwyg.compose

/**
 * Represents an action that can be performed on the editor.
 */
sealed interface EditorAction {

    /**
     * Toggles bold formatting on the current selection.
     */
    data object Bold : EditorAction

    /**
     * Toggles italic formatting on the current selection.
     */
    data object Italic : EditorAction

    /**
     * Replaces the whole editor contents with new HTML content.
     */
    data class SetHtml(val html: String) : EditorAction
}

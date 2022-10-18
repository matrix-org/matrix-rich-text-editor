package io.element.android.wysiwyg.inputhandlers.models

import uniffi.wysiwyg_composer.ComposerAction

/**
 * Mapping of [ComposerAction] inline format actions. These are text styles that can be applied to
 * a text selection in the editor.
 */
sealed interface InlineFormat {
    object Bold: InlineFormat
    object Italic: InlineFormat
    object Underline: InlineFormat
    object StrikeThrough: InlineFormat
    object InlineCode: InlineFormat
}

package io.element.android.wysiwyg.view.models

import androidx.compose.runtime.Immutable
import uniffi.wysiwyg_composer.ComposerAction

/**
 * Mapping of [ComposerAction] inline format actions. These are text styles that can be applied to
 * a text selection in the editor.
 */
@Immutable
sealed interface InlineFormat {
    data object Bold: InlineFormat
    data object Italic: InlineFormat
    data object Underline: InlineFormat
    data object StrikeThrough: InlineFormat
    data object InlineCode: InlineFormat
}

package io.element.android.wysiwyg.inputhandlers.models

import android.text.Spanned
import android.widget.EditText
import uniffi.wysiwyg_composer.TextUpdate.ReplaceAll

/**
 * Mapped model of [ReplaceAll] from the Rust code to be applied to the [EditText].
 */
internal data class ReplaceTextResult(
    /** Text in [Spanned] format after being parsed from HTML. */
    val text: CharSequence,
    /** Selection to apply to the editor. */
    val selection: IntRange,
)

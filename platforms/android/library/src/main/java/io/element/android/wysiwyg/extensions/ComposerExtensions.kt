package io.element.android.wysiwyg.extensions

import io.element.android.wysiwyg.BuildConfig
import timber.log.Timber
import uniffi.wysiwyg_composer.ComposerModelInterface

val LOG_ENABLED = BuildConfig.DEBUG

/**
 * Get the current HTML representation of the formatted text in the Rust code, along with its
 * selection.
 */
fun ComposerModelInterface.toHtml() = with(getCurrentDomState()) {
    "'${html.string()}' | Start: $start | End: $end"
}

/**
 * Log the current state of the editor in the Rust code.
 */
fun ComposerModelInterface.log() = if (LOG_ENABLED) {
    Timber.d("Html ${toHtml().revealZeroWidthCharacters()}")
    Timber.v("Tree\n ${toTree().revealZeroWidthCharacters()}")
} else Unit

// To visualize zero-width spaces easily
private fun String.revealZeroWidthCharacters() =
    replace("\u200b", "~")

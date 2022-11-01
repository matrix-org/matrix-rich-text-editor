package io.element.android.wysiwyg.extensions

import io.element.android.wysiwyg.BuildConfig
import timber.log.Timber
import uniffi.wysiwyg_composer.ComposerModelInterface
import uniffi.wysiwyg_composer.ComposerState

val LOG_ENABLED = BuildConfig.DEBUG

/**
 * Get the current HTML representation of the formatted text in the Rust code, along with its
 * selection.
 */
fun ComposerState.dump() = "'${html.string()}' | Start: $start | End: $end"

/**
 * Log the current state of the editor in the Rust code.
 */
fun ComposerModelInterface.log() = if (LOG_ENABLED) {
    Timber.d(
        getCurrentDomState().dump()
            // To visualize zero-width spaces easily
            .replace("\u200b", "~")
    )
} else Unit

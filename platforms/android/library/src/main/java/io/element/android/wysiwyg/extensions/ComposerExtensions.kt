package io.element.android.wysiwyg.extensions

import android.util.Log
import io.element.android.wysiwyg.BuildConfig
import uniffi.wysiwyg_composer.ComposerModel
import uniffi.wysiwyg_composer.ComposerModelInterface
import uniffi.wysiwyg_composer.ComposerState

val LOG_ENABLED = BuildConfig.DEBUG

fun ComposerState.dump() = "'${html.string()}' | Start: $start | End: $end"
fun ComposerModelInterface.log() = if (LOG_ENABLED)
    Log.d("COMPOSER_PROCESSOR", dumpState().dump()
            // To visualize zero-width spaces easily
        .replace("\u200b", "~"))
else 0

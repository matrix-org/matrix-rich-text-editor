package io.element.android.wysiwygpoc.extensions

import android.util.Log
import io.element.android.wysiwygpoc.BuildConfig
import uniffi.wysiwyg_composer.ComposerModel
import uniffi.wysiwyg_composer.ComposerState

val LOG_ENABLED = BuildConfig.DEBUG

fun ComposerState.dump() = "'${html.string()}' | Start: $start | End: $end"
fun ComposerModel.log() = if (LOG_ENABLED)
    Log.d("COMPOSER_PROCESSOR", dumpState().dump())
else 0

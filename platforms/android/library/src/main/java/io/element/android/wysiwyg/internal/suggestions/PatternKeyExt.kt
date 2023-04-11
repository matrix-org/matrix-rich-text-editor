package io.element.android.wysiwyg.internal.suggestions

import uniffi.wysiwyg_composer.PatternKey

internal fun PatternKey.toSymbol() = when(this) {
    PatternKey.AT -> "@"
    PatternKey.HASH -> "#"
    PatternKey.SLASH -> "/"
}
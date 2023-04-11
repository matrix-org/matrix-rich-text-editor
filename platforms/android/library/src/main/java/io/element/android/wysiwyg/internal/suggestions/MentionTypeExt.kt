package io.element.android.wysiwyg.internal.suggestions

import io.element.android.wysiwyg.suggestions.MentionType
import uniffi.wysiwyg_composer.PatternKey

internal fun MentionType.toInternalPatternKey(): PatternKey = when(this) {
    MentionType.User -> PatternKey.AT
    MentionType.Room -> PatternKey.HASH
}

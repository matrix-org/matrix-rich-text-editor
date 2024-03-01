package io.element.android.wysiwyg.utils

import android.text.TextWatcher

/**
 * A TextWatcher that can be suspended.
 * When suspended, the TextWatcher will not dispatch any events.
 */
internal interface SuspendableTextWatcher : TextWatcher {
    fun pause(block: () -> Unit)
}
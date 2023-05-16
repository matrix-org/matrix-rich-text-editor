package io.element.android.wysiwyg.display

import android.text.style.ReplacementSpan

/**
 * Different ways to display text
 */
sealed class TextDisplay {
    /**
     * Display the text using a custom span
     */
    data class Custom(val customSpan: ReplacementSpan): TextDisplay()

    /**
     * Display the text using a default pill shape
     */
    object Pill: TextDisplay()

    /**
     * Display the text without any replacement
     */
    object Plain: TextDisplay()
}


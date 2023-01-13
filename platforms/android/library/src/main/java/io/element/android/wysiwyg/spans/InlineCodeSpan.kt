package io.element.android.wysiwyg.spans

import android.text.style.TypefaceSpan

/**
 * Inline code (`some code` in Markdown, <code> in HTML) Span that applies a monospaced font style.
 *
 * Note that this span does not apply a background style; it simply tells the TextView where to
 * apply an inline background.
 *
 * See [io.element.android.wysiwyg.inlinebg.SpanBackgroundRenderer], based on the official Google sample:
 * - https://medium.com/androiddevelopers/drawing-a-rounded-corner-background-on-text-5a610a95af5
 * - https://github.com/googlearchive/android-text/tree/996fdb65bbfbb786c3ca4e4e40b30509067201fc/RoundedBackground-Kotlin
 */
internal class InlineCodeSpan: TypefaceSpan("monospace")

package io.element.android.wysiwyg.view.spans

import android.text.Spannable
import android.text.SpannableStringBuilder

/**
 * This factory is used to reuse the current source if possible to improve performance.
 */
internal class ReuseSourceSpannableFactory : Spannable.Factory() {
    override fun newSpannable(source: CharSequence?): Spannable {
        // Try to reuse current source if possible to improve performance
        return source as? Spannable ?: SpannableStringBuilder(source)
    }
}
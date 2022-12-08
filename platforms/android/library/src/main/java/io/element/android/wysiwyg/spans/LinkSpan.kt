package io.element.android.wysiwyg.spans

import android.text.TextPaint
import android.text.style.URLSpan

internal class LinkSpan(
    url: String
) : URLSpan(url) {
    override fun updateDrawState(ds: TextPaint) {
        // Check if the text is already underlined (for example by an UnderlineSpan)
        val wasUnderlinedByAnotherSpan = ds.isUnderlineText

        super.updateDrawState(ds)

        if (!wasUnderlinedByAnotherSpan) {
            ds.isUnderlineText = false
        }
    }
}
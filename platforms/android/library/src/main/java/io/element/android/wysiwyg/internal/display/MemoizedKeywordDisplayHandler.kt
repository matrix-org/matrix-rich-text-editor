package io.element.android.wysiwyg.internal.display

import io.element.android.wysiwyg.display.KeywordDisplayHandler
import io.element.android.wysiwyg.display.TextDisplay

internal class MemoizedKeywordDisplayHandler(
    private val delegate: KeywordDisplayHandler,
): KeywordDisplayHandler {
    private val cache = mutableMapOf<String, TextDisplay>()
    override val keywords: List<String> get() =
        delegate.keywords

    override fun resolveKeywordDisplay(text: String): TextDisplay {
        val cached = cache[text]

        if(cached != null) {
            return cached
        }

        val calculated = delegate.resolveKeywordDisplay(text)

        cache[text] = calculated

        return calculated
    }
}
package io.element.android.wysiwyg.internal.display

import io.element.android.wysiwyg.display.TextDisplay
import io.element.android.wysiwyg.display.LinkDisplayHandler

/**
 * This [LinkDisplayHandler] ensures that the editor does not request how to display the same item
 * from the host app on every editor update by caching the results in memory.
 */
internal class MemoizingLinkDisplayHandler(
    private val delegate: LinkDisplayHandler
): LinkDisplayHandler {
    private val cache = mutableMapOf<Pair<String, String>, TextDisplay>()
    override fun resolveLinkDisplay(text: String, url: String): TextDisplay {
        val key = text to url
        val cached = cache[key]

        if(cached != null) {
            return cached
        }

        val calculated = delegate.resolveLinkDisplay(text, url)

        cache[key] = calculated

        return calculated
    }
}
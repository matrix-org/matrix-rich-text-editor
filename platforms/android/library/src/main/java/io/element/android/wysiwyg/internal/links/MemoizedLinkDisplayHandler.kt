package io.element.android.wysiwyg.internal.links

import io.element.android.wysiwyg.links.LinkDisplay
import io.element.android.wysiwyg.links.LinkDisplayHandler


/**
 * This [LinkDisplayHandler] ensures that the editor does not request how to display the same link
 * from the host app on every editor update by caching the results in memory.
 */
internal class MemoizingLinkDisplayHandler(
    private val delegate: LinkDisplayHandler
): LinkDisplayHandler {
    private val cache = mutableMapOf<Pair<String, String>, LinkDisplay>()
    override fun resolveUrlDisplay(text: String, url: String): LinkDisplay {
        val key = text to url
        val cached = cache[key]

        if(cached != null) {
            return cached
        }

        val calculated = delegate.resolveUrlDisplay(text, url)

        cache[key] = calculated

        return calculated
    }
}
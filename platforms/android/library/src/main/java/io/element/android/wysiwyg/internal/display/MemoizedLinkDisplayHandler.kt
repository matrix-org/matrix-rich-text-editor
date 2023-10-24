package io.element.android.wysiwyg.internal.display

import io.element.android.wysiwyg.display.TextDisplay
import io.element.android.wysiwyg.display.MentionDisplayHandler

/**
 * This [MentionDisplayHandler] ensures that the editor does not request how to display the same item
 * from the host app on every editor update by caching the results in memory.
 */
internal class MemoizingMentionDisplayHandler(
    private val delegate: MentionDisplayHandler
): MentionDisplayHandler {
    private val cache = mutableMapOf<Pair<String, String>, TextDisplay>()
    private var atRoomCache: TextDisplay? = null
    override fun resolveMentionDisplay(text: String, url: String): TextDisplay {
        val key = text to url
        val cached = cache[key]

        if(cached != null) {
            return cached
        }

        val calculated = delegate.resolveMentionDisplay(text, url)

        cache[key] = calculated

        return calculated
    }

    override fun resolveAtRoomMentionDisplay(): TextDisplay {
        atRoomCache?.let {
            return it
        }

        val calculated = delegate.resolveAtRoomMentionDisplay()

        atRoomCache = calculated

        return calculated
    }

    override fun isMention(url: String): Boolean = delegate.isMention(url)
}

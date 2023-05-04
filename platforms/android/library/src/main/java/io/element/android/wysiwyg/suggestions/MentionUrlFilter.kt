package io.element.android.wysiwyg.suggestions

/**
 * Clients can implement a mention URL filter to let the editor
 * know which URLs represent mentions and should be displayed
 * accordingly.
 */
fun interface MentionUrlFilter {
    /**
     * Return true if the URL represents a mention
     */
    fun isMention(url: String): Boolean
}


package io.element.android.wysiwyg.display

/**
 * Clients can implement a mention display handler to let the editor
 * know how to display mentions.
 */
interface MentionDisplayHandler {
    /**
     * Return the method with which to display a given mention
     */
    fun resolveMentionDisplay(text: String, url: String): TextDisplay

    /**
     * Return the method with which to display an at-room mention
     */
    fun resolveAtRoomMentionDisplay(): TextDisplay

    /**
     * Return true if the given URL is a mention one
     */
    fun isMention(url: String): Boolean
}

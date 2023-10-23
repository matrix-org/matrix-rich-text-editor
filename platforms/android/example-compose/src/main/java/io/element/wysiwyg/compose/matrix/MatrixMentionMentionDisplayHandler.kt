package io.element.wysiwyg.compose.matrix

import io.element.android.wysiwyg.display.TextDisplay
import io.element.android.wysiwyg.display.MentionDisplayHandler

/**
 * Convenience implementation of a [MentionDisplayHandler] that detects Matrix mentions and
 * displays them as default pills.
 */
class MatrixMentionMentionDisplayHandler : MentionDisplayHandler {
    override fun resolveMentionDisplay(text: String, url: String): TextDisplay =
        TextDisplay.Pill

    override fun resolveAtRoomMentionDisplay(): TextDisplay =
        TextDisplay.Pill
}

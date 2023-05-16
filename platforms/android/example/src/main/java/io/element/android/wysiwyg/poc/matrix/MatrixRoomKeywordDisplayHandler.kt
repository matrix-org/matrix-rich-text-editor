package io.element.android.wysiwyg.poc.matrix

import io.element.android.wysiwyg.display.KeywordDisplayHandler
import io.element.android.wysiwyg.display.TextDisplay

/**
 * Convenience implementation of a [KeywordDisplayHandler] that detects Matrix @room
 * mentions displays them as default pills.
 */
class MatrixRoomKeywordDisplayHandler : KeywordDisplayHandler {
    override val keywords: List<String> =
        listOf("@room")

    override fun resolveKeywordDisplay(text: String): TextDisplay =
        when (text) {
            "@room" -> TextDisplay.Pill
            else -> TextDisplay.Plain
        }
}
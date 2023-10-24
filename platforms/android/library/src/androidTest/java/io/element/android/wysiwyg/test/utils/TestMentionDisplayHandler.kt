package io.element.android.wysiwyg.test.utils

import io.element.android.wysiwyg.display.MentionDisplayHandler
import io.element.android.wysiwyg.display.TextDisplay

class TestMentionDisplayHandler(
    val textDisplay: TextDisplay = TextDisplay.Pill,
) : MentionDisplayHandler {
    override fun resolveAtRoomMentionDisplay(): TextDisplay = textDisplay
    override fun resolveMentionDisplay(text: String, url: String): TextDisplay = textDisplay
}

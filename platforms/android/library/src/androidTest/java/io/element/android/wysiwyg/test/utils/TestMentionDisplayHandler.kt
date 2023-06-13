package io.element.android.wysiwyg.test.utils

import io.element.android.wysiwyg.display.MentionDisplayHandler
import io.element.android.wysiwyg.display.TextDisplay

class TestMentionDisplayHandler(
    val textDisplay: TextDisplay,
) : MentionDisplayHandler {
    override fun resolveAtRoomMentionDisplay(): TextDisplay = TextDisplay.Pill
    override fun resolveMentionDisplay(text: String, url: String): TextDisplay = TextDisplay.Pill
}
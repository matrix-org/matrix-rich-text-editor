package io.element.android.wysiwyg.fakes

import io.element.android.wysiwyg.display.KeywordDisplayHandler
import io.element.android.wysiwyg.display.TextDisplay

class SimpleKeywordDisplayHandler(
    private val keyword: String = "@room",
    private val displayAs: TextDisplay = TextDisplay.Pill,
) : KeywordDisplayHandler {
    override val keywords: List<String>
        get() = listOf(keyword)

    override fun resolveKeywordDisplay(text: String): TextDisplay =
        when (text) {
            keyword -> displayAs
            else -> TextDisplay.Plain
        }
}
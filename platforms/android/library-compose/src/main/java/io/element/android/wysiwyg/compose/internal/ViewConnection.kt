package io.element.android.wysiwyg.compose.internal

import io.element.android.wysiwyg.view.models.InlineFormat

internal interface ViewConnection {
    fun toggleInlineFormat(inlineFormat: InlineFormat): Boolean
    fun toggleList(ordered: Boolean)
    fun toggleCodeBlock(): Boolean
    fun toggleQuote(): Boolean
    fun undo()
    fun redo()
    fun indent()
    fun unindent()
    fun setHtml(html: String)
}
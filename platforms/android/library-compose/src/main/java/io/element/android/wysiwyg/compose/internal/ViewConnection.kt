package io.element.android.wysiwyg.compose.internal

interface ViewConnection {
    fun toggleBold(): Boolean

    fun toggleItalic(): Boolean

    fun setHtml(html: String)
}
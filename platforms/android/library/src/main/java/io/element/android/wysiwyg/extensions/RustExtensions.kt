package io.element.android.wysiwyg.extensions

fun List<UShort>.string() = with(StringBuffer()) {
    this@string.forEach {
        appendCodePoint(it.toInt())
    }
    toString()
}

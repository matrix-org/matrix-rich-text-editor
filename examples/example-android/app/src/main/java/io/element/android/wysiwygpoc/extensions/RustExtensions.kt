package io.element.android.wysiwygpoc.extensions

fun List<UShort>.string() = with(StringBuffer()) {
    this@string.forEach {
        appendCodePoint(it.toInt())
    }
    toString()
}

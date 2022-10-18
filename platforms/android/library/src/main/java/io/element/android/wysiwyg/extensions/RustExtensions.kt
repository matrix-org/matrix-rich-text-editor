package io.element.android.wysiwyg.extensions

/**
 * Translates the Rust [UShort] list returned for strings into actual JVM Strings that we can use.
 */
fun List<UShort>.string() = with(StringBuffer()) {
    this@string.forEach {
        appendCodePoint(it.toInt())
    }
    toString()
}

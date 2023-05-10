package io.element.android.wysiwyg.extensions

/**
 * Translates the Rust [UShort] list returned for strings into actual JVM Strings that we can use.
 */
internal fun List<UShort>.string() = with(StringBuffer()) {
    this@string.forEach {
        appendCodePoint(it.toInt())
    }
    toString()
}

/**
 * Translates a JVM String into a Rust [UShort] list.
 */
internal fun String.toUShortList(): List<UShort> = encodeToByteArray()
    .map(Byte::toUShort)

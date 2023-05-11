package io.element.android.wysiwyg.test.utils

import android.text.TextUtils

fun CharSequence.dumpSpans(): List<String> {
    val spans = mutableListOf<String>()
    TextUtils.dumpSpans(
        this, { span ->
            val spanWithoutHash = span.split(" ").filterIndexed { index, _ ->
                index != 1
            }.joinToString(" ")

            spans.add(spanWithoutHash)
        }, ""
    )
    return spans
}
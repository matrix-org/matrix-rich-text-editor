package io.element.android.wysiwyg.utils

import io.element.android.wysiwyg.BuildConfig

fun Throwable.throwIfDebugBuild(): Unit =
    if (BuildConfig.DEBUG) {
        throw this
    } else {
        printStackTrace()
    }
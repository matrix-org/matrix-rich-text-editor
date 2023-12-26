package io.element.android.wysiwyg.utils

import io.element.android.wysiwyg.BuildConfig
import timber.log.Timber
import uniffi.wysiwyg_composer.Disposable

internal class RustCleanerTask(
    private val disposable: Disposable,
) : Runnable {
    override fun run() {
        if (BuildConfig.DEBUG) {
            Timber.d("Cleaning up disposable: $disposable")
        }
        disposable.destroy()
    }
}
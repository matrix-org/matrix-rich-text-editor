package io.element.android.wysiwyg.utils

import timber.log.Timber
import uniffi.wysiwyg_composer.Disposable

internal class RustCleanerTask(
    private val disposable: Disposable,
) : Runnable {
    override fun run() {
        Timber.d("Cleaning up disposable: $disposable")
        disposable.destroy()
    }
}
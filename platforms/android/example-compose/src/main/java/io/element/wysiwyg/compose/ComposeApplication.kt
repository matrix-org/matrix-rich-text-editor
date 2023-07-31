package io.element.wysiwyg.compose

import android.app.Application
import timber.log.Timber

class ComposeApplication: Application() {
    override fun onCreate() {
        super.onCreate()
        Timber.plant(Timber.DebugTree())
    }
}
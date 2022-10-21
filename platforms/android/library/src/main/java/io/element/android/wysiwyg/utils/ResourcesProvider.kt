package io.element.android.wysiwyg.utils

import android.app.Application
import android.util.DisplayMetrics
import androidx.annotation.ColorRes
import androidx.core.content.res.ResourcesCompat

interface ResourcesProvider {
    fun getDisplayMetrics(): DisplayMetrics

    fun getColor(@ColorRes colorId: Int): Int
}

class AndroidResourcesProvider(
    private val application: Application,
) : ResourcesProvider {

    override fun getDisplayMetrics(): DisplayMetrics {
        return application.resources.displayMetrics
    }

    override fun getColor(colorId: Int): Int {
        return ResourcesCompat.getColor(application.resources, colorId, application.theme)
    }
}

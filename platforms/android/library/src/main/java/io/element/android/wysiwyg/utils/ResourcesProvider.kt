package io.element.android.wysiwyg.utils

import android.content.Context
import android.util.DisplayMetrics
import androidx.annotation.ColorRes
import androidx.core.content.res.ResourcesCompat

interface ResourcesProvider {
    fun getDisplayMetrics(): DisplayMetrics

    fun getColor(@ColorRes colorId: Int): Int
}

class AndroidResourcesProvider(
    private val context: Context,
) : ResourcesProvider {

    override fun getDisplayMetrics(): DisplayMetrics {
        return context.resources.displayMetrics
    }

    override fun getColor(colorId: Int): Int {
        return ResourcesCompat.getColor(context.resources, colorId, context.theme)
    }
}

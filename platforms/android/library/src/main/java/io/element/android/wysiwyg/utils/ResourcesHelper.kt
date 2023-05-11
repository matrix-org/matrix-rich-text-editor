package io.element.android.wysiwyg.utils

import android.app.Application
import android.util.DisplayMetrics
import androidx.annotation.ColorRes
import androidx.annotation.Dimension
import androidx.core.content.res.ResourcesCompat

internal interface ResourcesHelper {
    fun getDisplayMetrics(): DisplayMetrics

    fun dpToPx(@Dimension(unit = Dimension.DP) dp: Int): Float

    fun getColor(@ColorRes colorId: Int): Int
}

internal class AndroidResourcesHelper(
    private val application: Application,
) : ResourcesHelper {

    override fun getDisplayMetrics(): DisplayMetrics {
        return application.resources.displayMetrics
    }

    override fun dpToPx(dp: Int): Float {
        return dp * getDisplayMetrics().density
    }

    override fun getColor(colorId: Int): Int {
        return ResourcesCompat.getColor(application.resources, colorId, application.theme)
    }
}

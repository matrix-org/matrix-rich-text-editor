package io.element.android.wysiwyg.utils

import android.app.Application
import android.util.DisplayMetrics
import androidx.annotation.Dimension

internal interface ResourcesHelper {
    fun getDisplayMetrics(): DisplayMetrics

    fun dpToPx(@Dimension(unit = Dimension.DP) dp: Int): Float
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
}

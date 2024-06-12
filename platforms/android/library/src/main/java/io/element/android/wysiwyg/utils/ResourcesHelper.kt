package io.element.android.wysiwyg.utils

import android.content.Context
import android.util.DisplayMetrics
import androidx.annotation.ColorRes
import androidx.annotation.Dimension
import androidx.core.content.res.ResourcesCompat

/**
 * This class provides access to resources needed to convert HTML to spans.
 */
internal interface ResourcesHelper {
    fun getDisplayMetrics(): DisplayMetrics

    fun dpToPx(@Dimension(unit = Dimension.DP) dp: Int): Float

    fun getColor(@ColorRes colorId: Int): Int
}

/**
 * This class provides access to Android resources needed to convert HTML to spans.
 */
internal class AndroidResourcesHelper(
    private val context: Context,
) : ResourcesHelper {

    override fun getDisplayMetrics(): DisplayMetrics {
        return context.resources.displayMetrics
    }

    override fun dpToPx(dp: Int): Float {
        return dp * getDisplayMetrics().density
    }

    override fun getColor(colorId: Int): Int {
        return ResourcesCompat.getColor(context.resources, colorId, context.theme)
    }
}

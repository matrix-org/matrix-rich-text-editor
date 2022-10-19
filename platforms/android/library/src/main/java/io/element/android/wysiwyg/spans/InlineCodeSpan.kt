package io.element.android.wysiwyg.spans

import android.content.Context
import android.os.Parcel
import android.text.ParcelableSpan
import android.text.TextPaint
import android.text.style.BackgroundColorSpan
import android.text.style.MetricAffectingSpan
import android.text.style.TypefaceSpan
import android.text.style.UpdateAppearance
import androidx.core.content.ContextCompat

/**
 * Inline code (`some code` in Markdown, <code> in HTML) Span that applies a monospaced font style
 * and adds a background color.
 */
class InlineCodeSpan : MetricAffectingSpan, UpdateAppearance, ParcelableSpan {

    private val monoTypefaceSpan: TypefaceSpan
    private val backgroundColorSpan: BackgroundColorSpan

    constructor(context: Context): super() {
        monoTypefaceSpan = TypefaceSpan("monospace")
        backgroundColorSpan = BackgroundColorSpan(
            ContextCompat.getColor(context, android.R.color.darker_gray)
        )
    }

    constructor(parcel: Parcel): super() {
        monoTypefaceSpan = requireNotNull(parcel.readParcelable(TypefaceSpan::class.java.classLoader))
        backgroundColorSpan = requireNotNull(parcel.readParcelable(BackgroundColorSpan::class.java.classLoader))
    }

    override fun updateDrawState(tp: TextPaint) {
        monoTypefaceSpan.updateDrawState(tp)
        backgroundColorSpan.updateDrawState(tp)
    }

    override fun updateMeasureState(textPaint: TextPaint) {
        monoTypefaceSpan.updateMeasureState(textPaint)
    }

    fun getSpanTypeIdInternal(): Int = 1000

    override fun describeContents(): Int = 0

    override fun writeToParcel(dest: Parcel?, flags: Int) {
        dest?.writeParcelable(monoTypefaceSpan, flags)
        dest?.writeParcelable(backgroundColorSpan, flags)
    }

    override fun getSpanTypeId(): Int = getSpanTypeIdInternal()
}

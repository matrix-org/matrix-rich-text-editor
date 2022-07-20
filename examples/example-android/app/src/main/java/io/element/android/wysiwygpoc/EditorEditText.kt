package io.element.android.wysiwygpoc

import android.content.Context
import android.text.Editable
import android.text.Spannable
import android.text.TextWatcher
import android.util.AttributeSet
import android.widget.TextView
import androidx.appcompat.widget.AppCompatEditText

class EditorEditText : AppCompatEditText {

    private val spannableFactory = object : Spannable.Factory() {
        override fun newSpannable(source: CharSequence?): Spannable {
            return source as Spannable
        }
    }

    constructor(context: Context): super(context)

    constructor(context: Context, attrs: AttributeSet?): super(context, attrs)

    constructor(context: Context, attrs: AttributeSet?, defStyleAttr: Int):
            super(context, attrs, defStyleAttr)

    init {
        setSpannableFactory(spannableFactory)
    }

    fun interface OnSelectionChangeListener {
        fun selectionChanged(start: Int, end: Int)
    }

    var selectionChangeListener: OnSelectionChangeListener? = null

    override fun onSelectionChanged(selStart: Int, selEnd: Int) {
        super.onSelectionChanged(selStart, selEnd)
        selectionChangeListener?.selectionChanged(selStart, selEnd)
    }
}

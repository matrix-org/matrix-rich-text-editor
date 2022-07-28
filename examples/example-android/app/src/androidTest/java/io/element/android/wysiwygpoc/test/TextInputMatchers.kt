package io.element.android.wysiwygpoc.test

import android.view.View
import android.widget.TextView
import org.hamcrest.BaseMatcher
import org.hamcrest.Description

object TextInputMatchers {

    class SelectionIsAt(
        private val start: Int,
        private val end: Int,
    ) : BaseMatcher<View>() {
        override fun describeTo(description: Description?) {
            description?.appendText("selection was ($start, $end)")
        }

        override fun matches(item: Any?): Boolean {
            val textView = item as? TextView ?: return false
            return textView.selectionStart == start && textView.selectionEnd == end
        }
    }
}

fun selectionIsAt(start: Int, end: Int = start) = TextInputMatchers.SelectionIsAt(start, end)

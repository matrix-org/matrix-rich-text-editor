package io.element.android.wysiwyg.test.utils

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
            description?.appendText("selection should be ($start, $end)")
        }

        override fun describeMismatch(item: Any?, description: Description?) {
            if (item is TextView) {
                val expected = "($start, $end)"
                val result = "(${item.selectionStart}, ${item.selectionEnd})"
                description?.appendText("selection should be $expected, it was $result")
            } else {
                super.describeMismatch(item, description)
            }
        }

        override fun matches(item: Any?): Boolean {
            val textView = item as? TextView ?: return false
            return textView.selectionStart == start && textView.selectionEnd == end
        }
    }
}

fun selectionIsAt(start: Int, end: Int = start) = TextInputMatchers.SelectionIsAt(start, end)

package io.element.android.wysiwyg.test

import android.graphics.Typeface
import android.text.Spannable
import android.text.style.StyleSpan
import android.widget.TextView
import androidx.test.espresso.Espresso.onView
import androidx.test.espresso.ViewAssertion
import androidx.test.espresso.accessibility.AccessibilityChecks
import androidx.test.espresso.action.ViewActions.click
import androidx.test.espresso.matcher.ViewMatchers.withId
import androidx.test.ext.junit.rules.ActivityScenarioRule
import io.element.android.wysiwyg.R
import io.element.android.wysiwyg.test.utils.ImeActions
import io.element.android.wysiwyg.test.utils.TestActivity
import org.junit.Rule
import org.junit.Test

class RichTextEditorTests {

    @get:Rule
    val scenarioRule = ActivityScenarioRule(TestActivity::class.java)

    private val ipsum = "Lorem ipsum dolor sit amet, consectetur adipiscing elit."

    init {
        AccessibilityChecks.enable()
    }

    @Test
    fun testBoldFormatting() {
        val start = 6
        val end = 11
        // Write and select text
        onView(withId(R.id.editText))
            .perform(ImeActions.commitText(ipsum))
            .perform(ImeActions.setSelection(start, end))
        // Tap on 'Bold' button
        onView(withId(R.id.formatBoldButton))
            .perform(click())
        // Sadly, this is needed
        Thread.sleep(10)
        // Check text contains a Bold StyleSpan
        onView(withId(R.id.editText))
            .check(containsSpan(StyleSpan::class.java, start, end) {
                (it as? StyleSpan)?.style == Typeface.BOLD
            })
    }
}

fun containsSpan(
    spanClass: Class<*>,
    start: Int,
    end: Int,
    extraCheck: ((Any) -> Boolean)? = null,
): ViewAssertion {
    return ViewAssertion { view, _ ->
        if (view is TextView) {
            val spannableText = view.text as? Spannable
                ?: throw AssertionError("Text is not Spannable")
            val spans = spannableText.getSpans(start, end, spanClass)
            if (spans.isEmpty()) {
                throw AssertionError("No $spanClass found in ($start, $end)")
            } else if (extraCheck != null && spans.none(extraCheck)) {
                throw AssertionError("No span matches the extra check.")
            }
        } else {
            throw AssertionError("View is not TextView")
        }
    }
}

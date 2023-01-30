package io.element.android.wysiwyg

import androidx.test.espresso.Espresso.onView
import androidx.test.espresso.assertion.ViewAssertions.matches
import androidx.test.espresso.matcher.ViewMatchers
import androidx.test.espresso.matcher.ViewMatchers.withText
import androidx.test.ext.junit.rules.ActivityScenarioRule
import io.element.android.wysiwyg.test.R
import io.element.android.wysiwyg.test.utils.TestActivity
import io.element.android.wysiwyg.test.utils.TextViewActions
import org.junit.Rule
import org.junit.Test

internal class EditorStyledTextViewTest {

    @get:Rule
    val scenarioRule = ActivityScenarioRule(TestActivity::class.java)

    @Test
    fun testSetText() {
        onView(ViewMatchers.withId(R.id.styledTextView))
            .perform(TextViewActions.setText("Hello, world"))
            .check(matches(withText("Hello, world")))
    }
}
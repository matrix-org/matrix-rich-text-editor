package io.element.android.wysiwyg

import android.graphics.Canvas
import android.graphics.Paint
import android.text.style.ReplacementSpan
import android.text.style.URLSpan
import android.widget.TextView
import androidx.core.text.buildSpannedString
import androidx.core.text.inSpans
import androidx.test.espresso.Espresso.onView
import androidx.test.espresso.action.ViewActions
import androidx.test.espresso.assertion.ViewAssertions.matches
import androidx.test.espresso.matcher.ViewMatchers
import androidx.test.espresso.matcher.ViewMatchers.withText
import androidx.test.ext.junit.rules.ActivityScenarioRule
import io.element.android.wysiwyg.test.R
import io.element.android.wysiwyg.test.utils.TestActivity
import io.element.android.wysiwyg.test.utils.TextViewActions
import io.element.android.wysiwyg.view.spans.CustomMentionSpan
import io.element.android.wysiwyg.view.spans.LinkSpan
import io.element.android.wysiwyg.view.spans.PillSpan
import org.junit.Assert
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

    @Test
    fun testSetHtml() {
        scenarioRule.scenario.onActivity {
            it.findViewById<EditorStyledTextView>(R.id.styledTextView).apply {
                setHtml("<p>Hello, world</p>")
            }
        }
        onView(ViewMatchers.withId(R.id.styledTextView))
            .check(matches(withText("Hello, world")))
    }

    @Test
    fun testSetHtmlWithMention() {
        scenarioRule.scenario.onActivity {
            it.findViewById<EditorStyledTextView>(R.id.styledTextView).apply {
                setHtml("<p>Hello, <a href='https://matrix.to/#/@alice:matrix.org'>@Alice</a></p>")
            }
        }
        onView(ViewMatchers.withId(R.id.styledTextView))
            .check(matches(withText("Hello, @Alice")))
    }

    @Test
    fun testUrlClicks() {
        var pass = false
        scenarioRule.scenario.onActivity {
            it.findViewById<EditorStyledTextView>(R.id.styledTextView).apply {
                val spanned = buildSpannedString {
                    inSpans(URLSpan("")) {
                        append("Hello, world")
                    }
                }
                setText(spanned, TextView.BufferType.SPANNABLE)
                onLinkClickedListener = {
                    pass = true
                }
            }
        }
        onView(ViewMatchers.withId(R.id.styledTextView))
            .check(matches(withText("Hello, world")))
            .perform(ViewActions.click())

        Assert.assertTrue(pass)
    }

    @Test
    fun testLinkClicks() {
        var pass = false
        scenarioRule.scenario.onActivity {
            it.findViewById<EditorStyledTextView>(R.id.styledTextView).apply {
                val spanned = buildSpannedString {
                    inSpans(LinkSpan("")) {
                        append("Hello, world")
                    }
                }
                setText(spanned, TextView.BufferType.SPANNABLE)
                onLinkClickedListener = {
                    pass = true
                }
            }
        }
        onView(ViewMatchers.withId(R.id.styledTextView))
            .check(matches(withText("Hello, world")))
            .perform(ViewActions.click())

        Assert.assertTrue(pass)
    }

    @Test
    fun testPillSpanClicks() {
        var pass = false
        scenarioRule.scenario.onActivity {
            it.findViewById<EditorStyledTextView>(R.id.styledTextView).apply {
                val spanned = buildSpannedString {
                    inSpans(PillSpan(backgroundColor = 0, url = "")) {
                        append("Hello, world")
                    }
                }
                setText(spanned, TextView.BufferType.SPANNABLE)
                onLinkClickedListener = {
                    pass = true
                }
            }
        }
        onView(ViewMatchers.withId(R.id.styledTextView))
            .check(matches(withText("Hello, world")))
            .perform(ViewActions.click())

        Assert.assertTrue(pass)
    }

    @Test
    fun testCustomMentionSpanClicks() {
        var pass = false
        scenarioRule.scenario.onActivity {
            it.findViewById<EditorStyledTextView>(R.id.styledTextView).apply {
                val spanned = buildSpannedString {
                    inSpans(CustomMentionSpan(DummyReplacementSpan, url = "")) {
                        append("Hello, world")
                    }
                }
                setText(spanned, TextView.BufferType.SPANNABLE)
                onLinkClickedListener = {
                    pass = true
                }
            }
        }
        onView(ViewMatchers.withId(R.id.styledTextView))
            .check(matches(withText("Hello, world")))
            .perform(ViewActions.click())

        Assert.assertTrue(pass)
    }
}

object DummyReplacementSpan : ReplacementSpan() {
    override fun getSize(paint: Paint, text: CharSequence?, start: Int, end: Int, fm: Paint.FontMetricsInt?): Int = 1

    override fun draw(canvas: Canvas, text: CharSequence?, start: Int, end: Int, x: Float, top: Int, y: Int, bottom: Int, paint: Paint)  = Unit

}

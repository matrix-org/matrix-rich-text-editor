package io.element.android.wysiwyg.test.utils

import android.view.View
import android.widget.TextView
import android.widget.TextView.BufferType
import androidx.test.espresso.UiController
import androidx.test.espresso.ViewAction
import androidx.test.espresso.matcher.ViewMatchers.isDisplayed
import io.element.android.wysiwyg.EditorStyledTextView
import org.hamcrest.Matcher

object TextViewAction {
    class SetText(
        private val text: CharSequence,
        private val type: BufferType = BufferType.NORMAL,
    ) : ViewAction {
        override fun getConstraints(): Matcher<View> = isDisplayed()

        override fun getDescription(): String = "Set text to $text"

        override fun perform(uiController: UiController?, view: View?) {
            val textView = view as? TextView ?: return
            textView.setText(text, type)
        }
    }

    class SetHtml(
        private val html: String,
    ) : ViewAction {
        override fun getConstraints(): Matcher<View> = isDisplayed()

        override fun getDescription(): String = "Set html to $html"

        override fun perform(uiController: UiController?, view: View?) {
            val textView = view as? EditorStyledTextView ?: return
            textView.setHtml(html)
        }
    }

    class SetOnLinkClickedListener(
        private val listener: (String) -> Unit,
    ) : ViewAction {
        override fun getConstraints(): Matcher<View> = isDisplayed()

        override fun getDescription(): String = "Set link clicked listener"

        override fun perform(uiController: UiController?, view: View?) {
            val textView = view as? EditorStyledTextView ?: return
            textView.onLinkClickedListener = listener
        }
    }
}

object TextViewActions {
    fun setText(text: CharSequence, type: BufferType = BufferType.NORMAL) = TextViewAction.SetText(text, type)
    fun setHtml(html: String) = TextViewAction.SetHtml(html)
    fun setOnLinkClickedListener(listener: (String) -> Unit) = TextViewAction.SetOnLinkClickedListener(listener)
}

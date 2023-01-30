package io.element.android.wysiwyg.test.utils

import android.view.View
import android.widget.TextView
import androidx.test.espresso.UiController
import androidx.test.espresso.ViewAction
import androidx.test.espresso.matcher.ViewMatchers.isDisplayed
import org.hamcrest.Matcher

object TextViewAction {
    class SetText(
        private val text: String,
    ) : ViewAction {
        override fun getConstraints(): Matcher<View> = isDisplayed()

        override fun getDescription(): String = "Set text to $text"

        override fun perform(uiController: UiController?, view: View?) {
            val textView = view as? TextView ?: return
            textView.setText(text)
        }
    }

}

object TextViewActions {
    fun setText(text: String) = TextViewAction.SetText(text)
}

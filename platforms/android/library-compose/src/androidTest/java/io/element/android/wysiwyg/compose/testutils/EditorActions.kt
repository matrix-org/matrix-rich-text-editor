package io.element.android.wysiwyg.compose.testutils

import android.view.View
import androidx.appcompat.widget.AppCompatEditText
import androidx.test.espresso.UiController
import androidx.test.espresso.ViewAction
import androidx.test.espresso.matcher.ViewMatchers.isDisplayed
import org.hamcrest.Matcher

object Editor {
    class SetSelection(
        private val start: Int,
        private val end: Int,
    ) : ViewAction {
        override fun getConstraints(): Matcher<View> = isDisplayed()

        override fun getDescription(): String = "Set selection to $start, $end"

        override fun perform(uiController: UiController?, view: View?) {
            val editor = view as? AppCompatEditText ?: return
            editor.setSelection(start, end)
        }
    }

}

object EditorActions {
    fun setSelection(start: Int, end: Int) = Editor.SetSelection(start, end)
}

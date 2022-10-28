package io.element.android.wysiwyg.test.utils

import android.view.View
import androidx.test.espresso.UiController
import androidx.test.espresso.ViewAction
import androidx.test.espresso.matcher.ViewMatchers
import org.hamcrest.Matcher

class AnyAction(
    private val description: String = "Custom view action",
    private val action: (View) -> Unit,
) : ViewAction {
    override fun getConstraints(): Matcher<View> {
        return ViewMatchers.isDisplayed()
    }

    override fun getDescription(): String = description

    override fun perform(uiController: UiController, view: View) {
        action(view)
        uiController.loopMainThreadUntilIdle()
    }
}

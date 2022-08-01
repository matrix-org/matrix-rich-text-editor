package io.element.android.wysiwyg.test

import android.view.KeyEvent
import androidx.test.espresso.Espresso.onView
import androidx.test.espresso.action.ViewActions.*
import androidx.test.espresso.assertion.ViewAssertions.matches
import androidx.test.espresso.matcher.ViewMatchers.withId
import androidx.test.espresso.matcher.ViewMatchers.withText
import androidx.test.ext.junit.rules.ActivityScenarioRule
import androidx.test.ext.junit.runners.AndroidJUnit4
import io.element.android.wysiwyg.test.utils.ImeActions
import io.element.android.wysiwyg.test.utils.TestActivity
import io.element.android.wysiwyg.test.utils.selectionIsAt
import org.junit.After
import org.junit.Rule
import org.junit.Test
import org.junit.runner.RunWith

@RunWith(AndroidJUnit4::class)
class EditorEditTextInputTests {

    @get:Rule
    val scenarioRule = ActivityScenarioRule(TestActivity::class.java)

    private val ipsum = "Lorem ipsum dolor sit amet, consectetur adipiscing elit."

    @After
    fun cleanUp() {
        // Finish composing just in case, to prevent clashes between test cases
        onView(withId(R.id.editor)).perform(ImeActions.finishComposingText())
    }

    @Test
    fun testHardwareKeyboardTyping() {
        onView(withId(R.id.editor))
            .perform(typeText(ipsum))
            .check(matches(withText(ipsum.asHtml())))
    }

    @Test
    fun testReplace() {
        onView(withId(R.id.editor))
            .perform(replaceText(ipsum))
            .check(matches(withText(ipsum.asHtml())))
    }

    @Test
    fun testImeSetComposingText() {
        onView(withId(R.id.editor))
            .perform(ImeActions.setComposingText("Test"))
            .check(matches(withText("Test")))
    }

    @Test
    fun testImeCommitText() {
        onView(withId(R.id.editor))
            .perform(ImeActions.setComposingText("Test"))
                // This should actually be automatic
            .perform(ImeActions.setComposingRegion(0, 4))
                // This should replace "Test" with "Testing"
            .perform(ImeActions.commitText("Testing"))
            .check(matches(withText("Testing")))
    }

    @Test
    fun testImeBackspace() {
        onView(withId(R.id.editor))
            .perform(ImeActions.setComposingText("Test"))
            .perform(ImeActions.backspace())
            .check(matches(withText("Tes")))
    }

    @Test
    fun testSetSelection() {
        onView(withId(R.id.editor))
            .perform(ImeActions.setComposingText("Test"))
            .perform(ImeActions.setSelection(2))
            .check(matches(selectionIsAt(2)))
    }

    @Test
    fun testImeDeleteSurroundingText() {
        onView(withId(R.id.editor))
            .perform(ImeActions.setComposingText("Test"))
            .perform(ImeActions.setSelection(2))
            .perform(ImeActions.deleteSurrounding(1, 1))
            .check(matches(withText("Tt")))
    }

    @Test
    fun testHardwareKeyMovementNotIntercepted() {
        onView(withId(R.id.editor))
            .perform(ImeActions.setComposingText("Test"))
            .perform(pressKey(KeyEvent.KEYCODE_DPAD_LEFT))
            .check(matches(selectionIsAt(3)))
    }

}

private fun String.asHtml(): String {
    // Replace regular whitespace (0x20) with HTML's '&nsbp;' (0xa0)
    return this.replace(Char(0x20), Char(0xa0))
}

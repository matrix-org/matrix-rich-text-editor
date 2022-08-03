package io.element.android.wysiwyg.test

import android.view.KeyEvent
import androidx.test.espresso.Espresso.onView
import androidx.test.espresso.accessibility.AccessibilityChecks
import androidx.test.espresso.action.ViewActions.*
import androidx.test.espresso.assertion.ViewAssertions.matches
import androidx.test.espresso.matcher.ViewMatchers.withId
import androidx.test.espresso.matcher.ViewMatchers.withText
import androidx.test.ext.junit.rules.ActivityScenarioRule
import androidx.test.ext.junit.runners.AndroidJUnit4
import androidx.test.filters.FlakyTest
import io.element.android.wysiwyg.test.utils.ImeActions
import io.element.android.wysiwyg.test.utils.TestActivity
import io.element.android.wysiwyg.test.utils.selectionIsAt
import org.junit.After
import org.junit.Ignore
import org.junit.Rule
import org.junit.Test
import org.junit.runner.RunWith
import io.element.android.wysiwyg.R

@RunWith(AndroidJUnit4::class)
class EditorEditTextInputTests {

    @get:Rule
    val scenarioRule = ActivityScenarioRule(TestActivity::class.java)

    private val ipsum = "Lorem ipsum dolor sit amet, consectetur adipiscing elit."

    init {
        AccessibilityChecks.enable()
    }

    @After
    fun cleanUp() {
        // Finish composing just in case, to prevent clashes between test cases
        onView(withId(R.id.editText)).perform(ImeActions.finishComposingText())
    }

    @Test
    fun testHardwareKeyboardTyping() {
        onView(withId(R.id.editText))
            .perform(typeText(ipsum))
            .check(matches(withText(ipsum.asHtml())))
    }

    @Test
    fun testReplace() {
        onView(withId(R.id.editText))
            .perform(replaceText(ipsum))
            .check(matches(withText(ipsum.asHtml())))
    }

    @Test
    fun testImeSetComposingText() {
        onView(withId(R.id.editText))
            .perform(ImeActions.setComposingText("Test"))
            .check(matches(withText("Test")))
    }

    @Test
    fun testImeCommitText() {
        onView(withId(R.id.editText))
            .perform(ImeActions.setComposingText("Test"))
                // This should actually be automatic
            .perform(ImeActions.setComposingRegion(0, 4))
                // This should replace "Test" with "Testing"
            .perform(ImeActions.commitText("Testing"))
            .check(matches(withText("Testing")))
    }

    @Test
    fun testImeBackspace() {
        onView(withId(R.id.editText))
            .perform(ImeActions.setComposingText("Test"))
            .perform(ImeActions.backspace())
            .check(matches(withText("Tes")))
    }

    @Test
    fun testSetSelection() {
        onView(withId(R.id.editText))
            .perform(ImeActions.setComposingText("Test"))
            .perform(ImeActions.setSelection(2))
            .check(matches(selectionIsAt(2)))
    }

    @Test
    fun testImeDeleteSurroundingText() {
        onView(withId(R.id.editText))
            .perform(ImeActions.setComposingText("Test"))
            .perform(ImeActions.setSelection(2))
            .perform(ImeActions.deleteSurrounding(1, 1))
            .check(matches(withText("Tt")))
    }

    @Test
    @FlakyTest(detail = "It might be a race condition, but some times it does not work at all.")
    fun testHardwareKeyMovementNotIntercepted() {
        onView(withId(R.id.editText))
            .perform(ImeActions.setComposingText("Test"))
            .perform(pressKey(KeyEvent.KEYCODE_DPAD_LEFT))
            .check(matches(selectionIsAt(3)))
    }

    @Test
    fun testJapaneseInputHiraganaToKanji() {
        onView(withId(R.id.editText))
            .perform(ImeActions.setComposingText("う")) // U (Hiragana)
            .perform(ImeActions.setComposingText("み")) // Mi (Hiragana)
            .perform(ImeActions.commitText("海")) // Umi (Kanji through autocomplete)
            .check(matches(withText("海")))
    }

    @Test
    fun testJapaneseInputHiraganaDeletion() {
        onView(withId(R.id.editText))
            .perform(ImeActions.setComposingText("うみ")) // Umi (Hiragana)
            .perform(ImeActions.backspace())
            .check(matches(withText("う"))) // U (Hiragana)
    }

    @Test
    fun testJapaneseInputKanjiDeletion() {
        onView(withId(R.id.editText))
            .perform(ImeActions.commitText("海")) // Umi (Kanji through autocomplete)
            .perform(ImeActions.backspace())
            .check(matches(withText("")))
    }

    @Test
    fun testKoreanInputSeparateCharactersJoined() {
        onView(withId(R.id.editText))
            .perform(ImeActions.setComposingText("ㅂ")) // B/P (Piup)
            .perform(ImeActions.setComposingText("바")) // B/P + A
            .perform(ImeActions.setComposingText("밥")) // B/P + A + B/P
            .check(matches(withText("밥")))
    }

    @Test
    @Ignore("These are failing at the moment. The whole text is deleted.")
    fun testKoreanInputSeparateCharactersDeletion() {
        onView(withId(R.id.editText))
            .perform(ImeActions.setComposingText("ㅂ")) // B/P (Piup)
            .perform(ImeActions.setComposingText("바")) // B/P + A
            .perform(ImeActions.backspace())
            .check(matches(withText("ㅂ")))
    }

    @Test
    @Ignore("These are failing at the moment. The whole text is deleted.")
    fun testKoreanInputJoinedCharactersDeletion() {
        onView(withId(R.id.editText))
            .perform(ImeActions.commitText("밥")) // Bap (autocomplete)
            .perform(ImeActions.backspace())
            .check(matches(withText("바")))
    }

}

private fun String.asHtml(): String {
    // Replace regular whitespace (0x20) with HTML's '&nsbp;' (0xa0)
    return this.replace(Char(0x20), Char(0xa0))
}

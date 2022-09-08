package io.element.android.wysiwyg.test

import android.text.style.BulletSpan
import android.text.style.UnderlineSpan
import android.view.KeyEvent
import android.view.View
import android.widget.TextView
import androidx.core.text.getSpans
import androidx.test.espresso.Espresso.onView
import androidx.test.espresso.accessibility.AccessibilityChecks
import androidx.test.espresso.action.ViewActions.*
import androidx.test.espresso.assertion.ViewAssertions.matches
import androidx.test.espresso.matcher.BoundedDiagnosingMatcher
import androidx.test.espresso.matcher.ViewMatchers.withId
import androidx.test.espresso.matcher.ViewMatchers.withText
import androidx.test.ext.junit.rules.ActivityScenarioRule
import androidx.test.ext.junit.runners.AndroidJUnit4
import androidx.test.filters.FlakyTest
import io.element.android.wysiwyg.EditorEditText
import io.element.android.wysiwyg.R
import io.element.android.wysiwyg.spans.OrderedListSpan
import io.element.android.wysiwyg.test.utils.EditorActions
import io.element.android.wysiwyg.test.utils.ImeActions
import io.element.android.wysiwyg.test.utils.TestActivity
import io.element.android.wysiwyg.test.utils.selectionIsAt
import org.hamcrest.Description
import org.junit.After
import org.junit.Ignore
import org.junit.Rule
import org.junit.Test
import org.junit.runner.RunWith

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
            .check(matches(withText(ipsum)))
    }

    @Test
    fun testReplace() {
        onView(withId(R.id.editText))
            .perform(replaceText(ipsum))
            .check(matches(withText(ipsum)))
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
    @FlakyTest(detail = "Sometimes the pressKey event doesn't reach the view.")
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
    fun testAddingLink() {
        onView(withId(R.id.editText))
            .perform(ImeActions.setComposingText("a link to set"))
            .perform(ImeActions.setSelection(2, 6))
            .perform(EditorActions.setLink("https://element.io"))
            .check(matches(TextViewMatcher {
                // TODO: once we decide a Span for links, replace `UnderlineSpan`
                it.editableText.getSpans<UnderlineSpan>(start = 2, end = 6).isNotEmpty()
            }))
    }

    @Test
    fun testAddingOrderedList() {
        onView(withId(R.id.editText))
            .perform(EditorActions.toggleList(true))
            .perform(ImeActions.setComposingText("A list item"))
            .perform(ImeActions.enter())
            .perform(ImeActions.setComposingText("Another list item"))
            .check(matches(withText("\u200bA list item\n\u200bAnother list item")))
            .check(matches(TextViewMatcher {
                // Has 2 OrderedListSpans (prefixes, 1 per line)
                it.editableText.getSpans<OrderedListSpan>().count() == 2
            }))
    }

    @Test
    fun testAddingUnorderedList() {
        onView(withId(R.id.editText))
            .perform(EditorActions.toggleList(false))
            .perform(ImeActions.setComposingText("A list item"))
            .perform(ImeActions.enter())
            .perform(ImeActions.setComposingText("Another list item"))
            .check(matches(withText("\u200bA list item\n\u200bAnother list item")))
            .check(matches(TextViewMatcher {
                // Has 2 OrderedListSpans (prefixes, 1 per line)
                it.editableText.getSpans<BulletSpan>().count() == 2
            }))
    }

    @Test
    fun testUndo() {
        onView(withId(R.id.editText))
            .perform(ImeActions.setComposingText("Some text to undo"))
            .check(matches(withText("Some text to undo")))
            .perform(EditorActions.undo())
            .check(matches(withText("")))
    }

    @Test
    fun testRedo() {
        onView(withId(R.id.editText))
            .perform(ImeActions.setComposingText("Some text to undo"))
            .check(matches(withText("Some text to undo")))
            .perform(EditorActions.undo())
            .check(matches(withText("")))
            .perform(EditorActions.redo())
            .check(matches(withText("Some text to undo")))
    }

    // About IME backspace on Korean, that's handled by the IME, which automatically seems to either
    // remove the last code unit from the code point, or 'undo' the last action and send the last
    // compositing text.
    @Test
    @Ignore("These are failing at the moment. The whole text is deleted. Note that this backspace action mimicks HW keyboard backspace, not IME.")
    fun testKoreanInputSeparateCharactersDeletion() {
        onView(withId(R.id.editText))
            .perform(ImeActions.setComposingText("ㅂ")) // B/P (Piup)
            .perform(ImeActions.setComposingText("바")) // B/P + A
            .perform(ImeActions.backspace())
            .check(matches(withText("ㅂ")))
    }

    @Test
    @Ignore("These are failing at the moment. The whole text is deleted. Note that this backspace action mimicks HW keyboard backspace, not IME.")
    fun testKoreanInputJoinedCharactersDeletion() {
        onView(withId(R.id.editText))
            .perform(ImeActions.commitText("밥")) // Bap (autocomplete)
            .perform(ImeActions.backspace())
            .check(matches(withText("바")))
    }

}

class TextViewMatcher(
    private val check: (TextView) -> Boolean
) : BoundedDiagnosingMatcher<View, EditorEditText>(EditorEditText::class.java) {
    override fun matchesSafely(item: EditorEditText?, mismatchDescription: Description?): Boolean {
        return if (item != null && check(item)) {
            true
        } else {
            mismatchDescription?.appendText("Did not match TextViewMatcher")
            false
        }
    }

    override fun describeMoreTo(description: Description?) {
        description?.appendText("Matches TextViewMatcher")
    }

}

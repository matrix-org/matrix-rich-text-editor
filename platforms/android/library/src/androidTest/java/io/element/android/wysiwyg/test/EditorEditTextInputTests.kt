package io.element.android.wysiwyg.test

import android.graphics.Typeface
import android.text.Spannable
import android.text.style.BulletSpan
import android.text.style.StyleSpan
import android.text.style.UnderlineSpan
import android.view.KeyEvent
import android.view.View
import android.widget.TextView
import androidx.core.text.getSpans
import androidx.test.espresso.Espresso.onView
import androidx.test.espresso.ViewAssertion
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
import io.element.android.wysiwyg.inputhandlers.models.InlineFormat
import io.element.android.wysiwyg.spans.OrderedListSpan
import io.element.android.wysiwyg.test.utils.EditorActions
import io.element.android.wysiwyg.test.utils.ImeActions
import io.element.android.wysiwyg.test.utils.TestActivity
import io.element.android.wysiwyg.test.utils.selectionIsAt
import org.hamcrest.Description
import org.hamcrest.MatcherAssert.assertThat
import org.hamcrest.Matchers.equalTo
import org.junit.*
import org.junit.runner.RunWith
import uniffi.wysiwyg_composer.ComposerAction
import uniffi.wysiwyg_composer.MenuState

@RunWith(AndroidJUnit4::class)
class EditorEditTextInputTests {

    @get:Rule
    val scenarioRule = ActivityScenarioRule(TestActivity::class.java)

    private val ipsum = "Lorem Ipsum is simply dummy text of the printing and typesetting industry."

    init {
        AccessibilityChecks.enable()
    }

    @After
    fun cleanUp() {
        // Finish composing just in case, to prevent clashes between test cases
        onView(withId(R.id.rich_text_edit_text)).perform(ImeActions.finishComposingText())
    }

    @Test
    fun testHardwareKeyboardTyping() {
        onView(withId(R.id.rich_text_edit_text))
            .perform(typeText(ipsum))
            .check(matches(withText(ipsum)))
    }

    @Test
    fun testHardwareKeyboardBackspace() {
        onView(withId(R.id.rich_text_edit_text))
            .perform(typeText("Test"))
            .perform(pressKey(KeyEvent.KEYCODE_DEL))
            .check(matches(withText("Tes")))
            // Type a character again to make sure the composer and the UI match
            .perform(typeText("t"))
            .check(matches(withText("Test")))
    }

    @Test
    fun testReplace() {
        onView(withId(R.id.rich_text_edit_text))
            .perform(replaceText(ipsum))
            .check(matches(withText(ipsum)))
    }

    @Test
    fun testImeSetComposingText() {
        onView(withId(R.id.rich_text_edit_text))
            .perform(ImeActions.setComposingText("Test"))
            .check(matches(withText("Test")))
    }

    @Test
    fun testImeCommitText() {
        onView(withId(R.id.rich_text_edit_text))
            .perform(ImeActions.setComposingText("Test"))
                // This should actually be automatic
            .perform(ImeActions.setComposingRegion(0, 4))
                // This should replace "Test" with "Testing"
            .perform(ImeActions.commitText("Testing"))
            .check(matches(withText("Testing")))
    }

    @Test
    fun testImeBackspace() {
        onView(withId(R.id.rich_text_edit_text))
            .perform(ImeActions.setComposingText("Test"))
            .perform(ImeActions.backspace())
            .check(matches(withText("Tes")))
    }

    @Test
    fun testSetSelection() {
        onView(withId(R.id.rich_text_edit_text))
            .perform(ImeActions.setComposingText("Test"))
            .perform(ImeActions.setSelection(2))
            .check(matches(selectionIsAt(2)))
    }

    @Test
    fun testImeDeleteSurroundingText() {
        onView(withId(R.id.rich_text_edit_text))
            .perform(ImeActions.setComposingText("Test"))
            .perform(ImeActions.setSelection(2))
            .perform(ImeActions.deleteSurrounding(1, 1))
            .check(matches(withText("Tt")))
    }

    @Test
    fun testHardwareKeyMovementNotIntercepted() {
        onView(withId(R.id.rich_text_edit_text))
            .perform(typeText("Test"))
            .perform(pressKey(KeyEvent.KEYCODE_DPAD_LEFT))
            .check(matches(selectionIsAt(3)))
            .perform(pressKey(KeyEvent.KEYCODE_DPAD_LEFT))
            .check(matches(selectionIsAt(2)))
    }

    @Test
    fun testJapaneseInputHiraganaToKanji() {
        onView(withId(R.id.rich_text_edit_text))
            .perform(ImeActions.setComposingText("う")) // U (Hiragana)
            .perform(ImeActions.setComposingText("み")) // Mi (Hiragana)
            .perform(ImeActions.commitText("海")) // Umi (Kanji through autocomplete)
            .check(matches(withText("海")))
    }

    @Test
    fun testJapaneseInputHiraganaDeletion() {
        onView(withId(R.id.rich_text_edit_text))
            .perform(ImeActions.setComposingText("うみ")) // Umi (Hiragana)
            .perform(ImeActions.backspace())
            .check(matches(withText("う"))) // U (Hiragana)
    }

    @Test
    fun testJapaneseInputKanjiDeletion() {
        onView(withId(R.id.rich_text_edit_text))
            .perform(ImeActions.commitText("海")) // Umi (Kanji through autocomplete)
            .perform(ImeActions.backspace())
            .check(matches(withText("")))
    }

    @Test
    fun testKoreanInputSeparateCharactersJoined() {
        onView(withId(R.id.rich_text_edit_text))
            .perform(ImeActions.setComposingText("ㅂ")) // B/P (Piup)
            .perform(ImeActions.setComposingText("바")) // B/P + A
            .perform(ImeActions.setComposingText("밥")) // B/P + A + B/P
            .check(matches(withText("밥")))
    }

    @Test
    fun testAddingLink() {
        onView(withId(R.id.rich_text_edit_text))
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
        onView(withId(R.id.rich_text_edit_text))
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
        onView(withId(R.id.rich_text_edit_text))
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
        onView(withId(R.id.rich_text_edit_text))
            .perform(ImeActions.setComposingText("Some text to undo"))
            .check(matches(withText("Some text to undo")))
            .perform(EditorActions.undo())
            .check(matches(withText("")))
    }

    @Test
    fun testRedo() {
        onView(withId(R.id.rich_text_edit_text))
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
        onView(withId(R.id.rich_text_edit_text))
            .perform(ImeActions.setComposingText("ㅂ")) // B/P (Piup)
            .perform(ImeActions.setComposingText("바")) // B/P + A
            .perform(ImeActions.backspace())
            .check(matches(withText("ㅂ")))
    }

    @Test
    @Ignore("These are failing at the moment. The whole text is deleted. Note that this backspace action mimicks HW keyboard backspace, not IME.")
    fun testKoreanInputJoinedCharactersDeletion() {
        onView(withId(R.id.rich_text_edit_text))
            .perform(ImeActions.commitText("밥")) // Bap (autocomplete)
            .perform(ImeActions.backspace())
            .check(matches(withText("바")))
    }

    @Test
    fun testBoldFormatting() {
        val start = 6
        val end = 11
        // Write and select text
        onView(withId(R.id.rich_text_edit_text))
            .perform(ImeActions.commitText(ipsum))
            .perform(ImeActions.setSelection(start, end))
            .perform(EditorActions.toggleFormat(InlineFormat.Bold))
            // Check text contains a Bold StyleSpan
            .check(containsSpan(StyleSpan::class.java, start, end) {
                (it as? StyleSpan)?.style == Typeface.BOLD
            })
    }

    @Test
    fun testMenuStateChangedListener() {
        var isItalicHighlighted = false
        scenarioRule.scenario.onActivity {
            it.findViewById<EditorEditText>(R.id.rich_text_edit_text).menuStateChangedListener =
                EditorEditText.OnMenuStateChangedListener { state ->
                    if (state is MenuState.Update) {
                        if (state.reversedActions.contains(ComposerAction.Italic)) {
                            isItalicHighlighted = true
                        }
                    }
                }
        }

        val start = 6
        val end = 11
        onView(withId(R.id.rich_text_edit_text))
            .perform(ImeActions.commitText(ipsum))
            .perform(ImeActions.setSelection(start, end))
            .perform(EditorActions.toggleFormat(InlineFormat.Italic))

        Assert.assertTrue(isItalicHighlighted)
    }

    @Test
    fun testSetPlainText_ignoresHtml() {
        scenarioRule.scenario.onActivity {
            it.findViewById<EditorEditText>(R.id.rich_text_edit_text)
                .setPlainText("<b>$ipsum</b>")
        }
        onView(withId(R.id.rich_text_edit_text))
            .check(matches(withText("<b>$ipsum</b>")))
    }

    @Test
    fun testGetPlainText_stripsHtml() {
        scenarioRule.scenario.onActivity {
            val editText = it.findViewById<EditorEditText>(R.id.rich_text_edit_text)
            editText.setHtml("<b>$ipsum</b>")

            val plainText = editText.getPlainText()

            assertThat(plainText, equalTo(ipsum))
        }
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

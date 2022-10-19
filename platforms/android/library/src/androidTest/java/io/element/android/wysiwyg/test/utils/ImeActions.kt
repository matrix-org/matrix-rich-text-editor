package io.element.android.wysiwyg.test.utils

import android.view.View
import android.view.inputmethod.EditorInfo
import androidx.test.espresso.UiController
import androidx.test.espresso.ViewAction
import androidx.test.espresso.matcher.ViewMatchers.isDisplayed
import io.element.android.wysiwyg.InterceptInputConnection
import org.hamcrest.Matcher

object Ime {
    class SetComposingText(
        private val text: String,
    ) : ViewAction {

        override fun getConstraints(): Matcher<View> = isDisplayed()

        override fun getDescription(): String = "Set composing text in soft keyboard"

        override fun perform(uiController: UiController?, view: View?) {
            if (view == null) return
            val editorInfo = EditorInfo()
            val inputConnection = view.onCreateInputConnection(editorInfo)
            inputConnection.setComposingText(text, 1)
        }
    }

    class CommitText(
        private val text: String,
    ) : ViewAction {

        override fun getConstraints(): Matcher<View> = isDisplayed()

        override fun getDescription(): String = "Commit text in soft keyboard"

        override fun perform(uiController: UiController?, view: View?) {
            if (view == null) return
            val editorInfo = EditorInfo()
            val inputConnection = view.onCreateInputConnection(editorInfo)
            inputConnection.commitText(text, 1)
        }
    }

    class SetComposingRegion(
        private val start: Int,
        private val end: Int,
    ) : ViewAction {

        override fun getConstraints(): Matcher<View> = isDisplayed()

        override fun getDescription(): String = "Set composing region in soft keyboard"

        override fun perform(uiController: UiController?, view: View?) {
            if (view == null) return
            val editorInfo = EditorInfo()
            val inputConnection = view.onCreateInputConnection(editorInfo)
            inputConnection.setComposingRegion(start, end)
        }
    }

    class BackSpace : ViewAction {
        override fun getConstraints(): Matcher<View> = isDisplayed()

        override fun getDescription(): String = "Press backspace in soft keyboard"

        override fun perform(uiController: UiController?, view: View?) {
            if (view == null) return
            val editorInfo = EditorInfo()
            val inputConnection = view.onCreateInputConnection(editorInfo)
            (inputConnection as? InterceptInputConnection)?.onHardwareBackspaceKey()
        }
    }

    class FinishComposingText : ViewAction {
        override fun getConstraints(): Matcher<View> = isDisplayed()

        override fun getDescription(): String = "Finish composing text in soft keyboard"

        override fun perform(uiController: UiController?, view: View?) {
            if (view == null) return
            val editorInfo = EditorInfo()
            val inputConnection = view.onCreateInputConnection(editorInfo)
            inputConnection.finishComposingText()
        }
    }

    class SetSelection(
        private val start: Int,
        private val end: Int,
    ) : ViewAction {
        override fun getConstraints(): Matcher<View> = isDisplayed()

        override fun getDescription(): String = "Sets selection in TextView"

        override fun perform(uiController: UiController?, view: View?) {
            if (view == null) return
            val editorInfo = EditorInfo()
            val inputConnection = view.onCreateInputConnection(editorInfo)
            inputConnection.setSelection(start, end)
        }
    }

    class DeleteSurroundingText(
        private val before: Int,
        private val after: Int,
    ) : ViewAction {
        override fun getConstraints(): Matcher<View> = isDisplayed()

        override fun getDescription(): String = "Deletes $before characters before the selection and $after characters after it in TextView"

        override fun perform(uiController: UiController?, view: View?) {
            if (view == null) return
            val editorInfo = EditorInfo()
            val inputConnection = view.onCreateInputConnection(editorInfo)
            inputConnection.deleteSurroundingText(before, after)
        }
    }

    object Enter : ViewAction {
        override fun getConstraints(): Matcher<View> = isDisplayed()

        override fun getDescription(): String = "Simulates pressing the Enter key"

        override fun perform(uiController: UiController?, view: View?) {
            if (view == null) return
            val editorInfo = EditorInfo()
            val inputConnection = view.onCreateInputConnection(editorInfo) as? InterceptInputConnection ?: return
            inputConnection.onHardwareEnterKey()
        }

    }
}

object ImeActions {
    fun setComposingRegion(start: Int, end: Int) = Ime.SetComposingRegion(start, end)
    fun setComposingText(text: String) = Ime.SetComposingText(text)
    fun commitText(text: String) = Ime.CommitText(text)
    fun finishComposingText() = Ime.FinishComposingText()
    fun backspace() = Ime.BackSpace()
    fun setSelection(start: Int, end: Int = start) = Ime.SetSelection(start, end)
    fun deleteSurrounding(before: Int = 0, after: Int = 0) = Ime.DeleteSurroundingText(before, after)
    fun enter() = Ime.Enter
}

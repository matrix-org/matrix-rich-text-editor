package io.element.android.wysiwyg.test.utils

import android.text.Editable
import android.view.View
import androidx.core.widget.addTextChangedListener
import androidx.test.espresso.UiController
import androidx.test.espresso.ViewAction
import androidx.test.espresso.matcher.ViewMatchers.isDisplayed
import io.element.android.wysiwyg.EditorEditText
import io.element.android.wysiwyg.inputhandlers.models.InlineFormat
import io.element.android.wysiwyg.links.LinkDisplayHandler
import io.element.android.wysiwyg.utils.RustErrorCollector
import org.hamcrest.Matcher

object Editor {

    class SetText(
        private val text: String,
    ) : ViewAction {
        override fun getConstraints(): Matcher<View> = isDisplayed()

        override fun getDescription(): String = "Set text to $text"

        override fun perform(uiController: UiController?, view: View?) {
            val editor = view as? EditorEditText ?: return
            editor.setText(text)
        }
    }

    class SetHtml(
        private val html: String,
    ) : ViewAction {
        override fun getConstraints(): Matcher<View> = isDisplayed()

        override fun getDescription(): String = "Set html to $html"

        override fun perform(uiController: UiController?, view: View?) {
            val editor = view as? EditorEditText ?: return
            editor.setHtml(html)
        }
    }

    class SetLink(
        private val url: String,
    ) : ViewAction {
        override fun getConstraints(): Matcher<View> = isDisplayed()

        override fun getDescription(): String = "Set link to $url"

        override fun perform(uiController: UiController?, view: View?) {
            val editor = view as? EditorEditText ?: return
            editor.setLink(url)
        }
    }

    object RemoveLink : ViewAction {
        override fun getConstraints(): Matcher<View> = isDisplayed()

        override fun getDescription(): String = "Remove link"

        override fun perform(uiController: UiController?, view: View?) {
            val editor = view as? EditorEditText ?: return
            editor.removeLink()
        }
    }

    data class InsertLink(
        val text: String,
        val url: String,
    ) : ViewAction {
        override fun getConstraints(): Matcher<View> = isDisplayed()

        override fun getDescription(): String = "Insert text ($text) linking to $url"

        override fun perform(uiController: UiController?, view: View?) {
            val editor = view as? EditorEditText ?: return
            editor.insertLink(url = url, text = text)
        }
    }

    class SetLinkSuggestion(
        private val name: String,
        private val link: String,
    ) : ViewAction {
        override fun getConstraints(): Matcher<View> = isDisplayed()

        override fun getDescription(): String = "Set link at suggestion to $name, $link"

        override fun perform(uiController: UiController?, view: View?) {
            val editor = view as? EditorEditText ?: return
            editor.setLinkSuggestion(name = name,link = link)
        }
    }

    class SetLinkDisplayHandler(
        private val linkDisplayHandler: LinkDisplayHandler,
    ) : ViewAction {
        override fun getConstraints(): Matcher<View> = isDisplayed()

        override fun getDescription(): String = "Set link display handler"

        override fun perform(uiController: UiController?, view: View?) {
            val editor = view as? EditorEditText ?: return
            editor.linkDisplayHandler = linkDisplayHandler
        }
    }

    class ToggleList(
        private val ordered: Boolean,
    ) : ViewAction {
        override fun getConstraints(): Matcher<View> = isDisplayed()

        override fun getDescription(): String {
            val type = if (ordered) "ordered" else "unordered"
            return "Create $type list"
        }

        override fun perform(uiController: UiController?, view: View?) {
            val editor = view as? EditorEditText ?: return
            editor.toggleList(ordered)
        }
    }

    object Undo : ViewAction {
        override fun getConstraints(): Matcher<View> = isDisplayed()

        override fun getDescription(): String = "Performs undo action"

        override fun perform(uiController: UiController?, view: View?) {
            val editor = view as? EditorEditText ?: return
            editor.undo()
        }
    }

    object Redo : ViewAction {
        override fun getConstraints(): Matcher<View> = isDisplayed()

        override fun getDescription(): String = "Performs undo action"

        override fun perform(uiController: UiController?, view: View?) {
            val editor = view as? EditorEditText ?: return
            editor.redo()
        }
    }

    class ToggleFormat(
        private val format: InlineFormat
    ) : ViewAction {
        override fun getConstraints(): Matcher<View> = isDisplayed()

        override fun getDescription(): String = "Toggle format to $format"

        override fun perform(uiController: UiController?, view: View?) {
            val editor = view as? EditorEditText ?: return
            editor.toggleInlineFormat(format)
        }

    }

    class AddTextWatcher(
        private val textWatcher: (Editable?) -> Unit,
    ) : ViewAction {
        override fun getConstraints(): Matcher<View> = isDisplayed()

        override fun getDescription(): String = "Add a text watcher"

        override fun perform(uiController: UiController?, view: View?) {
            val editor = view as? EditorEditText ?: return
            editor.addTextChangedListener(
                onTextChanged = {_,_,_,_ -> },
                beforeTextChanged = {_,_,_,_ -> },
                afterTextChanged = textWatcher,
            )
        }
    }

    class TestCrash(
        private val errorCollector: RustErrorCollector?
    ) : ViewAction {
        override fun getConstraints(): Matcher<View> = isDisplayed()

        override fun getDescription(): String = "Test Rust crash"

        override fun perform(uiController: UiController?, view: View?) {
            val editor = view as? EditorEditText ?: return
            if(errorCollector != null) {
                editor.rustErrorCollector = errorCollector
            }
            editor.testComposerCrashRecovery()
        }
    }
}

object EditorActions {
    fun setText(text: String) = Editor.SetText(text)
    fun setHtml(html: String) = Editor.SetHtml(html)
    fun setLink(url: String) = Editor.SetLink(url)
    fun insertLink(text: String, url: String) = Editor.InsertLink(text, url)
    fun removeLink() = Editor.RemoveLink
    fun setLinkSuggestion(name: String, link: String) = Editor.SetLinkSuggestion(name, link)
    fun setLinkDisplayHandler(linkDisplayHandler: LinkDisplayHandler) = Editor.SetLinkDisplayHandler(linkDisplayHandler)
    fun toggleList(ordered: Boolean) = Editor.ToggleList(ordered)
    fun undo() = Editor.Undo
    fun redo() = Editor.Redo
    fun toggleFormat(format: InlineFormat) = Editor.ToggleFormat(format)
    fun addTextWatcher(watcher : (Editable?) -> Unit) = Editor.AddTextWatcher(watcher)
    fun testCrash(
        errorCollector: RustErrorCollector? = null
    ) = Editor.TestCrash(errorCollector)
}

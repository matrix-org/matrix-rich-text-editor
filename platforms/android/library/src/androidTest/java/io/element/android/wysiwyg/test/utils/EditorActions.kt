package io.element.android.wysiwyg.test.utils

import android.net.Uri
import android.text.Editable
import android.view.View
import androidx.core.view.ViewCompat
import androidx.core.widget.addTextChangedListener
import androidx.test.espresso.UiController
import androidx.test.espresso.ViewAction
import androidx.test.espresso.matcher.ViewMatchers.isDisplayed
import io.element.android.wysiwyg.EditorEditText
import io.element.android.wysiwyg.display.KeywordDisplayHandler
import io.element.android.wysiwyg.view.models.InlineFormat
import io.element.android.wysiwyg.display.LinkDisplayHandler
import io.element.android.wysiwyg.inputhandlers.UriContentListener
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
        private val text: String,
        private val url: String,
    ) : ViewAction {
        override fun getConstraints(): Matcher<View> = isDisplayed()

        override fun getDescription(): String = "Set link at suggestion to $text, $url"

        override fun perform(uiController: UiController?, view: View?) {
            val editor = view as? EditorEditText ?: return
            editor.setLinkSuggestion(url = url, text = text)
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

    class SetKeywordDisplayHandler(
        private val keywordDisplayHandler: KeywordDisplayHandler,
    ) : ViewAction {
        override fun getConstraints(): Matcher<View> = isDisplayed()

        override fun getDescription(): String = "Set keyword display handler"

        override fun perform(uiController: UiController?, view: View?) {
            val editor = view as? EditorEditText ?: return
            editor.keywordDisplayHandler = keywordDisplayHandler
        }
    }

    class ReplaceTextSuggestion(
        private val text: String,
    ) : ViewAction {
        override fun getConstraints(): Matcher<View> = isDisplayed()

        override fun getDescription(): String = "Set text at suggestion to $text"

        override fun perform(uiController: UiController?, view: View?) {
            val editor = view as? EditorEditText ?: return
            editor.replaceTextSuggestion(text = text)
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

    class AddContentWatcher(
        private val contentTypes: Array<String>,
        private val contentWatcher: (Uri) -> Unit,
    ) : ViewAction {
        override fun getConstraints(): Matcher<View> = isDisplayed()

        override fun getDescription(): String = "Add a content watcher"

        override fun perform(uiController: UiController?, view: View?) {
            val editor = view as? EditorEditText ?: return

            ViewCompat.setOnReceiveContentListener(
                editor,
                contentTypes,
                UriContentListener{
                    contentWatcher(it)
                }
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
    fun setLinkSuggestion(text: String, url: String) = Editor.SetLinkSuggestion(text, url)
    fun setLinkDisplayHandler(linkDisplayHandler: LinkDisplayHandler) = Editor.SetLinkDisplayHandler(linkDisplayHandler)
    fun replaceTextSuggestion(text: String) = Editor.ReplaceTextSuggestion(text)
    fun setKeywordDisplayHandler(keywordDisplayHandler: KeywordDisplayHandler) = Editor.SetKeywordDisplayHandler(keywordDisplayHandler)
    fun toggleList(ordered: Boolean) = Editor.ToggleList(ordered)
    fun undo() = Editor.Undo
    fun redo() = Editor.Redo
    fun toggleFormat(format: InlineFormat) = Editor.ToggleFormat(format)
    fun addTextWatcher(watcher: (Editable?) -> Unit) = Editor.AddTextWatcher(watcher)
    fun addContentWatcher(contentTypes: Array<String>, watcher: (Uri) -> Unit) = Editor.AddContentWatcher(contentTypes, watcher)
    fun testCrash(
        errorCollector: RustErrorCollector? = null
    ) = Editor.TestCrash(errorCollector)
}

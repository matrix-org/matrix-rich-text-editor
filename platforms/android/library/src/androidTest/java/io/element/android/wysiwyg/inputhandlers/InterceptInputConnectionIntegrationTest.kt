package io.element.android.wysiwyg.inputhandlers

import android.app.Application
import android.view.inputmethod.EditorInfo
import android.widget.EditText
import androidx.test.core.app.ApplicationProvider
import io.element.android.wysiwyg.inputhandlers.models.EditorInputAction
import io.element.android.wysiwyg.inputhandlers.models.InlineFormat
import io.element.android.wysiwyg.test.utils.dumpSpans
import io.element.android.wysiwyg.utils.AndroidHtmlConverter
import io.element.android.wysiwyg.utils.AndroidResourcesProvider
import io.element.android.wysiwyg.utils.HtmlToSpansParser
import io.element.android.wysiwyg.viewmodel.EditorViewModel
import org.hamcrest.MatcherAssert.assertThat
import org.hamcrest.Matchers.equalTo
import org.junit.Test
import uniffi.wysiwyg_composer.newComposerModel

class InterceptInputConnectionIntegrationTest {

    private val app: Application = ApplicationProvider.getApplicationContext()
    private val viewModel = EditorViewModel(
        composer = newComposerModel(),
        htmlConverter = AndroidHtmlConverter(
            provideHtmlToSpansParser = { html ->
                HtmlToSpansParser(
                    AndroidResourcesProvider(app),
                    html
                )
            },
        )
    )
    private val textView = EditText(ApplicationProvider.getApplicationContext())
    private val inputConnection = InterceptInputConnection(
        viewModel = viewModel,
        editorEditText = textView,
        baseInputConnection = textView.onCreateInputConnection(EditorInfo()),
    )

    private val baseEditedSpans = listOf(
        "world: android.widget.TextView.ChangeWatcher (0-5) fl=#6553618",
        "world: android.text.method.TextKeyListener (0-5) fl=#18",
        "world: android.widget.Editor.SpanController (0-5) fl=#18",
        ": android.text.Selection.START (5-5) fl=#546",
        ": android.text.Selection.END (5-5) fl=#34",
    )

    @Test
    fun testComposeBoldText() {
        viewModel.processInput(EditorInputAction.ApplyInlineFormat(InlineFormat.Bold))
        inputConnection.setComposingText("hello", 1)

        assertThat(textView.text.toString(), equalTo("hello"))
        assertThat(
            textView.text.dumpSpans(), equalTo(
                listOf(
                    "hello: android.widget.TextView.ChangeWatcher (0-5) fl=#6553618",
                    "hello: android.text.method.TextKeyListener (0-5) fl=#18",
                    "hello: android.widget.Editor.SpanController (0-5) fl=#18",
                    ": android.text.Selection.START (5-5) fl=#546",
                    ": android.text.Selection.END (5-5) fl=#34",
                    "hello: android.text.style.StyleSpan (0-5) fl=#33",
                    "hello: android.text.style.UnderlineSpan (0-5) fl=#289",
                    "hello: android.view.inputmethod.ComposingText (0-5) fl=#289",
                )
            )
        )
    }

    @Test
    fun testEditStyledText() {
        viewModel.processInput(EditorInputAction.ApplyInlineFormat(InlineFormat.Bold))
        inputConnection.setComposingText("hello", 1)
        assertThat(textView.text.toString(), equalTo("hello"))
        inputConnection.setComposingText("world", 1)
        inputConnection.commitText("world", 1)

        assertThat(
            textView.text.dumpSpans(), equalTo(
                baseEditedSpans + listOf(
                    "world: android.text.style.StyleSpan (0-5) fl=#33",
                )
            )
        )
    }

    @Test
    fun testEditUnderlinedText() {
        viewModel.processInput(EditorInputAction.ApplyInlineFormat(InlineFormat.Underline))
        inputConnection.setComposingText("hello", 1)
        assertThat(textView.text.toString(), equalTo("hello"))
        inputConnection.setComposingText("world", 1)
        inputConnection.commitText("world", 1)

        assertThat(
            textView.text.dumpSpans(), equalTo(
                baseEditedSpans + listOf(
                    "world: android.text.style.UnderlineSpan (0-5) fl=#33",
                )
            )
        )
    }

    @Test
    fun testEditStrikeThroughText() {
        viewModel.processInput(EditorInputAction.ApplyInlineFormat(InlineFormat.StrikeThrough))
        inputConnection.setComposingText("hello", 1)
        assertThat(textView.text.toString(), equalTo("hello"))
        inputConnection.setComposingText("world", 1)
        inputConnection.commitText("world", 1)

        assertThat(
            textView.text.dumpSpans(), equalTo(
                baseEditedSpans + listOf(
                    "world: android.text.style.StrikethroughSpan (0-5) fl=#33",
                )
            )
        )
    }

    @Test
    fun testEditInlineCodeText() {
        viewModel.processInput(EditorInputAction.ApplyInlineFormat(InlineFormat.InlineCode))
        inputConnection.setComposingText("hello", 1)
        assertThat(textView.text.toString(), equalTo("hello"))
        inputConnection.setComposingText("world", 1)
        inputConnection.commitText("world", 1)

        assertThat(
            textView.text.dumpSpans(), equalTo(
                baseEditedSpans + listOf(
                    "world: io.element.android.wysiwyg.spans.InlineCodeSpan (0-5) fl=#33",
                )
            )
        )
    }

    @Test
    fun testComposeOrderedListByWholeWord() {
        viewModel.processInput(EditorInputAction.ToggleList(ordered = true))
        inputConnection.setComposingText("hello", 1)

        assertThat(textView.text.toString(), equalTo("\u200Bhello"))
        assertThat(
            textView.text.dumpSpans(), equalTo(
                listOf(
                    "\u200Bhello: android.widget.TextView.ChangeWatcher (0-6) fl=#6553618",
                    "\u200Bhello: android.text.method.TextKeyListener (0-6) fl=#18",
                    "\u200Bhello: android.widget.Editor.SpanController (0-6) fl=#18",
                    ": android.text.Selection.START (6-6) fl=#546",
                    ": android.text.Selection.END (6-6) fl=#34",
                    "\u200B: io.element.android.wysiwyg.spans.ExtraCharacterSpan (0-1) fl=#33",
                    "\u200Bhello: io.element.android.wysiwyg.spans.OrderedListSpan (0-6) fl=#33",
                    "hello: android.text.style.UnderlineSpan (1-6) fl=#289",
                    "hello: android.view.inputmethod.ComposingText (1-6) fl=#289",
                )
            )
        )
    }

    @Test
    fun testComposeUnorderedListLetterByLetter() {
        viewModel.processInput(EditorInputAction.ToggleList(ordered = false))
        inputConnection.setComposingText("h", 1)
        inputConnection.setComposingText("he", 1)
        inputConnection.setComposingText("hel", 1)
        inputConnection.setComposingText("hell", 1)
        inputConnection.setComposingText("hello", 1)

        assertThat(textView.text.toString(), equalTo("\u200Bhello"))
        assertThat(
            textView.text.dumpSpans(), equalTo(
                listOf(
                    "\u200Bhello: android.widget.TextView.ChangeWatcher (0-6) fl=#6553618",
                    "\u200Bhello: android.text.method.TextKeyListener (0-6) fl=#18",
                    "\u200Bhello: android.widget.Editor.SpanController (0-6) fl=#18",
                    ": android.text.Selection.START (6-6) fl=#546",
                    ": android.text.Selection.END (6-6) fl=#34",
                    "\u200B: io.element.android.wysiwyg.spans.ExtraCharacterSpan (0-1) fl=#33",
                    "\u200Bhello: android.text.style.BulletSpan (0-6) fl=#33",
                    "hello: android.text.style.UnderlineSpan (1-6) fl=#289",
                    "hello: android.view.inputmethod.ComposingText (1-6) fl=#289",
                )
            )
        )
    }

    @Test
    fun testComposeUnorderedListByWholeWord() {
        viewModel.processInput(EditorInputAction.ToggleList(ordered = false))
        inputConnection.setComposingText("hello", 1)

        assertThat(textView.text.toString(), equalTo("\u200Bhello"))
        assertThat(
            textView.text.dumpSpans(), equalTo(
                listOf(
                    "\u200Bhello: android.widget.TextView.ChangeWatcher (0-6) fl=#6553618",
                    "\u200Bhello: android.text.method.TextKeyListener (0-6) fl=#18",
                    "\u200Bhello: android.widget.Editor.SpanController (0-6) fl=#18",
                    ": android.text.Selection.START (6-6) fl=#546",
                    ": android.text.Selection.END (6-6) fl=#34",
                    "\u200B: io.element.android.wysiwyg.spans.ExtraCharacterSpan (0-1) fl=#33",
                    "\u200Bhello: android.text.style.BulletSpan (0-6) fl=#33",
                    "hello: android.text.style.UnderlineSpan (1-6) fl=#289",
                    "hello: android.view.inputmethod.ComposingText (1-6) fl=#289",
                )
            )
        )
    }

    @Test
    fun testComposeOrderedListLetterByLetter() {
        viewModel.processInput(EditorInputAction.ToggleList(ordered = true))
        inputConnection.setComposingText("h", 1)
        inputConnection.setComposingText("he", 1)
        inputConnection.setComposingText("hel", 1)
        inputConnection.setComposingText("hell", 1)
        inputConnection.setComposingText("hello", 1)

        assertThat(textView.text.toString(), equalTo("\u200Bhello"))
        assertThat(
            textView.text.dumpSpans(), equalTo(
                listOf(
                    "\u200Bhello: android.widget.TextView.ChangeWatcher (0-6) fl=#6553618",
                    "\u200Bhello: android.text.method.TextKeyListener (0-6) fl=#18",
                    "\u200Bhello: android.widget.Editor.SpanController (0-6) fl=#18",
                    ": android.text.Selection.START (6-6) fl=#546",
                    ": android.text.Selection.END (6-6) fl=#34",
                    "\u200B: io.element.android.wysiwyg.spans.ExtraCharacterSpan (0-1) fl=#33",
                    "\u200Bhello: io.element.android.wysiwyg.spans.OrderedListSpan (0-6) fl=#33",
                    "hello: android.text.style.UnderlineSpan (1-6) fl=#289",
                    "hello: android.view.inputmethod.ComposingText (1-6) fl=#289",
                )
            )
        )
    }

    @Test
    fun testComposeOrderedListLetterWithEmoji() {
        viewModel.processInput(EditorInputAction.ToggleList(ordered = true))
        inputConnection.setComposingText("😋", 1)
        inputConnection.setComposingText("😋😋", 1)

        assertThat(textView.text.toString(), equalTo("\u200B😋😋"))
        assertThat(
            textView.text.dumpSpans(), equalTo(
                listOf(
                    "\u200B😋😋: android.widget.TextView.ChangeWatcher (0-5) fl=#6553618",
                    "\u200B😋😋: android.text.method.TextKeyListener (0-5) fl=#18",
                    "\u200B😋😋: android.widget.Editor.SpanController (0-5) fl=#18",
                    ": android.text.Selection.START (5-5) fl=#546",
                    ": android.text.Selection.END (5-5) fl=#34",
                    "\u200B: io.element.android.wysiwyg.spans.ExtraCharacterSpan (0-1) fl=#33",
                    "\u200B😋😋: io.element.android.wysiwyg.spans.OrderedListSpan (0-5) fl=#33",
                    "😋😋: android.text.style.UnderlineSpan (1-5) fl=#289",
                    "😋😋: android.view.inputmethod.ComposingText (1-5) fl=#289",
                )
            )
        )
    }

    @Test
    fun testIncrementalCommitTextRespectsFormatting() {
        // Set initial text
        val initialText = viewModel.processInput(
            EditorInputAction.ReplaceAllHtml("<strong>test</strong>")
        )?.text
        textView.setText(initialText)
        // Disable bold at end of string
        textView.setSelection(4)
        viewModel.processInput(EditorInputAction.ApplyInlineFormat(InlineFormat.Bold))
        // Autocomplete 'test' -> 'testing'
        inputConnection.setComposingRegion(0, 4)
        inputConnection.commitText("testing", 1)

        assertThat(viewModel.getHtml(), equalTo("<strong>test</strong>ing"))
    }

    @Test
    fun testIncrementalCommitWithFormattingDisabledKeepsItDisabledWithWhitespace() {
        // Set initial text
        val initialText = viewModel.processInput(
            EditorInputAction.ReplaceAllHtml("<strong>test</strong>")
        )?.text
        textView.setText(initialText)
        // Disable bold at end of string
        textView.setSelection(4)
        viewModel.processInput(EditorInputAction.ApplyInlineFormat(InlineFormat.Bold))
        // Autocomplete 'test' -> 'testing'
        inputConnection.setComposingRegion(0, 4)
        inputConnection.commitText("test ", 1)
        // Add some extra text
        inputConnection.setComposingText("whitespaces", 1)

        assertThat(viewModel.getHtml(), equalTo("<strong>test</strong> whitespaces"))
    }
}

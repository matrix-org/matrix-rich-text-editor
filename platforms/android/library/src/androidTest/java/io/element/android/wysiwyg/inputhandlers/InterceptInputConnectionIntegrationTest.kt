package io.element.android.wysiwyg.inputhandlers

import android.app.Application
import android.view.inputmethod.EditorInfo
import android.widget.EditText
import androidx.test.core.app.ApplicationProvider
import io.element.android.wysiwyg.fakes.createFakeStyleConfig
import io.element.android.wysiwyg.internal.viewmodel.EditorInputAction
import io.element.android.wysiwyg.internal.viewmodel.EditorViewModel
import io.element.android.wysiwyg.test.utils.dumpSpans
import io.element.android.wysiwyg.utils.HtmlConverter
import io.element.android.wysiwyg.utils.NBSP
import io.element.android.wysiwyg.view.models.InlineFormat
import org.hamcrest.MatcherAssert.assertThat
import org.hamcrest.Matchers.equalTo
import org.junit.Test
import uniffi.wysiwyg_composer.newComposerModel
import uniffi.wysiwyg_composer.newMentionDetector

class InterceptInputConnectionIntegrationTest {

    private val app: Application = ApplicationProvider.getApplicationContext()
    private val styleConfig = createFakeStyleConfig()
    private val viewModel = EditorViewModel(
        provideComposer = { newComposerModel() },
        provideMentionDetector = { newMentionDetector() },
    ).also {
        it.htmlConverter = HtmlConverter.Factory.create(context = app,
            styleConfig = styleConfig,
            mentionDisplayHandler = null,
        )
    }
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
        simulateInput(EditorInputAction.ApplyInlineFormat(InlineFormat.Bold))
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
        simulateInput(EditorInputAction.ApplyInlineFormat(InlineFormat.Bold))
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
        simulateInput(EditorInputAction.ApplyInlineFormat(InlineFormat.Underline))
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
                    "world: io.element.android.wysiwyg.view.spans.InlineCodeSpan (0-5) fl=#33",
                )
            )
        )
    }

    @Test
    fun testComposeOrderedListByWholeWord() {
        simulateInput(EditorInputAction.ToggleList(ordered = true))
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
                    "hello: io.element.android.wysiwyg.view.spans.OrderedListSpan (0-5) fl=#34",
                    "hello: android.text.style.UnderlineSpan (0-5) fl=#289",
                    "hello: android.view.inputmethod.ComposingText (0-5) fl=#289",
                )
            )
        )
    }

    @Test
    fun testComposeUnorderedListLetterByLetter() {
        simulateInput(EditorInputAction.ToggleList(ordered = false))
        inputConnection.setComposingText("h", 1)
        inputConnection.setComposingText("he", 1)
        inputConnection.setComposingText("hel", 1)
        inputConnection.setComposingText("hell", 1)
        inputConnection.setComposingText("hello", 1)

        assertThat(textView.text.toString(), equalTo("hello"))
        assertThat(
            textView.text.dumpSpans().joinToString(",\n"), equalTo(
                """
                    hello: android.widget.TextView.ChangeWatcher (0-5) fl=#6553618,
                    hello: android.text.method.TextKeyListener (0-5) fl=#18,
                    hello: android.widget.Editor.SpanController (0-5) fl=#18,
                    : android.text.Selection.START (5-5) fl=#546,
                    : android.text.Selection.END (5-5) fl=#34,
                    hello: io.element.android.wysiwyg.view.spans.UnorderedListSpan (0-5) fl=#34,
                    hello: android.text.style.UnderlineSpan (0-5) fl=#289,
                    hello: android.view.inputmethod.ComposingText (0-5) fl=#289
                """.trimIndent()
            )
        )
    }

    @Test
    fun testComposeUnorderedListByWholeWord() {
        simulateInput(EditorInputAction.ToggleList(ordered = false))
        inputConnection.setComposingText("hello", 1)

        assertThat(textView.text.toString(), equalTo("hello"))
        assertThat(
            textView.text.dumpSpans().joinToString(",\n"), equalTo(
                """
                    hello: android.widget.TextView.ChangeWatcher (0-5) fl=#6553618,
                    hello: android.text.method.TextKeyListener (0-5) fl=#18,
                    hello: android.widget.Editor.SpanController (0-5) fl=#18,
                    : android.text.Selection.START (5-5) fl=#546,
                    : android.text.Selection.END (5-5) fl=#34,
                    hello: io.element.android.wysiwyg.view.spans.UnorderedListSpan (0-5) fl=#34,
                    hello: android.text.style.UnderlineSpan (0-5) fl=#289,
                    hello: android.view.inputmethod.ComposingText (0-5) fl=#289
                """.trimIndent()
            )
        )
    }

    @Test
    fun testComposeOrderedListLetterByLetter() {
        simulateInput(EditorInputAction.ToggleList(ordered = true))
        inputConnection.setComposingText("h", 1)
        inputConnection.setComposingText("he", 1)
        inputConnection.setComposingText("hel", 1)
        inputConnection.setComposingText("hell", 1)
        inputConnection.setComposingText("hello", 1)

        assertThat(textView.text.toString(), equalTo("hello"))
        assertThat(
            textView.text.dumpSpans().joinToString(",\n"), equalTo(
                """
                    hello: android.widget.TextView.ChangeWatcher (0-5) fl=#6553618,
                    hello: android.text.method.TextKeyListener (0-5) fl=#18,
                    hello: android.widget.Editor.SpanController (0-5) fl=#18,
                    : android.text.Selection.START (5-5) fl=#546,
                    : android.text.Selection.END (5-5) fl=#34,
                    hello: io.element.android.wysiwyg.view.spans.OrderedListSpan (0-5) fl=#34,
                    hello: android.text.style.UnderlineSpan (0-5) fl=#289,
                    hello: android.view.inputmethod.ComposingText (0-5) fl=#289
                """.trimIndent()
            )
        )
    }

    @Test
    fun testComposeOrderedListLetterWithEmoji() {
        simulateInput(EditorInputAction.ToggleList(ordered = true))
        inputConnection.setComposingText("😋", 1)
        inputConnection.setComposingText("😋😋", 1)

        assertThat(textView.text.toString(), equalTo("😋😋"))
        assertThat(
            textView.text.dumpSpans(), equalTo(
                listOf(
                    "😋😋: android.widget.TextView.ChangeWatcher (0-4) fl=#6553618",
                    "😋😋: android.text.method.TextKeyListener (0-4) fl=#18",
                    "😋😋: android.widget.Editor.SpanController (0-4) fl=#18",
                    ": android.text.Selection.START (4-4) fl=#546",
                    ": android.text.Selection.END (4-4) fl=#34",
                    "😋😋: io.element.android.wysiwyg.view.spans.OrderedListSpan (0-4) fl=#34",
                    "😋😋: android.text.style.UnderlineSpan (0-4) fl=#289",
                    "😋😋: android.view.inputmethod.ComposingText (0-4) fl=#289",
                )
            )
        )
    }

    @Test
    fun testComposeCodeBlockLetterByLetter() {
        simulateInput(EditorInputAction.CodeBlock)
        inputConnection.setComposingText("h", 1)
        inputConnection.setComposingText("he", 1)
        inputConnection.setComposingText("hel", 1)
        inputConnection.setComposingText("hell", 1)
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
                    "hello: io.element.android.wysiwyg.view.spans.CodeBlockSpan (0-5) fl=#33",
                    "hello: android.text.style.UnderlineSpan (0-5) fl=#289",
                    "hello: android.view.inputmethod.ComposingText (0-5) fl=#289"
                )
            )
        )
    }

    @Test
    fun testEnterInCodeBlockAtStart() {
        simulateInput(EditorInputAction.CodeBlock)

        // Should replace the code block span with an empty paragraph
        inputConnection.onHardwareEnterKey()

        assertThat(textView.text.toString(), equalTo("$NBSP"))
        assertThat(
            textView.text.dumpSpans(), equalTo(
                listOf(
                    "$NBSP: android.widget.TextView.ChangeWatcher (0-1) fl=#6553618",
                    "$NBSP: android.text.method.TextKeyListener (0-1) fl=#18",
                    "$NBSP: android.widget.Editor.SpanController (0-1) fl=#18",
                    ": android.text.Selection.START (0-0) fl=#546",
                    ": android.text.Selection.END (0-0) fl=#34",
                    "$NBSP: io.element.android.wysiwyg.view.spans.ExtraCharacterSpan (0-1) fl=#17"
                )
            )
        )
    }

    @Test
    fun testDoubleEnterInCodeBlockAtMiddle() {
        simulateInput(EditorInputAction.CodeBlock)
        inputConnection.setComposingText("Test", 1)

        // First line break, should just add a line break character
        inputConnection.onHardwareEnterKey()

        assertThat(textView.text.toString(), equalTo("Test\n$NBSP"))
        assertThat(
            textView.text.dumpSpans(), equalTo(
                listOf(
                    "Test\n$NBSP: android.widget.TextView.ChangeWatcher (0-6) fl=#6553618",
                    "Test\n$NBSP: android.text.method.TextKeyListener (0-6) fl=#18",
                    "Test\n$NBSP: android.widget.Editor.SpanController (0-6) fl=#18",
                    ": android.text.Selection.START (5-5) fl=#546",
                    ": android.text.Selection.END (5-5) fl=#34",
                    "Test\n$NBSP: io.element.android.wysiwyg.view.spans.CodeBlockSpan (0-6) fl=#33",
                    "$NBSP: io.element.android.wysiwyg.view.spans.ExtraCharacterSpan (5-6) fl=#17",
                )
            )
        )

        // Second one, should create a new line break outside the code block
        inputConnection.onHardwareEnterKey()

        assertThat(textView.text.toString(), equalTo("Test\n$NBSP"))
        assertThat(
            textView.text.dumpSpans(), equalTo(
                listOf(
                    "Test\n$NBSP: android.widget.TextView.ChangeWatcher (0-6) fl=#6553618",
                    "Test\n$NBSP: android.text.method.TextKeyListener (0-6) fl=#18",
                    "Test\n$NBSP: android.widget.Editor.SpanController (0-6) fl=#18",
                    ": android.text.Selection.START (5-5) fl=#34",
                    ": android.text.Selection.END (5-5) fl=#34",
                    "Test: io.element.android.wysiwyg.view.spans.CodeBlockSpan (0-4) fl=#33",
                    "$NBSP: io.element.android.wysiwyg.view.spans.ExtraCharacterSpan (5-6) fl=#17",
                )
            )
        )
    }

    @Test
    fun testComposeQuoteLetterByLetter() {
        simulateInput(EditorInputAction.Quote)
        inputConnection.setComposingText("h", 1)
        inputConnection.setComposingText("he", 1)
        inputConnection.setComposingText("hel", 1)
        inputConnection.setComposingText("hell", 1)
        inputConnection.setComposingText("hello", 1)

        assertThat(textView.text.toString(), equalTo("hello"))
        assertThat(
            textView.text.dumpSpans().joinToString(",\n"), equalTo(
                """
                    hello: android.widget.TextView.ChangeWatcher (0-5) fl=#6553618,
                    hello: android.text.method.TextKeyListener (0-5) fl=#18,
                    hello: android.widget.Editor.SpanController (0-5) fl=#18,
                    : android.text.Selection.START (5-5) fl=#546,
                    : android.text.Selection.END (5-5) fl=#34,
                    hello: io.element.android.wysiwyg.view.spans.QuoteSpan (0-5) fl=#33,
                    hello: android.text.style.UnderlineSpan (0-5) fl=#289,
                    hello: android.view.inputmethod.ComposingText (0-5) fl=#289
                """.trimIndent()
            )
        )
    }

    @Test
    fun testEnterInQuoteAtStart() {
        simulateInput(EditorInputAction.Quote)

        // Should replace the quote span with an empty paragraph
        inputConnection.onHardwareEnterKey()

        assertThat(textView.text.toString(), equalTo("$NBSP"))
        assertThat(
            textView.text.dumpSpans(), equalTo(
                listOf(
                    "$NBSP: android.widget.TextView.ChangeWatcher (0-1) fl=#6553618",
                    "$NBSP: android.text.method.TextKeyListener (0-1) fl=#18",
                    "$NBSP: android.widget.Editor.SpanController (0-1) fl=#18",
                    ": android.text.Selection.START (0-0) fl=#546",
                    ": android.text.Selection.END (0-0) fl=#34",
                    "$NBSP: io.element.android.wysiwyg.view.spans.ExtraCharacterSpan (0-1) fl=#17"
                )
            )
        )
    }

    @Test
    fun testDoubleEnterInQuoteAtMiddle() {
        simulateInput(EditorInputAction.Quote)
        inputConnection.setComposingText("Test", 1)

        // First line break, should just add a line break character
        inputConnection.onHardwareEnterKey()

        assertThat(textView.text.toString(), equalTo("Test\n$NBSP"))
        assertThat(
            textView.text.dumpSpans(), equalTo(
                listOf(
                    "Test\n$NBSP: android.widget.TextView.ChangeWatcher (0-6) fl=#6553618",
                    "Test\n$NBSP: android.text.method.TextKeyListener (0-6) fl=#18",
                    "Test\n$NBSP: android.widget.Editor.SpanController (0-6) fl=#18",
                    ": android.text.Selection.START (5-5) fl=#546",
                    ": android.text.Selection.END (5-5) fl=#34",
                    "Test\n$NBSP: io.element.android.wysiwyg.view.spans.QuoteSpan (0-6) fl=#33",
                    "$NBSP: io.element.android.wysiwyg.view.spans.ExtraCharacterSpan (5-6) fl=#17",
                )
            )
        )

        // Second one, should create a new line break outside the quote
        inputConnection.onHardwareEnterKey()

        assertThat(textView.text.toString(), equalTo("Test\n$NBSP"))
        assertThat(
            textView.text.dumpSpans(), equalTo(
                listOf(
                    "Test\n$NBSP: android.widget.TextView.ChangeWatcher (0-6) fl=#6553618",
                    "Test\n$NBSP: android.text.method.TextKeyListener (0-6) fl=#18",
                    "Test\n$NBSP: android.widget.Editor.SpanController (0-6) fl=#18",
                    ": android.text.Selection.START (5-5) fl=#34",
                    ": android.text.Selection.END (5-5) fl=#34",
                    "Test: io.element.android.wysiwyg.view.spans.QuoteSpan (0-4) fl=#33",
                    "$NBSP: io.element.android.wysiwyg.view.spans.ExtraCharacterSpan (5-6) fl=#17",
                )
            )
        )
    }

    @Test
    fun testEnterInEmptyModel() {
        inputConnection.onHardwareEnterKey()

        assertThat(textView.text.toString(), equalTo("$NBSP\n$NBSP"))

        assertThat(
            textView.text.dumpSpans(), equalTo(
                listOf(
                    "$NBSP\n$NBSP: android.widget.TextView.ChangeWatcher (0-3) fl=#6553618",
                    "$NBSP\n$NBSP: android.text.method.TextKeyListener (0-3) fl=#18",
                    "$NBSP\n$NBSP: android.widget.Editor.SpanController (0-3) fl=#18",
                    ": android.text.Selection.START (2-2) fl=#546",
                    ": android.text.Selection.END (2-2) fl=#34",
                    "$NBSP: io.element.android.wysiwyg.view.spans.ExtraCharacterSpan (0-1) fl=#17",
                    "$NBSP: io.element.android.wysiwyg.view.spans.ExtraCharacterSpan (2-3) fl=#17"
                )
            )
        )
    }

    @Test
    fun testEnterAtStartOfTextCreatesNewParagraphBefore() {
        simulateInput(EditorInputAction.ReplaceAllHtml("Initial text"))
        viewModel.updateSelection(textView.editableText, 0, 0)

        inputConnection.onHardwareEnterKey()

        assertThat(textView.text.toString(), equalTo("$NBSP\nInitial text"))
    }

    @Test
    fun testEnterAtEndOfTextCreatesNewParagraphAfter() {
        simulateInput(EditorInputAction.ReplaceAllHtml("Initial text"))

        inputConnection.onHardwareEnterKey()

        assertThat(textView.text.toString(), equalTo("Initial text\n$NBSP"))
    }

    @Test
    fun testEnterAtMiddleOfTextSplitsItIntoTwoParagraphs() {
        simulateInput(EditorInputAction.ReplaceAllHtml("Initial text"))
        viewModel.updateSelection(textView.editableText, 7, 8)

        inputConnection.onHardwareEnterKey()

        assertThat(textView.text.toString(), equalTo("Initial\ntext"))
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

        assertThat(viewModel.getContentAsMessageHtml(), equalTo("<strong>test</strong>ing"))
    }

    @Test
    fun testIncrementalCommitWithDisabledFormattingKeepsItDisabledAfterWhitespace() {
        // Set initial text
        simulateInput(
            EditorInputAction.ReplaceAllHtml("<strong>test</strong>")
        )
        simulateInput(EditorInputAction.ApplyInlineFormat(InlineFormat.Bold))
        // Autocomplete 'test' -> 'test '
        inputConnection.setComposingRegion(0, 4)
        inputConnection.commitText("test ", 1)
        // Add some extra text
        inputConnection.setComposingText("whitespaces", 1)

        assertThat(
            viewModel.getContentAsMessageHtml(), equalTo("<strong>test</strong> whitespaces")
        )
    }

    private fun simulateInput(editorInputAction: EditorInputAction) =
        viewModel.processInput(editorInputAction)?.let { (text, selection) ->
            textView.setText(text)
            textView.setSelection(selection.first, selection.last)
        }
}

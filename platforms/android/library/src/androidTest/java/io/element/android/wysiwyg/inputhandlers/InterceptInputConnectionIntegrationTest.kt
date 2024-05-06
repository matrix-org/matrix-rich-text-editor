package io.element.android.wysiwyg.inputhandlers

import android.app.Application
import android.view.KeyEvent
import android.view.inputmethod.EditorInfo
import android.widget.EditText
import androidx.test.core.app.ApplicationProvider
import io.element.android.wysiwyg.EditorTextWatcher
import io.element.android.wysiwyg.fakes.createFakeStyleConfig
import io.element.android.wysiwyg.internal.viewmodel.ComposerResult
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

class InterceptInputConnectionIntegrationTest {

    private val app: Application = ApplicationProvider.getApplicationContext()
    private val styleConfig = createFakeStyleConfig()
    private val viewModel = EditorViewModel(
        provideComposer = { newComposerModel() },
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
        textWatcher = EditorTextWatcher(),
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
                    "hello: android.text.style.StyleSpan (0-5) fl=#33",
                    "hello: android.text.method.TextKeyListener (0-5) fl=#18",
                    "hello: android.text.style.UnderlineSpan (0-5) fl=#289",
                    "hello: android.view.inputmethod.ComposingText (0-5) fl=#289",
                    "hello: android.widget.Editor.SpanController (0-5) fl=#18",
                    ": android.text.Selection.START (5-5) fl=#546",
                    ": android.text.Selection.END (5-5) fl=#34",
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
                baseEditedSpans.toMutableList().apply {
                    add(1, "world: android.text.style.StyleSpan (0-5) fl=#33")
                }
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
                baseEditedSpans.toMutableList().apply {
                    add(1, "world: android.text.style.UnderlineSpan (0-5) fl=#33")
                }
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
                baseEditedSpans.toMutableList().apply {
                    add(1, "world: android.text.style.StrikethroughSpan (0-5) fl=#33")
                }
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
                baseEditedSpans.toMutableList().apply {
                    add(1, "world: io.element.android.wysiwyg.view.spans.InlineCodeSpan (0-5) fl=#33")
                }
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
                    "hello: io.element.android.wysiwyg.view.spans.OrderedListSpan (0-5) fl=#34",
                    "hello: android.text.style.UnderlineSpan (0-5) fl=#289",
                    "hello: android.view.inputmethod.ComposingText (0-5) fl=#289",
                    "hello: android.text.method.TextKeyListener (0-5) fl=#18",
                    "hello: android.widget.Editor.SpanController (0-5) fl=#18",
                    ": android.text.Selection.START (5-5) fl=#546",
                    ": android.text.Selection.END (5-5) fl=#34",
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
                    hello: io.element.android.wysiwyg.view.spans.UnorderedListSpan (0-5) fl=#34,
                    hello: android.text.style.UnderlineSpan (0-5) fl=#289,
                    hello: android.view.inputmethod.ComposingText (0-5) fl=#289,
                    hello: android.text.method.TextKeyListener (0-5) fl=#18,
                    hello: android.widget.Editor.SpanController (0-5) fl=#18,
                    : android.text.Selection.START (5-5) fl=#546,
                    : android.text.Selection.END (5-5) fl=#34
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
                    hello: io.element.android.wysiwyg.view.spans.UnorderedListSpan (0-5) fl=#34,
                    hello: android.text.style.UnderlineSpan (0-5) fl=#289,
                    hello: android.view.inputmethod.ComposingText (0-5) fl=#289,
                    hello: android.text.method.TextKeyListener (0-5) fl=#18,
                    hello: android.widget.Editor.SpanController (0-5) fl=#18,
                    : android.text.Selection.START (5-5) fl=#546,
                    : android.text.Selection.END (5-5) fl=#34
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
                    hello: io.element.android.wysiwyg.view.spans.OrderedListSpan (0-5) fl=#34,
                    hello: android.text.style.UnderlineSpan (0-5) fl=#289,
                    hello: android.view.inputmethod.ComposingText (0-5) fl=#289,
                    hello: android.text.method.TextKeyListener (0-5) fl=#18,
                    hello: android.widget.Editor.SpanController (0-5) fl=#18,
                    : android.text.Selection.START (5-5) fl=#546,
                    : android.text.Selection.END (5-5) fl=#34
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
                    "😋😋: io.element.android.wysiwyg.view.spans.OrderedListSpan (0-4) fl=#34",
                    "😋😋: android.text.style.UnderlineSpan (0-4) fl=#289",
                    "😋😋: android.view.inputmethod.ComposingText (0-4) fl=#289",
                    "😋😋: android.text.method.TextKeyListener (0-4) fl=#18",
                    "😋😋: android.widget.Editor.SpanController (0-4) fl=#18",
                    ": android.text.Selection.START (4-4) fl=#546",
                    ": android.text.Selection.END (4-4) fl=#34",
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
                    "hello: io.element.android.wysiwyg.view.spans.CodeBlockSpan (0-5) fl=#33",
                    "hello: android.text.style.UnderlineSpan (0-5) fl=#289",
                    "hello: android.view.inputmethod.ComposingText (0-5) fl=#289",
                    "hello: android.text.method.TextKeyListener (0-5) fl=#18",
                    "hello: android.widget.Editor.SpanController (0-5) fl=#18",
                    ": android.text.Selection.START (5-5) fl=#546",
                    ": android.text.Selection.END (5-5) fl=#34"
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
                    "$NBSP: io.element.android.wysiwyg.view.spans.ExtraCharacterSpan (0-1) fl=#17",
                    "$NBSP: android.text.method.TextKeyListener (0-1) fl=#18",
                    "$NBSP: android.widget.Editor.SpanController (0-1) fl=#18",
                    ": android.text.Selection.START (0-0) fl=#546",
                    ": android.text.Selection.END (0-0) fl=#34"
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
                    "Test\n$NBSP: io.element.android.wysiwyg.view.spans.CodeBlockSpan (0-6) fl=#33",
                    "$NBSP: io.element.android.wysiwyg.view.spans.ExtraCharacterSpan (5-6) fl=#17",
                    "Test\n$NBSP: android.text.method.TextKeyListener (0-6) fl=#18",
                    "Test\n$NBSP: android.widget.Editor.SpanController (0-6) fl=#18",
                    ": android.text.Selection.START (5-5) fl=#546",
                    ": android.text.Selection.END (5-5) fl=#34",
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
                    "Test: io.element.android.wysiwyg.view.spans.CodeBlockSpan (0-4) fl=#33",
                    "$NBSP: io.element.android.wysiwyg.view.spans.ExtraCharacterSpan (5-6) fl=#17",
                    "Test\n$NBSP: android.text.method.TextKeyListener (0-6) fl=#18",
                    "Test\n$NBSP: android.widget.Editor.SpanController (0-6) fl=#18",
                    ": android.text.Selection.START (5-5) fl=#546",
                    ": android.text.Selection.END (5-5) fl=#34",
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
                    hello: io.element.android.wysiwyg.view.spans.QuoteSpan (0-5) fl=#33,
                    hello: android.text.style.UnderlineSpan (0-5) fl=#289,
                    hello: android.view.inputmethod.ComposingText (0-5) fl=#289,
                    hello: android.text.method.TextKeyListener (0-5) fl=#18,
                    hello: android.widget.Editor.SpanController (0-5) fl=#18,
                    : android.text.Selection.START (5-5) fl=#546,
                    : android.text.Selection.END (5-5) fl=#34
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
                    "$NBSP: io.element.android.wysiwyg.view.spans.ExtraCharacterSpan (0-1) fl=#17",
                    "$NBSP: android.text.method.TextKeyListener (0-1) fl=#18",
                    "$NBSP: android.widget.Editor.SpanController (0-1) fl=#18",
                    ": android.text.Selection.START (0-0) fl=#546",
                    ": android.text.Selection.END (0-0) fl=#34"
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
                    "Test\n$NBSP: io.element.android.wysiwyg.view.spans.QuoteSpan (0-6) fl=#33",
                    "$NBSP: io.element.android.wysiwyg.view.spans.ExtraCharacterSpan (5-6) fl=#17",
                    "Test\n$NBSP: android.text.method.TextKeyListener (0-6) fl=#18",
                    "Test\n$NBSP: android.widget.Editor.SpanController (0-6) fl=#18",
                    ": android.text.Selection.START (5-5) fl=#546",
                    ": android.text.Selection.END (5-5) fl=#34",
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
                    "Test: io.element.android.wysiwyg.view.spans.QuoteSpan (0-4) fl=#33",
                    "$NBSP: io.element.android.wysiwyg.view.spans.ExtraCharacterSpan (5-6) fl=#17",
                    "Test\n$NBSP: android.text.method.TextKeyListener (0-6) fl=#18",
                    "Test\n$NBSP: android.widget.Editor.SpanController (0-6) fl=#18",
                    ": android.text.Selection.START (5-5) fl=#546",
                    ": android.text.Selection.END (5-5) fl=#34",
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
                    "$NBSP: io.element.android.wysiwyg.view.spans.ExtraCharacterSpan (0-1) fl=#17",
                    "$NBSP\n$NBSP: android.text.method.TextKeyListener (0-3) fl=#18",
                    "$NBSP: io.element.android.wysiwyg.view.spans.ExtraCharacterSpan (2-3) fl=#17",
                    "$NBSP\n$NBSP: android.widget.Editor.SpanController (0-3) fl=#18",
                    ": android.text.Selection.START (2-2) fl=#546",
                    ": android.text.Selection.END (2-2) fl=#34"
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
        val initialText = (viewModel.processInput(
            EditorInputAction.ReplaceAllHtml("<strong>test</strong>")
        ) as? ComposerResult.ReplaceText)?.text
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

    @Test
    fun testAddingAndRemovingEmojiInMiddleOfComposition() {
        // Set initial text
        simulateInput(
            EditorInputAction.ReplaceAllHtml("Test")
        )
        textView.setSelection(2)

        // Insert emoji at index 2. This would normally cause a crash.
        inputConnection.commitText("\uD83D\uDE00", 1)
        assertThat(viewModel.getContentAsMessageHtml(), equalTo("Te\uD83D\uDE00st"))
        // Since an emoji of length 2 was added, the selection is now at index 4
        assertThat(textView.selectionStart, equalTo(4))
        assertThat(textView.selectionEnd, equalTo(4))

        // Remove the emoji. This would also cause some issues with some keyboards.
        inputConnection.deleteSurroundingText(1, 0)
        assertThat(viewModel.getContentAsMessageHtml(), equalTo("Test"))
        assertThat(textView.selectionStart, equalTo(2))
        assertThat(textView.selectionEnd, equalTo(2))
    }

    @Test
    fun testHalfWidthEnglishInChineseKeyboards() {
        inputConnection.run {
            sendHardwareKeyboardInput(KeyEvent(KeyEvent.ACTION_DOWN, KeyEvent.KEYCODE_T))
            setComposingText("", 1)
            sendHardwareKeyboardInput(KeyEvent(KeyEvent.ACTION_DOWN, KeyEvent.KEYCODE_E))
            setComposingText("", 1)
            sendHardwareKeyboardInput(KeyEvent(KeyEvent.ACTION_DOWN, KeyEvent.KEYCODE_S))
            setComposingText("", 1)
            sendHardwareKeyboardInput(KeyEvent(KeyEvent.ACTION_DOWN, KeyEvent.KEYCODE_T))
        }
        assertThat(textView.text.toString(), equalTo("test"))
    }

    @Test
    fun testPunctuationSymbolAfterCommittingText() {
        // This simulates the behaviour of Gboard on Android < 13 when adding a punctuation symbol
        // after a committed word.
        inputConnection.run {
            // Manually entered letter
            setComposingText("A", 1)
            // Committed text from suggestions
            commitText("Anything ", 1)
            // Assert selection is properly updated
            assertThat(textView.selectionEnd, equalTo(9))
            // Punctuation symbol added, first the extra whitespace is removed by the system
            deleteSurroundingText(1, 0)
            // Then the punctuation symbol is added instead
            commitText(".", 1)
        }
        assertThat(textView.text.toString(), equalTo("Anything."))
    }

    @Test
    fun testPunctuationSymbolAfterCommittingText_Android14() {
        // This simulates the behaviour of Gboard on Android < 13 when adding a punctuation symbol
        // after a committed word.
        inputConnection.run {
            // Manually entered letter
            setComposingText("A", 1)
            // Committed text from suggestions
            replaceText(0, 1, "Anything ", 1, null)
            // Assert selection is properly updated
            assertThat(textView.selectionEnd, equalTo(9))
            // Punctuation symbol added, first the extra whitespace is removed by the system
            deleteSurroundingText(1, 0)
            // Then the punctuation symbol is added instead
            commitText(".", 1)
        }
        assertThat(textView.text.toString(), equalTo("Anything."))
    }

    private fun simulateInput(editorInputAction: EditorInputAction) =
        when (val result = viewModel.processInput(editorInputAction)) {
            is ComposerResult.ReplaceText -> {
                textView.setText(result.text)
                textView.setSelection(result.selection.first, result.selection.last)
            }
            is ComposerResult.SelectionUpdated -> {
                textView.setSelection(result.selection.first, result.selection.last)
            }
            null -> Unit
        }
}

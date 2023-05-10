package io.element.android.wysiwyg.viewmodel

import io.element.android.wysiwyg.internal.viewmodel.EditorInputAction
import io.element.android.wysiwyg.view.models.InlineFormat
import io.element.android.wysiwyg.view.models.LinkAction
import io.element.android.wysiwyg.internal.viewmodel.ReplaceTextResult
import io.element.android.wysiwyg.internal.viewmodel.EditorViewModel
import io.element.android.wysiwyg.mocks.MockComposer
import io.element.android.wysiwyg.mocks.MockComposerUpdateFactory
import io.element.android.wysiwyg.mocks.MockTextUpdateFactory
import io.element.android.wysiwyg.utils.BasicHtmlConverter
import io.element.android.wysiwyg.utils.RustErrorCollector
import io.mockk.mockk
import io.mockk.verify
import org.hamcrest.MatcherAssert.assertThat
import org.hamcrest.Matchers.equalTo
import org.hamcrest.Matchers.notNullValue
import org.junit.Before
import org.junit.Test
import uniffi.wysiwyg_composer.ActionState
import uniffi.wysiwyg_composer.ComposerAction
import uniffi.wysiwyg_composer.MenuAction
import uniffi.wysiwyg_composer.MenuState
import uniffi.wysiwyg_composer.PatternKey
import uniffi.wysiwyg_composer.SuggestionPattern
import uniffi.wysiwyg_composer.LinkAction as ComposerLinkAction

internal class EditorViewModelTest {

    private val composer = MockComposer()
    private val htmlConverter = BasicHtmlConverter()
    private val viewModel = EditorViewModel(
        provideComposer = { composer.instance },
        htmlConverter = htmlConverter,
    )
    private val actionsStatesCallback = mockk<(
        Map<ComposerAction, ActionState>
    ) -> Unit>(relaxed = true)

    companion object {
        private const val paragraph =
            "Lorem Ipsum is simply dummy text of the printing and typesetting industry."
        private const val updatedParagraph =
            "Lorem Ipsum is updated!"
        private const val htmlParagraphs =
            "<p><b>$paragraph</b></p>" +
                    "<p><i>$paragraph</i></p>"
        private const val markdownParagraphs = "**paragraph**\n**paragraph**"
        private const val linkUrl = "https://matrix.org"
        private const val linkText = "Matrix"
        private val actionStates =
            mapOf(
                ComposerAction.BOLD to ActionState.REVERSED,
                ComposerAction.LINK to ActionState.DISABLED,
                ComposerAction.ITALIC to ActionState.ENABLED,
                ComposerAction.UNORDERED_LIST to ActionState.ENABLED,
                ComposerAction.ORDERED_LIST to ActionState.ENABLED,
                ComposerAction.REDO to ActionState.ENABLED,
                ComposerAction.UNDO to ActionState.ENABLED,
                ComposerAction.STRIKE_THROUGH to ActionState.ENABLED,
                ComposerAction.INLINE_CODE to ActionState.ENABLED,
                ComposerAction.UNDERLINE to ActionState.ENABLED,
                ComposerAction.INDENT to ActionState.ENABLED,
                ComposerAction.UNINDENT to ActionState.ENABLED,
            )

        private val composerStateUpdate = MockComposerUpdateFactory.create(
            textUpdate = MockTextUpdateFactory.createReplaceAll(updatedParagraph, 2, 3),
            menuState = MenuState.Update(actionStates = actionStates),
        )
        private val replaceTextResult = ReplaceTextResult(updatedParagraph, 2..3)
    }

    @Before
    fun setUp() {
        viewModel.setActionStatesCallback(actionsStatesCallback)
    }

    @Test
    fun `when menu state callback is not set, it processes input without an error`() {
        composer.givenReplaceTextResult(composerStateUpdate)
        viewModel.setActionStatesCallback(null)

        val result = viewModel.processInput(EditorInputAction.ReplaceText(paragraph))

        verify(inverse = true) {
            actionsStatesCallback(actionStates)
        }

        assertThat(result, equalTo(replaceTextResult))
    }

    @Test
    fun `when process replace text action, it returns a text update`() {
        composer.givenReplaceTextResult(composerStateUpdate)

        val result = viewModel.processInput(EditorInputAction.ReplaceText(paragraph))

        verify {
            composer.instance.replaceText(paragraph)
            actionsStatesCallback(actionStates)
        }
        assertThat(result, equalTo(replaceTextResult))
    }

    @Test
    fun `when process insert paragraph action, it returns a text update`() {
        composer.givenEnterResult(composerStateUpdate)

        val result = viewModel.processInput(EditorInputAction.InsertParagraph)

        verify {
            composer.instance.enter()
            actionsStatesCallback(actionStates)
        }
        assertThat(result, equalTo(replaceTextResult))
    }

    @Test
    fun `when process backspace action, it returns a text update`() {
        composer.givenBackspaceResult(composerStateUpdate)

        val result = viewModel.processInput(EditorInputAction.BackPress)

        verify {
            composer.instance.backspace()
            actionsStatesCallback(actionStates)
        }
        assertThat(result, equalTo(replaceTextResult))
    }

    @Test
    fun `when process bold action, it returns a text update`() {
        composer.givenBoldResult(composerStateUpdate)

        val result = viewModel.processInput(EditorInputAction.ApplyInlineFormat(InlineFormat.Bold))

        verify {
            composer.instance.bold()
            actionsStatesCallback(actionStates)
        }
        assertThat(result, equalTo(replaceTextResult))
    }

    @Test
    fun `when process italic action, it returns a text update`() {
        composer.givenItalicResult(composerStateUpdate)

        val result =
            viewModel.processInput(EditorInputAction.ApplyInlineFormat(InlineFormat.Italic))

        verify {
            composer.instance.italic()
            actionsStatesCallback(actionStates)
        }
        assertThat(result, equalTo(replaceTextResult))
    }


    @Test
    fun `when process underline action, it returns a text update`() {
        composer.givenUnderlineResult(composerStateUpdate)

        val result =
            viewModel.processInput(EditorInputAction.ApplyInlineFormat(InlineFormat.Underline))

        verify {
            composer.instance.underline()
            actionsStatesCallback(actionStates)
        }
        assertThat(result, equalTo(replaceTextResult))
    }

    @Test
    fun `when process strike through action, it returns a text update`() {
        composer.givenStrikeThroughResult(composerStateUpdate)

        val result =
            viewModel.processInput(EditorInputAction.ApplyInlineFormat(InlineFormat.StrikeThrough))

        verify {
            composer.instance.strikeThrough()
            actionsStatesCallback(actionStates)
        }
        assertThat(result, equalTo(replaceTextResult))
    }

    @Test
    fun `when process inline code action, it returns a text update`() {
        composer.givenInlineCodeResult(composerStateUpdate)

        val result =
            viewModel.processInput(EditorInputAction.ApplyInlineFormat(InlineFormat.InlineCode))

        verify {
            composer.instance.inlineCode()
            actionsStatesCallback(actionStates)
        }
        assertThat(result, equalTo(replaceTextResult))
    }

    @Test
    fun `when process delete in action, it returns a text update`() {
        composer.givenDeleteInResult(3, 4, composerStateUpdate)

        val result = viewModel.processInput(EditorInputAction.DeleteIn(3, 4))

        verify {
            composer.instance.deleteIn(3.toUInt(), 4.toUInt())
            actionsStatesCallback(actionStates)
        }
        assertThat(result, equalTo(replaceTextResult))
    }

    @Test
    fun `given internal edit link action, when get, it returns the right action`() {
        composer.givenLinkAction(ComposerLinkAction.Edit(linkUrl))

        assertThat(
            viewModel.getLinkAction(), equalTo(
                LinkAction.SetLink(
                    currentUrl = linkUrl
                )
            )
        )
    }

    @Test
    fun `given internal create with text action, when get, it returns the right action`() {
        composer.givenLinkAction(ComposerLinkAction.CreateWithText)

        assertThat(viewModel.getLinkAction(), equalTo(LinkAction.InsertLink))
    }

    @Test
    fun `given internal create link action, when get, it returns the right action`() {
        composer.givenLinkAction(ComposerLinkAction.Create)

        assertThat(
            viewModel.getLinkAction(), equalTo(
                LinkAction.SetLink(
                    currentUrl = null
                )
            )
        )
    }

    @Test
    fun `when process set link action, it returns a text update`() {
        composer.givenSetLinkResult("https://element.io", composerStateUpdate)

        val result = viewModel.processInput(EditorInputAction.SetLink("https://element.io"))

        verify {
            composer.instance.setLink("https://element.io", attributes = emptyList())
            actionsStatesCallback(actionStates)
        }
        assertThat(result, equalTo(replaceTextResult))
    }

    @Test
    fun `when process remove link action, it returns a text update`() {
        composer.givenRemoveLinkResult(composerStateUpdate)

        val result = viewModel.processInput(EditorInputAction.RemoveLink)

        verify {
            composer.instance.removeLinks()
            actionsStatesCallback(actionStates)
        }
        assertThat(result, equalTo(replaceTextResult))
    }

    @Test
    fun `when process set link with text action, it returns a text update`() {
        composer.givenSetLinkWithTextResult(
            url = linkUrl, text = linkText,
            composerStateUpdate
        )

        val result = viewModel.processInput(
            EditorInputAction.SetLinkWithText(linkUrl, linkText)
        )

        verify {
            composer.instance.setLinkWithText(
                url = linkUrl, text = linkText, attributes = emptyList()
            )
            actionsStatesCallback(actionStates)
        }
        assertThat(result, equalTo(replaceTextResult))
    }

    @Test
    fun `when process set link suggestion action, it returns a text update`() {
        val name = "jonny"
        val url = "https://matrix.to/#/@test:matrix.org"
        val suggestionPattern =
            SuggestionPattern(PatternKey.AT, text = "jonny", 0.toUInt(), 5.toUInt())
        composer.givenReplaceTextResult(MockComposerUpdateFactory.create(
            menuAction = MenuAction.Suggestion(suggestionPattern)
        ))
        viewModel.processInput(EditorInputAction.ReplaceText("@jonny"))

        composer.givenSetLinkSuggestionResult(name, url, composerStateUpdate)
        val result = viewModel.processInput(EditorInputAction.SetLinkSuggestion(url, name))

        verify {
            composer.instance.setLinkSuggestion(url, attributes = emptyList(), text = name, suggestion = suggestionPattern)
        }
        assertThat(result, equalTo(replaceTextResult))
    }

    @Test
    fun `when process replace all html action, it returns a text update`() {
        composer.givenReplaceAllHtmlResult("new html", composerStateUpdate)

        val result = viewModel.processInput(EditorInputAction.ReplaceAllHtml("new html"))

        verify {
            composer.instance.setContentFromHtml("new html")
            actionsStatesCallback(actionStates)
        }
        assertThat(result, equalTo(replaceTextResult))
    }

    @Test
    fun `when process replace all markdown action, it returns a text update`() {
        composer.givenReplaceAllMarkdownResult("new **markdown**", composerStateUpdate)

        val result =
            viewModel.processInput(EditorInputAction.ReplaceAllMarkdown("new **markdown**"))

        verify {
            composer.instance.setContentFromMarkdown("new **markdown**")
            actionsStatesCallback(actionStates)
        }
        assertThat(result, equalTo(replaceTextResult))
    }

    @Test
    fun `when process undo action, it returns a text update`() {
        composer.givenUndoResult(composerStateUpdate)

        val result = viewModel.processInput(EditorInputAction.Undo)

        verify {
            composer.instance.undo()
            actionsStatesCallback(actionStates)
        }
        assertThat(result, equalTo(replaceTextResult))
    }

    @Test
    fun `when process redo action, it returns a text update`() {
        composer.givenRedoResult(composerStateUpdate)

        val result = viewModel.processInput(EditorInputAction.Redo)

        verify {
            composer.instance.redo()
            actionsStatesCallback(actionStates)
        }
        assertThat(result, equalTo(replaceTextResult))
    }

    @Test
    fun `when process toggle ordered list action, it returns a text update`() {
        composer.givenToggleOrderedListResult(composerStateUpdate)

        val result = viewModel.processInput(EditorInputAction.ToggleList(ordered = true))

        verify {
            composer.instance.orderedList()
            actionsStatesCallback(actionStates)
        }
        assertThat(result, equalTo(replaceTextResult))
    }

    @Test
    fun `when process toggle unordered list action, it returns a text update`() {
        composer.givenToggleUnorderedListResult(composerStateUpdate)

        val result = viewModel.processInput(EditorInputAction.ToggleList(ordered = false))

        verify {
            composer.instance.unorderedList()
            actionsStatesCallback(actionStates)
        }
        assertThat(result, equalTo(replaceTextResult))
    }

    @Test
    fun `when process code block, it returns a text update`() {
        composer.givenToggleCodeBlock(composerStateUpdate)

        val result = viewModel.processInput(EditorInputAction.CodeBlock)

        verify {
            composer.instance.codeBlock()
            actionsStatesCallback(actionStates)
        }
        assertThat(result, equalTo(replaceTextResult))
    }

    @Test
    fun `when process quote, it returns a text update`() {
        composer.givenToggleQuote(composerStateUpdate)

        val result = viewModel.processInput(EditorInputAction.Quote)

        verify {
            composer.instance.quote()
            actionsStatesCallback(actionStates)
        }
        assertThat(result, equalTo(replaceTextResult))
    }

    @Test
    fun `given formatted text, getHtml function returns formatted HTML`() {
        composer.givenGetContentAsHtml(htmlParagraphs)

        val html = viewModel.getHtml()

        assertThat(html, equalTo(htmlParagraphs))
    }

    @Test
    fun `given markdown text, getMarkdown function returns markdown`() {
        composer.givenGetContentAsMarkdown(markdownParagraphs)

        val html = viewModel.getMarkdown()

        assertThat(html, equalTo(markdownParagraphs))
    }

    @Test
    fun `given an error callback, it will collect errors thrown by the Rust library`() {
        composer.givenErrorInUpdateSelection()
        var errorCollected: Throwable? = null
        // Collect the error
        viewModel.rustErrorCollector = RustErrorCollector { error ->
            errorCollected = error
        }

        // Use runCatching so the tests can continue in debug mode, otherwise they would crash
        runCatching { viewModel.updateSelection(mockk(relaxed = true), 0, 0) }

        assertThat(errorCollected, notNullValue())
    }

}

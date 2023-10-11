package io.element.android.wysiwyg.internal.viewmodel

import android.text.Editable
import androidx.annotation.VisibleForTesting
import androidx.lifecycle.ViewModel
import io.element.android.wysiwyg.BuildConfig
import io.element.android.wysiwyg.extensions.log
import io.element.android.wysiwyg.extensions.string
import io.element.android.wysiwyg.internal.view.models.toApiModel
import io.element.android.wysiwyg.utils.EditorIndexMapper
import io.element.android.wysiwyg.utils.HtmlConverter
import io.element.android.wysiwyg.utils.RustErrorCollector
import io.element.android.wysiwyg.view.models.InlineFormat
import io.element.android.wysiwyg.view.models.LinkAction
import uniffi.wysiwyg_composer.*

internal class EditorViewModel(
    private val provideComposer: () -> ComposerModelInterface?,
    private val htmlConverter: HtmlConverter,
) : ViewModel() {

    private var composer: ComposerModelInterface? = provideComposer()

    var rustErrorCollector: RustErrorCollector? = null

    private var actionStatesCallback: ((Map<ComposerAction, ActionState>) -> Unit)? = null
    var menuActionCallback: ((MenuAction) -> Unit)? = null
    var linkActionCallback: ((LinkAction?) -> Unit)? = null

    private var curMenuAction: MenuAction = MenuAction.None
    private var curLinkAction: LinkAction? = null

    // If there is an internal error in the Rust model, we can manually recover to this state.
    private var recoveryContentPlainText: String = ""

    private var crashOnComposerFailure: Boolean = BuildConfig.DEBUG

    fun setActionStatesCallback(callback: ((Map<ComposerAction, ActionState>) -> Unit)?) {
        this.actionStatesCallback = callback
        actionStates()?.let { actionStatesCallback?.invoke(it) }
    }

    fun updateSelection(editable: Editable, start: Int, end: Int) {
        val (newStart, newEnd) = EditorIndexMapper.fromEditorToComposer(start, end, editable)
            ?: return

        val update = runCatching {
            composer?.select(newStart, newEnd)
        }
            .onFailure(::onComposerFailure)
            .getOrNull()

        handleComposerUpdates(update)

        composer?.log()
    }

    fun processInput(action: EditorInputAction): ReplaceTextResult? {
        val update = runCatching {
            when (action) {
                is EditorInputAction.ReplaceText -> {
                    // This conversion to a plain String might be too simple
                    composer?.replaceText(action.value.toString())
                }

                is EditorInputAction.ReplaceTextIn -> {
                    // This conversion to a plain String might be too simple
                    composer?.replaceTextIn(
                        action.value.toString(),
                        action.start.toUInt(),
                        action.end.toUInt()
                    )
                }

                is EditorInputAction.ReplaceTextSuggestion -> replaceTextSuggestion(action)
                is EditorInputAction.InsertParagraph -> composer?.enter()
                is EditorInputAction.BackPress -> composer?.backspace()
                is EditorInputAction.ApplyInlineFormat -> when (action.format) {
                    InlineFormat.Bold -> composer?.bold()
                    InlineFormat.Italic -> composer?.italic()
                    InlineFormat.Underline -> composer?.underline()
                    InlineFormat.StrikeThrough -> composer?.strikeThrough()
                    InlineFormat.InlineCode -> composer?.inlineCode()
                }

                is EditorInputAction.DeleteIn -> composer?.deleteIn(
                    action.start.toUInt(),
                    action.end.toUInt()
                )

                is EditorInputAction.Delete -> composer?.delete()
                is EditorInputAction.SetLink -> composer?.setLink(
                    url = action.url,
                    attributes = emptyList()
                )

                is EditorInputAction.EditLink -> composer?.editLinkWithText(
                    url = action.url,
                    text = action.text,
                    attributes = emptyList()
                )

                is EditorInputAction.RemoveLink -> composer?.removeLinks()
                is EditorInputAction.SetLinkWithText -> composer?.setLinkWithText(
                    action.link,
                    action.text,
                    attributes = emptyList()
                )

                is EditorInputAction.ReplaceAllHtml -> composer?.setContentFromHtml(action.html)
                is EditorInputAction.ReplaceAllMarkdown -> composer?.setContentFromMarkdown(action.markdown)
                is EditorInputAction.Undo -> composer?.undo()
                is EditorInputAction.Redo -> composer?.redo()
                is EditorInputAction.ToggleList ->
                    if (action.ordered) composer?.orderedList() else composer?.unorderedList()

                is EditorInputAction.CodeBlock -> composer?.codeBlock()
                is EditorInputAction.Quote -> composer?.quote()
                is EditorInputAction.Indent -> composer?.indent()
                is EditorInputAction.Unindent -> composer?.unindent()
                is EditorInputAction.SetLinkSuggestion -> insertMentionAtSuggestion(action)
            }
        }.onFailure(::onComposerFailure)
            .getOrNull()

        composer?.log()

        return handleComposerUpdates(update)
    }

    private fun handleComposerUpdates(update: ComposerUpdate?): ReplaceTextResult? {
        if (update != null) {
            val menuState = update.menuState()
            if (menuState is MenuState.Update) {
                actionStatesCallback?.invoke(menuState.actionStates)
            }

            val menuAction = update.menuAction()
            if (menuAction !is MenuAction.Keep) {
                curMenuAction = menuAction
                menuActionCallback?.invoke(menuAction)
            }

            val linkActionUpdate = update.linkAction()
            if (linkActionUpdate is LinkActionUpdate.Update) {
                curLinkAction = linkActionUpdate.linkAction.toApiModel()
                linkActionCallback?.invoke(curLinkAction)
            }
        }

        return when (val textUpdate = update?.textUpdate()) {
            is TextUpdate.ReplaceAll -> {
                val replacementHtml = textUpdate.replacementHtml.string()

                recoveryContentPlainText = composer?.getContentAsPlainText() ?: ""

                ReplaceTextResult(
                    text = stringToSpans(replacementHtml),
                    selection = textUpdate.startUtf16Codeunit.toInt()..textUpdate.endUtf16Codeunit.toInt(),
                )
            }

            is TextUpdate.Select,
            is TextUpdate.Keep,
            null -> null
        }
    }

    fun getContentAsMessageHtml(): String {
        return composer?.getContentAsMessageHtml().orEmpty()
    }

    fun getMarkdown(): String =
        composer?.getContentAsMarkdown().orEmpty()

    fun getCurrentFormattedText(): CharSequence {
        return stringToSpans(getContentAsMessageHtml())
    }

    /**
     * Get the Rust model's internal representation of it's content.
     *
     * Note that this should not be used for messages; instead [getContentAsMessageHtml] should be used.
     */
    fun getInternalHtml(): String {
        return composer?.getContentAsHtml().orEmpty()
    }

    fun actionStates(): Map<ComposerAction, ActionState>? {
        return composer?.actionStates()
    }

    fun getLinkAction(): LinkAction? =
        composer?.getLinkAction()?.toApiModel()

    fun rerender(): CharSequence =
        stringToSpans(getInternalHtml())

    private fun onComposerFailure(error: Throwable, attemptContentRecovery: Boolean = true) {
        rustErrorCollector?.onRustError(error)

        if (crashOnComposerFailure) {
            throw error
        }

        if (error is InternalException) { // InternalException means Rust panic
            // Recover from the crash
            composer = provideComposer()

            if (attemptContentRecovery) {
                runCatching {
                    composer?.replaceText(recoveryContentPlainText)
                }.onFailure {
                    onComposerFailure(it, attemptContentRecovery = false)
                }
            }
        }
    }

    private fun insertMentionAtSuggestion(action: EditorInputAction.SetLinkSuggestion): ComposerUpdate? {
        val (url, text) = action

        val suggestion = (curMenuAction as? MenuAction.Suggestion)
            ?.suggestionPattern
            ?: return null

        return composer?.insertMentionAtSuggestion(
            url = url,
            text = text,
            suggestion = suggestion,
            attributes = emptyList()
        )
    }

    private fun replaceTextSuggestion(action: EditorInputAction.ReplaceTextSuggestion): ComposerUpdate? {
        val suggestion = (curMenuAction as? MenuAction.Suggestion)
            ?.suggestionPattern
            ?: return null

        return composer?.replaceTextSuggestion(
            suggestion = suggestion,
            newText = action.value,
        )
    }

    @VisibleForTesting
    internal fun testComposerCrashRecovery() {
        val crashOnComposerFailure = this.crashOnComposerFailure

        // Normally debug builds should fail fast and crash but
        // we disable this behaviour in order to test the recovery
        // behaviour
        this.crashOnComposerFailure = false

        runCatching {
            composer?.debugPanic()
        }.onFailure {
            onComposerFailure(it)
        }

        this.crashOnComposerFailure = crashOnComposerFailure
    }

    private fun stringToSpans(string: String): CharSequence =
        htmlConverter.fromHtmlToSpans(string)

}

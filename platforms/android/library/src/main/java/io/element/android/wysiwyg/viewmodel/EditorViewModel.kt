package io.element.android.wysiwyg.viewmodel

import android.text.Editable
import androidx.lifecycle.ViewModel
import io.element.android.wysiwyg.extensions.log
import io.element.android.wysiwyg.extensions.string
import io.element.android.wysiwyg.inputhandlers.models.EditorInputAction
import io.element.android.wysiwyg.inputhandlers.models.InlineFormat
import io.element.android.wysiwyg.inputhandlers.models.LinkAction
import io.element.android.wysiwyg.inputhandlers.models.ReplaceTextResult
import io.element.android.wysiwyg.utils.EditorIndexMapper
import io.element.android.wysiwyg.utils.HtmlConverter
import io.element.android.wysiwyg.utils.RustErrorCollector
import io.element.android.wysiwyg.utils.throwIfDebugBuild
import uniffi.wysiwyg_composer.*
import uniffi.wysiwyg_composer.LinkAction as ComposerLinkAction

internal class EditorViewModel(
    private val composer: ComposerModelInterface?,
    private val htmlConverter: HtmlConverter,
) : ViewModel() {

    var rustErrorCollector: RustErrorCollector? = null

    private var actionStatesCallback: ((Map<ComposerAction, ActionState>) -> Unit)? = null

    fun setActionStatesCallback(callback: ((Map<ComposerAction, ActionState>) -> Unit)?) {
        this.actionStatesCallback = callback
        actionStates()?.let { actionStatesCallback?.invoke(it) }
    }

    fun updateSelection(editable: Editable, start: Int, end: Int) {
        val (newStart, newEnd) = EditorIndexMapper.fromEditorToComposer(start, end, editable)
            ?: return

        val update = runCatching {
            composer?.select(newStart, newEnd)
        }.onFailure { error ->
            rustErrorCollector?.onRustError(error)
            error.throwIfDebugBuild()
        }.getOrNull()
        val menuState = update?.menuState()
        if (menuState is MenuState.Update) {
            actionStatesCallback?.invoke(menuState.actionStates)
        }
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
                    composer?.replaceTextIn(action.value.toString(), action.start.toUInt(), action.end.toUInt())
                }
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
                is EditorInputAction.SetLink -> composer?.setLink(link = action.link)
                is EditorInputAction.RemoveLink -> composer?.removeLinks()
                is EditorInputAction.SetLinkWithText -> composer?.setLinkWithText(action.link, action.text)
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
            }
        }.onFailure { error ->
            rustErrorCollector?.onRustError(error)
            error.throwIfDebugBuild()
        }.getOrNull()

        composer?.log()

        val menuState = update?.menuState()
        if (menuState is MenuState.Update) {
            actionStatesCallback?.invoke(menuState.actionStates)
        }

        return when (val textUpdate = update?.textUpdate()) {
            is TextUpdate.ReplaceAll -> ReplaceTextResult(
                text = stringToSpans(textUpdate.replacementHtml.string()),
                selection = textUpdate.startUtf16Codeunit.toInt()..textUpdate.endUtf16Codeunit.toInt(),
            )
            is TextUpdate.Select,
            is TextUpdate.Keep,
            null -> null
        }
    }

    fun getHtml(): String {
        return composer?.getContentAsHtml().orEmpty()
    }

    fun getMarkdown(): String =
        composer?.getContentAsMarkdown().orEmpty()

    fun getCurrentFormattedText(): CharSequence {
        return stringToSpans(getHtml())
    }

    fun actionStates(): Map<ComposerAction, ActionState>? {
        return composer?.actionStates()
    }

    fun getLinkAction(): LinkAction? =
        composer?.getLinkAction()?.let {
            when(it) {
                is ComposerLinkAction.Edit -> LinkAction.SetLink(currentLink = it.link)
                is ComposerLinkAction.Create -> LinkAction.SetLink(currentLink = null)
                is ComposerLinkAction.CreateWithText -> LinkAction.InsertLink
            }
        }

    private fun stringToSpans(string: String): CharSequence =
        htmlConverter.fromHtmlToSpans(string)

}

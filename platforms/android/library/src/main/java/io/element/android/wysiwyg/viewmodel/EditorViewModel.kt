package io.element.android.wysiwyg.viewmodel

import android.text.Editable
import androidx.lifecycle.ViewModel
import io.element.android.wysiwyg.extensions.log
import io.element.android.wysiwyg.extensions.string
import io.element.android.wysiwyg.inputhandlers.models.EditorInputAction
import io.element.android.wysiwyg.inputhandlers.models.InlineFormat
import io.element.android.wysiwyg.inputhandlers.models.ReplaceTextResult
import io.element.android.wysiwyg.utils.EditorIndexMapper
import io.element.android.wysiwyg.utils.HtmlConverter
import io.element.android.wysiwyg.utils.throwIfDebugBuild
import uniffi.wysiwyg_composer.ComposerModelInterface
import uniffi.wysiwyg_composer.MenuState
import uniffi.wysiwyg_composer.TextUpdate

internal class EditorViewModel(
    private val composer: ComposerModelInterface?,
    private val htmlConverter: HtmlConverter,
) : ViewModel() {

    private var menuStateCallback: ((MenuState) -> Unit)? = null

    fun setMenuStateCallback(callback: ((MenuState) -> Unit)?) {
        this.menuStateCallback = callback
        getMenuState()?.let { menuStateCallback?.invoke(it) }
    }

    fun updateSelection(editable: Editable, start: Int, end: Int) {
        val (newStart, newEnd) = EditorIndexMapper.fromEditorToComposer(start, end, editable)
            ?: return

        val update = composer?.select(newStart, newEnd)
        val menuState = update?.menuState()
        if (menuState is MenuState.Update) {
            menuStateCallback?.invoke(menuState)
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
                is EditorInputAction.InsertParagraph -> composer?.enter()
                is EditorInputAction.BackPress -> composer?.backspace()
                is EditorInputAction.ApplyInlineFormat -> when (action.format) {
                    InlineFormat.Bold -> composer?.bold()
                    InlineFormat.Italic -> composer?.italic()
                    InlineFormat.Underline -> composer?.underline()
                    InlineFormat.StrikeThrough -> composer?.strikeThrough()
                    InlineFormat.InlineCode -> composer?.inlineCode()
                }
                is EditorInputAction.Delete -> composer?.deleteIn(
                    action.start.toUInt(),
                    action.end.toUInt()
                )
                is EditorInputAction.SetLink -> composer?.setLink(action.link)
                is EditorInputAction.ReplaceAllHtml -> composer?.replaceAllHtml(action.html)
                is EditorInputAction.Undo -> composer?.undo()
                is EditorInputAction.Redo -> composer?.redo()
                is EditorInputAction.ToggleList ->
                    if (action.ordered) composer?.orderedList() else composer?.unorderedList()
            }
        }.onFailure {
            it.throwIfDebugBuild()
        }.getOrNull()

        composer?.log()

        update?.menuState()?.let { menuStateCallback?.invoke(it) }

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
        return composer?.getCurrentDomState()?.html?.string().orEmpty()
    }

    fun getPlainText(): String =
        htmlConverter.fromHtmlToPlainText(getHtml())

    fun getCurrentFormattedText(): CharSequence {
        return stringToSpans(getHtml())
    }

    fun getMenuState(): MenuState? {
        return composer?.getCurrentMenuState()
    }

    private fun stringToSpans(string: String): CharSequence =
        htmlConverter.fromHtmlToSpans(string)

}

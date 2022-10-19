package io.element.android.wysiwyg.viewmodel

import android.content.Context
import android.text.Editable
import android.text.Spanned
import androidx.lifecycle.ViewModel
import io.element.android.wysiwyg.BuildConfig
import io.element.android.wysiwyg.extensions.log
import io.element.android.wysiwyg.extensions.string
import io.element.android.wysiwyg.inputhandlers.models.EditorInputAction
import io.element.android.wysiwyg.inputhandlers.models.InlineFormat
import io.element.android.wysiwyg.inputhandlers.models.ReplaceTextResult
import io.element.android.wysiwyg.utils.AndroidResourcesProvider
import io.element.android.wysiwyg.utils.EditorIndexMapper
import io.element.android.wysiwyg.utils.HtmlToSpansParser
import io.element.android.wysiwyg.utils.ResourcesProvider
import uniffi.wysiwyg_composer.ComposerModel
import uniffi.wysiwyg_composer.MenuState
import uniffi.wysiwyg_composer.TextUpdate
import uniffi.wysiwyg_composer.newComposerModel

internal class EditorViewModel : ViewModel() {

    private var composer: ComposerModel? = null
    private var menuStateCallback: ((MenuState) -> Unit)? = null
    private lateinit var resourcesProvider: ResourcesProvider

    fun initialize(context: Context, isInEditMode: Boolean) {
        resourcesProvider = AndroidResourcesProvider(context)
        if (composer == null) {
            composer = if (isInEditMode) null else newComposerModel()
        }
    }

    fun setMenuStateCallback(callback: ((MenuState) -> Unit)?) {
        this.menuStateCallback = callback
        getMenuState()?.let { menuStateCallback?.invoke(it) }
    }

    fun updateSelection(editable: Editable, start: Int, end: Int) {
        val (newStart, newEnd) = EditorIndexMapper.fromEditorToComposer(start, end, editable) ?: return

        val update = composer?.select(newStart, newEnd)
        val menuState = update?.menuState()
        if (menuState is MenuState.Update) {
            menuStateCallback?.invoke(menuState)
        }
        composer?.log()
    }

    fun processInput(action: EditorInputAction): TextUpdate? {
        val update = runCatching {
            when (action) {
                is EditorInputAction.ReplaceText -> {
                    // This conversion to a plain String might be too simple
                    composer?.replaceText(action.value.toString())
                }
                is EditorInputAction.InsertParagraph -> {
                    composer?.enter()
                }
                is EditorInputAction.BackPress -> {
                    composer?.backspace()
                }
                is EditorInputAction.ApplyInlineFormat -> {
                    when (action.format) {
                        InlineFormat.Bold -> composer?.bold()
                        InlineFormat.Italic -> composer?.italic()
                        InlineFormat.Underline -> composer?.underline()
                        InlineFormat.StrikeThrough -> composer?.strikeThrough()
                        InlineFormat.InlineCode -> composer?.inlineCode()
                    }
                }
                is EditorInputAction.Delete -> {
                    composer?.deleteIn(action.start.toUInt(), action.end.toUInt())
                }
                is EditorInputAction.SetLink -> composer?.setLink(action.link)
                is EditorInputAction.ReplaceAllHtml -> composer?.replaceAllHtml(action.html)
                is EditorInputAction.Undo -> composer?.undo()
                is EditorInputAction.Redo -> composer?.redo()
                is EditorInputAction.ToggleList -> {
                    if (action.ordered) composer?.orderedList() else composer?.unorderedList()
                }
            }
        }.onFailure {
            if (BuildConfig.DEBUG) {
                throw it
            } else {
                it.printStackTrace()
            }
        }.getOrNull()

        update?.menuState()?.let { menuStateCallback?.invoke(it) }

        return update?.textUpdate().also {
            composer?.log()
        }
    }

    fun processUpdate(update: TextUpdate): ReplaceTextResult? {
        return when (update) {
            is TextUpdate.Keep -> null
            is TextUpdate.ReplaceAll -> {
                ReplaceTextResult(
                    text = stringToSpans(update.replacementHtml.string()),
                    selection = update.startUtf16Codeunit.toInt()..update.endUtf16Codeunit.toInt(),
                )
            }
            is TextUpdate.Select -> null
        }
    }

    fun getHtml(): String {
        return composer?.getCurrentDomState()?.html?.string().orEmpty()
    }

    fun getCurrentFormattedText(): CharSequence {
        return stringToSpans(getHtml())
    }

    fun getMenuState(): MenuState? {
        return composer?.getCurrentMenuState() as? MenuState.Update
    }

    private fun stringToSpans(string: String): Spanned {
        return HtmlToSpansParser(resourcesProvider, string).convert()
    }

}

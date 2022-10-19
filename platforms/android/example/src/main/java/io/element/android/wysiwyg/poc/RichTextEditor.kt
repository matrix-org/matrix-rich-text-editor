package io.element.android.wysiwyg.poc

import android.content.Context
import android.graphics.Rect
import android.util.AttributeSet
import android.view.LayoutInflater
import android.view.View
import com.google.android.material.textfield.TextInputLayout
import io.element.android.wysiwyg.EditorEditText
import io.element.android.wysiwyg.inputhandlers.models.InlineFormat
import io.element.android.wysiwyg.poc.databinding.ViewRichTextEditorBinding
import uniffi.wysiwyg_composer.ComposerAction
import uniffi.wysiwyg_composer.MenuState

class RichTextEditor : TextInputLayout {

    private val binding = ViewRichTextEditorBinding.inflate(LayoutInflater.from(context), this, true)

    constructor(context: Context): super(context)

    constructor(context: Context, attrs: AttributeSet?): super(context, attrs)

    constructor(context: Context, attrs: AttributeSet?, defStyleAttr: Int):
            super(context, attrs, defStyleAttr)

    var onSetLinkListener: OnSetLinkListener? = null

    override fun onFinishInflate() {
        super.onFinishInflate()

        with (binding) {
            formatBoldButton.setOnClickListener {
                editText.toggleInlineFormat(InlineFormat.Bold)
            }
            formatItalicButton.setOnClickListener {
                editText.toggleInlineFormat(InlineFormat.Italic)
            }
            formatUnderlineButton.setOnClickListener {
                editText.toggleInlineFormat(InlineFormat.Underline)
            }
            formatStrikeThroughButton.setOnClickListener {
                editText.toggleInlineFormat(InlineFormat.StrikeThrough)
            }
            formatInlineCodeButton.setOnClickListener {
                editText.toggleInlineFormat(InlineFormat.InlineCode)
            }
            addLinkButton.setOnClickListener {
                onSetLinkListener?.openLinkDialog(null) { link ->
                    editText.setLink(link)
                }
            }
            undoButton.setOnClickListener {
                editText.undo()
            }
            redoButton.setOnClickListener {
                editText.redo()
            }
            orderedListButton.setOnClickListener {
                editText.toggleList(true)
            }
            unorderedListButton.setOnClickListener {
                editText.toggleList(false)
            }

            editText.menuStateChangedListener = EditorEditText.OnMenuStateChangedListener { state ->
                if (state is MenuState.Update) {
                    updateMenuState(state)
                }
            }
        }
    }

    private fun updateMenuState(menuState: MenuState.Update) {
        with(binding) {
            updateMenuStateFor(formatBoldButton, ComposerAction.Bold, menuState)
            updateMenuStateFor(formatItalicButton, ComposerAction.Italic, menuState)
            updateMenuStateFor(formatUnderlineButton, ComposerAction.Underline, menuState)
            updateMenuStateFor(formatInlineCodeButton, ComposerAction.InlineCode, menuState)
            updateMenuStateFor(formatStrikeThroughButton, ComposerAction.StrikeThrough, menuState)
            updateMenuStateFor(addLinkButton, ComposerAction.Link, menuState)
            updateMenuStateFor(undoButton, ComposerAction.Undo, menuState)
            updateMenuStateFor(redoButton, ComposerAction.Redo, menuState)
            updateMenuStateFor(orderedListButton, ComposerAction.OrderedList, menuState)
            updateMenuStateFor(unorderedListButton, ComposerAction.UnorderedList, menuState)
        }
    }

    private fun updateMenuStateFor(button: View, action: ComposerAction, menuState: MenuState.Update) {
        button.isEnabled = !menuState.disabledActions.contains(action)
        button.isActivated = menuState.reversedActions.contains(action)
    }

    override fun requestFocus(direction: Int, previouslyFocusedRect: Rect?): Boolean {
        return binding.editText.requestFocus(direction, previouslyFocusedRect)
    }

}

interface OnSetLinkListener {
    fun openLinkDialog(link: String?, callback: (String) -> Unit)
}

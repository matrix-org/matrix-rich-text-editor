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
import uniffi.wysiwyg_composer.ActionState
import uniffi.wysiwyg_composer.ComposerAction

class RichTextEditor : TextInputLayout {

    private val binding = ViewRichTextEditorBinding.inflate(LayoutInflater.from(context), this, true)

    constructor(context: Context): super(context)

    constructor(context: Context, attrs: AttributeSet?): super(context, attrs)

    constructor(context: Context, attrs: AttributeSet?, defStyleAttr: Int):
            super(context, attrs, defStyleAttr)

    var onSetLinkListener: OnSetLinkListener? = null

    override fun onAttachedToWindow() {
        super.onAttachedToWindow()

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

            editText.actionStatesChangedListener = EditorEditText.OnActionStatesChangedListener { actionStates ->
                updateActionStates(actionStates)
            }
        }
    }

    private fun updateActionStates(actionStates: Map<ComposerAction, ActionState>) {
        with(binding) {
            updateActionStateFor(formatBoldButton, ComposerAction.BOLD, actionStates)
            updateActionStateFor(formatItalicButton, ComposerAction.ITALIC, actionStates)
            updateActionStateFor(formatUnderlineButton, ComposerAction.UNDERLINE, actionStates)
            updateActionStateFor(formatInlineCodeButton, ComposerAction.INLINE_CODE, actionStates)
            updateActionStateFor(formatStrikeThroughButton, ComposerAction.STRIKE_THROUGH, actionStates)
            updateActionStateFor(addLinkButton, ComposerAction.LINK, actionStates)
            updateActionStateFor(undoButton, ComposerAction.UNDO, actionStates)
            updateActionStateFor(redoButton, ComposerAction.REDO, actionStates)
            updateActionStateFor(orderedListButton, ComposerAction.ORDERED_LIST, actionStates)
            updateActionStateFor(unorderedListButton, ComposerAction.UNORDERED_LIST, actionStates)
        }
    }

    private fun updateActionStateFor(
        button: View,
        action: ComposerAction,
        actionStates: Map<ComposerAction, ActionState>
    ) {
        val state = actionStates[action];
        button.isEnabled = state != ActionState.DISABLED;
        button.isActivated = state == ActionState.REVERSED;
    }

    override fun requestFocus(direction: Int, previouslyFocusedRect: Rect?): Boolean {
        return binding.editText.requestFocus(direction, previouslyFocusedRect)
    }

}

interface OnSetLinkListener {
    fun openLinkDialog(link: String?, callback: (String) -> Unit)
}

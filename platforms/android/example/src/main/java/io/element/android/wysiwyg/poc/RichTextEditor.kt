package io.element.android.wysiwyg.poc

import android.content.Context
import android.graphics.Rect
import android.util.AttributeSet
import android.view.LayoutInflater
import android.view.View
import android.widget.LinearLayout
import androidx.core.view.isGone
import androidx.core.view.isVisible
import io.element.android.wysiwyg.EditorEditText
import io.element.android.wysiwyg.inputhandlers.models.InlineFormat
import io.element.android.wysiwyg.inputhandlers.models.LinkAction
import io.element.android.wysiwyg.poc.databinding.ViewRichTextEditorBinding
import uniffi.wysiwyg_composer.ActionState
import uniffi.wysiwyg_composer.ComposerAction

class RichTextEditor : LinearLayout {

    private val binding = ViewRichTextEditorBinding.inflate(LayoutInflater.from(context), this, true)

    constructor(context: Context): super(context)

    constructor(context: Context, attrs: AttributeSet?): super(context, attrs)

    constructor(context: Context, attrs: AttributeSet?, defStyleAttr: Int):
            super(context, attrs, defStyleAttr)

    var onSetLinkListener: OnSetLinkListener? = null

    override fun onAttachedToWindow() {
        super.onAttachedToWindow()

        with (binding) {
            formattingSwitch.apply {
                isChecked = true
                setOnCheckedChangeListener { _, isChecked ->
                    if (isChecked) {
                        val markdown = binding.markdownEditText.text.toString()
                        binding.richTextEditText.setMarkdown(markdown)
                    } else {
                        val markdown = binding.richTextEditText.getMarkdown()
                        binding.markdownEditText.setText(markdown)
                    }
                    binding.menuFormattingGroup.isVisible = isChecked
                    binding.richTextInputLayout.isVisible = isChecked
                    binding.markdownInputLayout.isGone = isChecked
                }
            }
            formatBoldButton.setOnClickListener {
                richTextEditText.toggleInlineFormat(InlineFormat.Bold)
            }
            formatItalicButton.setOnClickListener {
                richTextEditText.toggleInlineFormat(InlineFormat.Italic)
            }
            formatUnderlineButton.setOnClickListener {
                richTextEditText.toggleInlineFormat(InlineFormat.Underline)
            }
            formatStrikeThroughButton.setOnClickListener {
                richTextEditText.toggleInlineFormat(InlineFormat.StrikeThrough)
            }
            formatInlineCodeButton.setOnClickListener {
                richTextEditText.toggleInlineFormat(InlineFormat.InlineCode)
            }
            formatCodeBlockButton.setOnClickListener {
                richTextEditText.toggleCodeBlock()
            }
            formatQuoteButton.setOnClickListener {
                richTextEditText.toggleQuote()
            }
            addLinkButton.setOnClickListener {
                val linkAction = richTextEditText.getLinkAction() ?: return@setOnClickListener
                when(linkAction) {
                    is LinkAction.InsertLink -> {
                        onSetLinkListener?.openInsertLinkDialog { text, link ->
                            richTextEditText.insertLink(link = link, text = text)
                        }
                    }
                    is LinkAction.SetLink ->
                        onSetLinkListener?.openSetLinkDialog(linkAction.currentLink) { link ->
                            richTextEditText.setLink(link)
                        }
                }
            }
            undoButton.setOnClickListener {
                richTextEditText.undo()
            }
            redoButton.setOnClickListener {
                richTextEditText.redo()
            }
            orderedListButton.setOnClickListener {
                richTextEditText.toggleList(true)
            }
            unorderedListButton.setOnClickListener {
                richTextEditText.toggleList(false)
            }
            indentButton.setOnClickListener {
                richTextEditText.indent()
            }
            unindentButton.setOnClickListener {
                richTextEditText.unindent()
            }

            richTextEditText.actionStatesChangedListener = EditorEditText.OnActionStatesChangedListener { actionStates ->
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
            updateActionStateFor(formatCodeBlockButton, ComposerAction.CODE_BLOCK, actionStates)
            updateActionStateFor(formatQuoteButton, ComposerAction.QUOTE, actionStates)
            updateActionStateFor(indentButton, ComposerAction.INDENT, actionStates)
            updateActionStateFor(unindentButton, ComposerAction.UNINDENT, actionStates)
        }
    }

    private fun updateActionStateFor(
        button: View,
        action: ComposerAction,
        actionStates: Map<ComposerAction, ActionState>
    ) {
        val state = actionStates[action]
        button.isVisible = state != ActionState.HIDDEN
        button.isEnabled = state != ActionState.DISABLED
        button.isActivated = state == ActionState.REVERSED
    }

    override fun requestFocus(direction: Int, previouslyFocusedRect: Rect?): Boolean {
        return binding.richTextEditText.requestFocus(direction, previouslyFocusedRect)
    }

}

interface OnSetLinkListener {
    fun openSetLinkDialog(currentLink: String?, callback: (url: String?) -> Unit)
    fun openInsertLinkDialog(callback: (text: String, url: String) -> Unit)
}

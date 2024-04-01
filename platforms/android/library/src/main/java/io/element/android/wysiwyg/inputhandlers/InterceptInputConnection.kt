package io.element.android.wysiwyg.inputhandlers

import android.text.Editable
import android.text.Selection
import android.view.KeyEvent
import android.view.inputmethod.BaseInputConnection.getComposingSpanEnd
import android.view.inputmethod.BaseInputConnection.getComposingSpanStart
import android.view.inputmethod.InputConnection
import android.view.inputmethod.InputConnectionWrapper
import android.view.inputmethod.TextAttribute
import android.widget.TextView
import androidx.annotation.VisibleForTesting
import androidx.core.text.isDigitsOnly
import io.element.android.wysiwyg.EditorTextWatcher
import io.element.android.wysiwyg.internal.utils.TextRangeHelper
import io.element.android.wysiwyg.internal.viewmodel.EditorInputAction
import io.element.android.wysiwyg.internal.viewmodel.EditorViewModel
import io.element.android.wysiwyg.internal.viewmodel.ReplaceTextResult
import io.element.android.wysiwyg.utils.EditorIndexMapper
import io.element.android.wysiwyg.utils.HtmlToSpansParser.FormattingSpans.removeFormattingSpans
import kotlin.math.abs
import kotlin.math.max
import kotlin.math.min

internal class InterceptInputConnection(
    baseInputConnection: InputConnection,
    editorEditText: TextView,
    private val viewModel: EditorViewModel,
    private val textWatcher: EditorTextWatcher,
) : InputConnectionWrapper(baseInputConnection, true) {
    private val editable = editorEditText.editableText

    init {
        textWatcher.updateCallback = { updatedText, start, end, previousText ->
            replaceTextInternal(start, end, updatedText, previousText)
        }
    }

    /**
     * Hack to have keyboard input events work as IME ones.
     */
    fun sendHardwareKeyboardInput(keyEvent: KeyEvent) {
        when {
            keyEvent.isPrintingKey || keyEvent.keyCode == KeyEvent.KEYCODE_SPACE -> {
                onHardwareCharacterKey(Char(keyEvent.unicodeChar).toString())
            }
            keyEvent.keyCode == KeyEvent.KEYCODE_ENTER -> {
                onHardwareEnterKey()
            }
            keyEvent.keyCode == KeyEvent.KEYCODE_DEL -> {
                onHardwareBackspaceKey()
            }
            keyEvent.keyCode == KeyEvent.KEYCODE_FORWARD_DEL ->
                onHardwareDeleteKey()
        }
    }

    // Called when started typing
    override fun setComposingText(text: CharSequence, newCursorPosition: Int): Boolean {
        val (start, end) = getCurrentCompositionOrSelection()
        val result = processTextEntry(text, start, end, null)

        return if (result != null) {
            beginBatchEdit()
            val newStart = start.coerceIn(0, result.text.length)
            val newEnd = (newStart + text.length).coerceIn(newStart, result.text.length)

            replaceAll(charSequence = result.text, start = start, end = end, newEnd = newEnd)
            val editorSelectionIndex = editorIndex(result.selection.last, editable)
            setSelection(editorSelectionIndex, editorSelectionIndex)
            setComposingRegion(newStart, newEnd)
            endBatchEdit()
            true
        } else {
            super.setComposingText(text, newCursorPosition)
        }
    }

    // Called for suggestion from IME selected
    override fun commitText(text: CharSequence, newCursorPosition: Int): Boolean {
        val (start, end) = getCurrentCompositionOrSelection()
        return replaceTextInternal(start, end, text, null)
    }

    // In Android 13+, this is called instead of [commitText] when selecting suggestions from IME
    override fun replaceText(
        start: Int,
        end: Int,
        text: CharSequence,
        newCursorPosition: Int,
        textAttribute: TextAttribute?
    ): Boolean {
        return replaceTextInternal(start, end, text, null)
    }

    private fun replaceTextInternal(
        start: Int,
        end: Int,
        text: CharSequence,
        oldText: CharSequence?,
    ): Boolean {
        val result = processTextEntry(text, start, end, oldText?.toString())

        return if (result != null) {
            beginBatchEdit()
            replaceAll(charSequence = result.text, start = start, end = end, newEnd = start + text.length)
            val editorSelectionIndex = editorIndex(result.selection.last, editable)
            setSelection(editorSelectionIndex, editorSelectionIndex)
            setComposingRegion(editorSelectionIndex, editorSelectionIndex)
            endBatchEdit()
            true
        } else {
            false
        }
    }

    private fun processTextEntry(newText: CharSequence?, start: Int, end: Int, previousText: String?): ReplaceTextResult? {
        val actualPreviousText = previousText ?: editable.substring(start until end)
        return withProcessor {
            when {
                // Special case for whitespace, to keep the formatting status we need to add it first
                newText != null && newText.length > 1 && newText.lastOrNull() == ' ' -> {
                    val toAppend = newText.substring(0 until newText.length - 1)
                    val (cStart, cEnd) = EditorIndexMapper.fromEditorToComposer(start, end, editable)
                        ?: error("Invalid indexes in composer $start, $end")
                    // First add whitespace
                    var result = processInput(EditorInputAction.ReplaceTextIn(cEnd, cEnd, " "))
                    // Then replace text if needed
                    if (toAppend != actualPreviousText) {
                        result = processInput(EditorInputAction.ReplaceTextIn(cStart, cEnd, toAppend))?.let {
                            // Fix selection to include whitespace at the end
                            val prevSelection = it.selection
                            it.copy(selection = prevSelection.first until prevSelection.last + 2)
                        }
                    }
                    result
                }
                // This only happens when a new line key stroke is received
                newText?.lastOrNull() == '\n' -> {
                    processInput(EditorInputAction.InsertParagraph)
                }
                actualPreviousText.isNotEmpty() && newText?.startsWith(actualPreviousText) == true -> {
                    // Appending new text at the end
                    val pos = end - start
                    val diff = newText.length - actualPreviousText.length
                    val toAppend = newText.substring(pos until pos + diff)
                    val (_, cEnd) = EditorIndexMapper.fromEditorToComposer(start, end, editable)
                        ?: error("Invalid indexes in composer $start, $end")
                    processInput(EditorInputAction.ReplaceTextIn(cEnd, cEnd, toAppend))
                }
                newText != null && actualPreviousText.startsWith(newText) -> {
                    // Removing text from the end
                    val diff = actualPreviousText.length - newText.length
                    val pos = end - diff
                    val (cStart, cEnd) = EditorIndexMapper.fromEditorToComposer(pos, end, editable)
                        ?: error("Invalid indexes in composer $pos, $end")
                    processInput(EditorInputAction.ReplaceTextIn(cStart, cEnd, ""))
                }
                else -> {
                    // New composing text
                    val (cStart, cEnd) = EditorIndexMapper.fromEditorToComposer(start, end, editable)
                        ?: error("Invalid indexes in composer $start, $end")
                    processInput(EditorInputAction.ReplaceTextIn(cStart, cEnd, newText.toString()))
                }
            }
        }
    }

    @VisibleForTesting(otherwise = VisibleForTesting.PRIVATE)
    internal fun onHardwareBackspaceKey(): Boolean {
        val start = Selection.getSelectionStart(editable)
        val end = Selection.getSelectionEnd(editable)

        val toDelete = if (start == end) 1 else abs(start - end)
        // We're going to copy backspace behaviour, the selection must be at the greater value
        val deletePos = max(start, end)

        // Imitate the software key backspace which updates the selection start to match the end.
        Selection.setSelection(editable, deletePos, deletePos)

        return deleteSurroundingText(toDelete, 0)
    }

    @VisibleForTesting(otherwise = VisibleForTesting.PRIVATE)
    internal fun onHardwareDeleteKey(): Boolean {
        val start = Selection.getSelectionStart(editable)
        val end = Selection.getSelectionEnd(editable)
        if (start > editable.count() || end > editable.count()) return false

        val toDelete = if (start == end) 1 else abs(start - end)

        // Imitate the software key backspace which updates the selection start to match the end.
        Selection.setSelection(editable, start, start)

        return deleteSurroundingText(0, toDelete)
    }

    @VisibleForTesting(otherwise = VisibleForTesting.PRIVATE)
    internal fun onHardwareEnterKey(): Boolean {
        val selectionStart = Selection.getSelectionStart(editable)
        val selectionEnd = Selection.getSelectionEnd(editable)

        val (compositionStart, compositionEnd) = getCurrentCompositionOrSelection()

        val newText = if(selectionStart == selectionEnd && selectionStart == compositionEnd) {
            val oldText = editable.subSequence(compositionStart until compositionEnd)
            "$oldText\n"
        } else
            "\n"

        return commitText(newText, 1)
    }

    private fun onHardwareCharacterKey(newChars: String): Boolean {
        // The current composition may not be up to date at this point so we do not attempt to
        // append to the current composition with the hardware keyboard.
        finishComposingText()

        // We previously tried to have hardware keys behave as composing text, but it's not
        // possible, we run into issues where existing keyboards (i.e. Trime) send empty composing
        // text and that breaks the input. So we need to use `commitText` instead.
        return commitText(newChars, 1)
    }

    override fun deleteSurroundingText(beforeLength: Int, afterLength: Int): Boolean {
        if (beforeLength == 0 && afterLength == 0) return false
        val start = Selection.getSelectionStart(editable)
        val end = Selection.getSelectionEnd(editable)

        var handled = false
        beginBatchEdit()
        if (afterLength > 0) {
            val (deleteFrom, deleteTo) =
                TextRangeHelper.extendRangeToReplacementSpans(
                    editable, start = end, length = afterLength
                )

            val result = withProcessor {
                val action = if (deleteTo - deleteFrom > 1) {
                    EditorInputAction.DeleteIn(deleteFrom, deleteTo)
                } else {
                    EditorInputAction.Delete
                }
                processInput(action)
            }
            if (result != null) {
                replaceAll(result.text, start = end, end = end + afterLength, newEnd = end)
                val editorSelectionIndex = editorIndex(result.selection.first, editable)
                setSelection(editorSelectionIndex, editorSelectionIndex)
                setComposingRegion(editorSelectionIndex, editorSelectionIndex)
            }
            // TODO: handle result == null
            handled = true
        }

        if (beforeLength > 0) {
            val (deleteFrom, deleteTo) =
                TextRangeHelper.extendRangeToReplacementSpans(
                    editable, start = start - beforeLength, length = beforeLength
                )

            val result = withProcessor {
                if (deleteTo - deleteFrom > 1) {
                    updateSelection(editable, deleteFrom, deleteTo)
                }
                processInput(EditorInputAction.BackPress)
            }
            if (result != null) {
                replaceAll(result.text, start = start - beforeLength, end = start, newEnd = start - beforeLength)
                val editorSelectionIndex = editorIndex(result.selection.first, editable)
                setSelection(editorSelectionIndex, editorSelectionIndex)
                setComposingRegion(editorSelectionIndex, editorSelectionIndex)
            }
            // TODO: handle result == null
            handled = true
        }
        endBatchEdit()

        return handled
    }

    private fun getCurrentCompositionOrSelection(): Pair<Int, Int> {
        val content = editable
        var start = getComposingSpanStart(content)
        var end = getComposingSpanEnd(content)

        if (start == -1 && end == -1) {
            start = Selection.getSelectionStart(content)
            end = Selection.getSelectionEnd(content)
        }

        // If order is inverted, swap them
        if (start > end) {
            start = end.also { end = start }
        }

        return start to end
    }

    private fun replaceAll(
        charSequence: CharSequence,
        start: Int = 0,
        end: Int = editable.length,
        newEnd: Int = charSequence.length
    ) {
        val clampedAfterCount = (newEnd.coerceAtMost(charSequence.length) - start).coerceAtLeast(0)
        textWatcher.runInEditor {
            notifyBeforeTextChanged(editable, start, end - start, clampedAfterCount)
            editable.removeFormattingSpans()
            editable.clear()
            editable.append(charSequence)
            notifyOnTextChanged(editable, start, end - start, clampedAfterCount)
            notifyAfterTextChanged(editable)
        }
    }

    private fun editorIndex(composerIndex: Int, editable: Editable): Int {
        return min(EditorIndexMapper.editorIndexFromComposer(composerIndex, editable), editable.length)
    }

    private fun <T> withProcessor(block: EditorViewModel.() -> T): T {
        return viewModel.run(block)
    }
}

package io.element.android.wysiwyg

import android.os.Build
import android.os.Bundle
import android.text.Editable
import android.text.Selection
import android.text.Spannable
import android.text.style.BackgroundColorSpan
import android.view.KeyEvent
import android.view.inputmethod.*
import android.widget.TextView
import androidx.annotation.RequiresApi
import androidx.annotation.VisibleForTesting
import androidx.core.text.getSpans
import io.element.android.wysiwyg.spans.HtmlToSpansParser
import kotlin.math.abs
import kotlin.math.max
import kotlin.math.min

class InterceptInputConnection(
    private val baseInputConnection: InputConnection,
    private val editorEditText: TextView,
    private val inputProcessor: InputProcessor,
) : BaseInputConnection(editorEditText, true) {

    init {
        // TODO: remove this once we have an initial menu state update
        inputProcessor.processInput(EditorInputAction.InsertText(""))
    }

    override fun getEditable(): Editable {
        return editorEditText.editableText
    }

    override fun beginBatchEdit(): Boolean {
        return baseInputConnection.beginBatchEdit()
    }

    override fun endBatchEdit(): Boolean {
        return baseInputConnection.endBatchEdit()
    }

    @RequiresApi(Build.VERSION_CODES.N)
    override fun closeConnection() {
        super.closeConnection()
        baseInputConnection.closeConnection()
    }

    override fun clearMetaKeyStates(states: Int): Boolean {
        return baseInputConnection.clearMetaKeyStates(states)
    }

    override fun sendKeyEvent(event: KeyEvent?): Boolean {
        return super.sendKeyEvent(event)
    }

    override fun commitCompletion(text: CompletionInfo?): Boolean {
        return baseInputConnection.commitCompletion(text)
    }

    override fun commitCorrection(correctionInfo: CorrectionInfo?): Boolean {
        return baseInputConnection.commitCorrection(correctionInfo)
    }

    override fun performEditorAction(actionCode: Int): Boolean {
        return baseInputConnection.performEditorAction(actionCode)
    }

    override fun performContextMenuAction(id: Int): Boolean {
        return baseInputConnection.performContextMenuAction(id)
    }

    override fun getExtractedText(request: ExtractedTextRequest?, flags: Int): ExtractedText {
        return baseInputConnection.getExtractedText(request, flags)
    }

    override fun performPrivateCommand(action: String?, data: Bundle?): Boolean {
        return baseInputConnection.performPrivateCommand(action, data)
    }

    override fun setImeConsumesInput(imeConsumesInput: Boolean): Boolean {
        baseInputConnection.setImeConsumesInput(imeConsumesInput)
        return super.setImeConsumesInput(imeConsumesInput)
    }


    fun processKeyEvent(keyEvent: KeyEvent) {
        val content = editable
        val start = Selection.getSelectionStart(content)
        val end = Selection.getSelectionEnd(content)
        val (cStart, cEnd) = getCurrentComposition()
        when {
            keyEvent.isPrintingKey || keyEvent.keyCode == KeyEvent.KEYCODE_SPACE -> {
                withProcessor {
                    updateSelection(editable, start, end)
                    val newText = if (keyEvent.keyCode == KeyEvent.KEYCODE_SPACE) {
                        " "
                    } else {
                        Char(keyEvent.unicodeChar).toString()
                    }
                    val result = processInput(EditorInputAction.InsertText(newText))?.let {
                        processUpdate(it)
                    }
                    val selectionLength = end - start
                    beginBatchEdit()
                    if (result != null) {
                        editable.clear()
                        editable.replace(0, editable.length, result.text)
                    } else {
                        editable.replace(start, end, newText)
                    }
                    setComposingRegion(cStart, cEnd - selectionLength)
                    setSelectionOnEditable(editable, start + 1)
                    endBatchEdit()
                }
            }
            keyEvent.keyCode == KeyEvent.KEYCODE_ENTER -> {
                val result = withProcessor {
                    processInput(EditorInputAction.InsertParagraph)?.let {
                        processUpdate(it)
                    }
                }
                beginBatchEdit()
                if (result != null) {
                    editable.replace(0, editable.length, result.text)
                    setSelectionOnEditable(editable, result.selection.last)
                } else {
                    editable.replace(start, end, "\n")
                }
                endBatchEdit()
            }
            keyEvent.keyCode == KeyEvent.KEYCODE_DEL -> {
                inputProcessor.updateSelection(editable, start, end)
                backspace()
            }
        }
    }

    // Called when started typing
    override fun setComposingText(text: CharSequence?, newCursorPosition: Int): Boolean {
        val (start, end) = getCurrentComposition()
        inputProcessor.updateSelection(editable, start, end)
        val result = withProcessor {
            processInput(EditorInputAction.InsertText(text.toString()))?.let { processUpdate(it) }
        }

        return if (result != null) {
            val compositionEnd = text?.length?.let { it + start } ?: end
            // Here we restore the background color spans from the IME input. This seems to be
            // important for Japanese input.
            if (text is Spannable && result.text is Spannable) {
                copyImeHighlightSpans(text, result.text, start)
            }
            replaceAll(result.text, compositionStart = start, compositionEnd = compositionEnd, newCursorPosition)
            setSelectionOnEditable(editable, result.selection.last, result.selection.last)
            true
        } else {
            super.setComposingText(text, newCursorPosition)
        }
    }

    // Called for suggestion from IME selected
    override fun commitText(text: CharSequence?, newCursorPosition: Int): Boolean {
        val (start, end) = getCurrentComposition()
        val result = withProcessor {
            if (text?.lastOrNull() == '\n') {
                processInput(EditorInputAction.InsertParagraph)
            } else {
                inputProcessor.updateSelection(editable, start, end)
                processInput(EditorInputAction.InsertText(text.toString()))
            }?.let { processUpdate(it) }
        }

        return if (result != null) {
            replaceAll(result.text, compositionStart = end, compositionEnd = end, newCursorPosition)
            setSelectionOnEditable(editable, result.selection.last, result.selection.last)
            true
        } else {
            super.commitText(text, newCursorPosition)
        }
    }

    @VisibleForTesting(otherwise = VisibleForTesting.PRIVATE)
    internal fun backspace(): Boolean {
        val start = Selection.getSelectionStart(editable)
        val end = Selection.getSelectionEnd(editable)
        if (start == 0 && end == 0) return false

        val result = withProcessor {
            updateSelection(editable, start, end)
            processInput(EditorInputAction.BackPress)?.let { processUpdate(it) }
        }

        return if (result != null) {
            val newSelection = if (start == end) end-1 else min(start, end)
            beginBatchEdit()
            editable.replace(0, editable.length, result.text)
            setSelectionOnEditable(editable, newSelection)
            endBatchEdit()
            true
        } else {
            // Workaround for keyboard input
            val maxValue = max(end, start)
            setSelectionOnEditable(editable, maxValue, maxValue)
            val toDelete = if (start == end) 1 else abs(start - end)
            super.deleteSurroundingText(toDelete, 0)
        }
    }

    override fun deleteSurroundingText(beforeLength: Int, afterLength: Int): Boolean {
        if (beforeLength == 0 && afterLength == 0) return false
        val start = Selection.getSelectionStart(editable)
        val end = Selection.getSelectionEnd(editable)
        val deleteFrom = (start-beforeLength).coerceAtLeast(0)
        val deleteTo = end + afterLength

        var handled = false
        beginBatchEdit()
        if (afterLength > 0) {
            val result = withProcessor {
                updateSelection(editable, end, deleteTo)
                processInput(EditorInputAction.BackPress)?.let { processUpdate(it) }
            }
            if (result != null) {
                replaceAll(result.text, 0, editable.length, 1)
                setSelectionOnEditable(editable, result.selection.first, result.selection.last)
                setComposingRegion(result.selection.first, result.selection.last)
            }
            // TODO: handle result == null
            handled = true
        }

        if (beforeLength > 0) {
            val result = withProcessor {
                updateSelection(editable, deleteFrom, start)
                processInput(EditorInputAction.BackPress)?.let { processUpdate(it) }
            }
            if (result != null) {
                replaceAll(result.text, 0, editable.length, 1)
                setSelectionOnEditable(editable, result.selection.first, result.selection.last)
                setComposingRegion(result.selection.first, result.selection.last)
            }
            // TODO: handle result == null
            handled = true
        }
        endBatchEdit()

        return handled
    }

    override fun requestCursorUpdates(cursorUpdateMode: Int): Boolean {
        return baseInputConnection.requestCursorUpdates(cursorUpdateMode)
    }

    fun getCurrentComposition(): Pair<Int, Int> {
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

    private fun replaceAll(charSequence: CharSequence, compositionStart: Int, compositionEnd: Int, newCursorPosition: Int) {
        beginBatchEdit()
        editable.clear()
        editable.replace(0, editable.length, charSequence)
        setComposingRegion(compositionStart, compositionEnd)
        endBatchEdit()
    }

    private fun copyImeHighlightSpans(from: Spannable, to: Spannable, offset: Int) {
        val highlightSpans = from.getSpans(0, from.count(), BackgroundColorSpan::class.java)
            .orEmpty()
        for (span in highlightSpans) {
            val spanStart = from.getSpanStart(span) + offset
            val spanEnd = from.getSpanEnd(span) + offset
            to.setSpan(span, spanStart, spanEnd, 0)
        }
    }

    private fun setSelectionOnEditable(editable: Editable, start: Int, end: Int = start) {
        val newStart = min(composerIndexInEditable(start), editable.length)
        val newEnd = min(composerIndexInEditable(end), editable.length)
        Selection.setSelection(editable, newStart, newEnd)
    }

    private fun composerIndexInEditable(index: Int): Int {
        val editable = this.editable
        var consumed = 0
        var i = 0
        while (index > consumed) {
            val spans = editable.getSpans<HtmlToSpansParser.ZeroWidthLineBreak>(start = i, end = i+1)
            if (spans.isEmpty()) {
                consumed++
                i++
            } else {
                i += spans.count()
            }
        }
        return i
    }

    private fun <T> withProcessor(block: InputProcessor.() -> T): T {
        return inputProcessor.run(block)
    }
}

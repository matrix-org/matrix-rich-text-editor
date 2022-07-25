package io.element.android.wysiwygpoc

import android.content.Context
import android.os.Bundle
import android.text.Editable
import android.text.Selection
import android.view.KeyEvent
import android.view.inputmethod.*
import android.widget.TextView
import androidx.core.widget.addTextChangedListener
import kotlin.math.abs

class InterceptInputConnection(
    private val editorEditText: TextView
) : BaseInputConnection(editorEditText, true) {

    private var batchEditNesting = 0
    val inputProcessor = InputProcessor(uniffi.wysiwyg_composer.newComposerModel())

    private var expectedEditable: CharSequence? = null

    init {
        // Used to try to catch spell checker actions. Sadly, this would only work if we always
        // used TextUpdate.ReplaceAll instead of TextUpdate.Keep.
        editorEditText.addTextChangedListener(onTextChanged = { text, start, before, count ->
            println("Changed: $text | $expectedEditable")
            if (expectedEditable != null && text.contentEquals(expectedEditable)) return@addTextChangedListener
            println("Actually changed.")
        })
    }

    override fun getEditable(): Editable {
        return editorEditText.editableText
    }

    private val inputMethodManager: InputMethodManager? = editorEditText.context.getSystemService(Context.INPUT_METHOD_SERVICE) as? InputMethodManager

    override fun beginBatchEdit(): Boolean {
        synchronized(this) {
            if (batchEditNesting >= 0) {
                editorEditText.beginBatchEdit()
                batchEditNesting++
                return true
            }
        }
        return false
    }

    override fun endBatchEdit(): Boolean {
        synchronized(this) {
            if (batchEditNesting > 0) {
                // When the connection is reset by the InputMethodManager and reportFinish
                // is called, some endBatchEdit calls may still be asynchronously received from the
                // IME. Do not take these into account, thus ensuring that this IC's final
                // contribution to mTextView's nested batch edit count is zero.
                editorEditText.endBatchEdit()
                batchEditNesting--
                return true
            }
        }
        return false
    }

    override fun closeConnection() {
        super.closeConnection()

        synchronized(this) {
            while (batchEditNesting > 0) {
                endBatchEdit()
            }
            // Will prevent any further calls to begin or endBatchEdit
            batchEditNesting--
        }
    }

    override fun clearMetaKeyStates(states: Int): Boolean {
        val content = editorEditText.editableText ?: return false
        val keyListener = editorEditText.keyListener
        try {
            keyListener?.clearMetaKeyState(editorEditText, content, states)
        } catch (e: AbstractMethodError) {
            // This is an old listener that doesn't implement the
            // new method.
        }
        return true
    }

    override fun sendKeyEvent(event: KeyEvent?): Boolean {
        return super.sendKeyEvent(event)
    }

    override fun commitCompletion(text: CompletionInfo?): Boolean {
        with (editorEditText) {
            beginBatchEdit()
            onCommitCompletion(text)
            endBatchEdit()
        }
        return true
    }

    override fun commitCorrection(correctionInfo: CorrectionInfo?): Boolean {
        with (editorEditText) {
            beginBatchEdit()
            onCommitCorrection(correctionInfo)
            endBatchEdit()
        }
        return true
    }

    override fun performEditorAction(actionCode: Int): Boolean {
        editorEditText.onEditorAction(actionCode)
        return true
    }

    override fun performContextMenuAction(id: Int): Boolean {
        with (editorEditText) {
            beginBatchEdit()
            onTextContextMenuItem(id)
            endBatchEdit()
        }
        return true
    }

    override fun getExtractedText(request: ExtractedTextRequest?, flags: Int): ExtractedText {
        val text = ExtractedText()
        editorEditText.extractText(request, text)
        return text
    }

    override fun performPrivateCommand(action: String?, data: Bundle?): Boolean {
        editorEditText.onPrivateIMECommand(action, data)
        return true
    }

    /**
     * Hack to have keyboard input events work as IME ones.
     */
    fun sendHardwareKeyboardInput(keyEvent: KeyEvent) {
        val char = Char(keyEvent.unicodeChar)
        val isNotPrintable = char == Char(0)
        val start = Selection.getSelectionStart(editable)
        val end = Selection.getSelectionEnd(editable)
        val (cStart, cEnd) = getCurrentComposition()
        if (cStart != -1 && cEnd != -1) {
            removeComposingSpans(editable)
        }
        when {
            isNotPrintable -> {
                if (keyEvent.keyCode == KeyEvent.KEYCODE_DEL) {
                    inputProcessor.updateSelection(start, end)
                    backspace()
                }
            }
            else -> {
                // Replace selection
                if (abs(end - start) > 1) {
                    val newText = editable.subSequence(start, end) as Editable
                    newText.replace(0, end - start, char.toString())
                    setComposingRegion(start, end)
                    setComposingText(newText, 1)
                } else {
                    val newText = editable.subSequence(cStart, cEnd) as Editable
                    newText.insert(cEnd - cStart, char.toString())
                    setComposingRegion(cStart, end)
                    if (newText.toString() != " ") {
                        setComposingText(newText, 1)
                    } else {
                        commitText(newText, 1)
                    }
                }
            }
        }
    }

    // Called when started typing
    override fun setComposingText(text: CharSequence?, newCursorPosition: Int): Boolean {
        val (start, end) = getCurrentComposition()
        inputProcessor.updateSelection(start, end)
        val textUpdate = inputProcessor.processInput(EditorInputAction.InsertText(text.toString()))
        val result = textUpdate?.let { inputProcessor.processUpdate(it) }

//        return super.setComposingText(text, newCursorPosition)

        return if (result != null) {
            replaceAll(result, newCursorPosition)
            true
        } else {
            super.setComposingText(text, newCursorPosition)
        }
    }

    // Called for suggestion from IME selected
    override fun commitText(text: CharSequence?, newCursorPosition: Int): Boolean {
        val (start, end) = getCurrentComposition()
        inputProcessor.updateSelection(start, end)
        val textUpdate = inputProcessor.processInput(EditorInputAction.InsertText(text.toString()))
        val result = textUpdate?.let { inputProcessor.processUpdate(it) }

//        return super.commitText(text, newCursorPosition)

        return if (result != null) {
            replaceAll(result, newCursorPosition)
            true
        } else {
            super.commitText(text, newCursorPosition)
        }
    }

    private fun backspace(): Boolean {
        val start = Selection.getSelectionStart(editable)
        val end = Selection.getSelectionEnd(editable)
        if (start == 0 && end == 0) return false

        val textUpdate = inputProcessor.processInput(EditorInputAction.BackPress)
        val result = textUpdate?.let { inputProcessor.processUpdate(it) }

        return if (result != null) {
            replaceAll(result, newCursorPosition = 1)
            true
        } else {
            // Workaround for keyboard input
            if (end != start) {
                setSelection(end, end)
            }
            super.deleteSurroundingText(end - start, 0)
        }
    }

    override fun deleteSurroundingText(beforeLength: Int, afterLength: Int): Boolean {
        val end = Selection.getSelectionEnd(editable)
        val start = end - beforeLength + afterLength

        inputProcessor.updateSelection(start, end)

        return backspace()
    }

    fun applyInlineFormat(format: InlineFormat) {
        val start = Selection.getSelectionStart(editable)
        val end = Selection.getSelectionEnd(editable)
        inputProcessor.updateSelection(start, end)

        val update = inputProcessor.processInput(EditorInputAction.ApplyInlineFormat(format))
        val result = update?.let { inputProcessor.processUpdate(it) }

        result?.let { replaceAll(result, newCursorPosition = 0) }
    }

    override fun requestCursorUpdates(cursorUpdateMode: Int): Boolean {
        // It is possible that any other bit is used as a valid flag in a future release.
        // We should reject the entire request in such a case.
        val knownFlagsMask = InputConnection.CURSOR_UPDATE_IMMEDIATE or InputConnection.CURSOR_UPDATE_MONITOR
        val unknownFlags = cursorUpdateMode and knownFlagsMask.inv()

        if (unknownFlags != 0) {
            return false
        }

        if (inputMethodManager == null) {
            return false
        }

        if (cursorUpdateMode and InputConnection.CURSOR_UPDATE_IMMEDIATE != 0) {
            if (!editorEditText.isInLayout) {
                editorEditText.requestLayout()
            }
        }
        return true
    }

    private fun getCurrentComposition(): Pair<Int, Int> {
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

    private fun replaceAll(charSequence: CharSequence, newCursorPosition: Int) {
        beginBatchEdit()
        updateSelectionInternal(newCursorPosition)
        expectedEditable = charSequence
        editable.replace(0, editable.length, charSequence)
        endBatchEdit()
    }

    private fun updateSelectionInternal(newCursorPosition: Int) {
        val (start, end) = getCurrentComposition()
        val content = editable
        var cursorPosition = newCursorPosition
        cursorPosition += if (newCursorPosition > 0) end else start
        cursorPosition = cursorPosition.coerceIn(0, content.length)
        Selection.setSelection(content, cursorPosition)
    }
}

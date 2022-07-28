package io.element.android.wysiwyg

import android.content.Context
import android.os.Bundle
import android.text.Editable
import android.text.Selection
import android.text.Spannable
import android.text.style.BackgroundColorSpan
import android.view.KeyEvent
import android.view.inputmethod.*
import android.widget.TextView
import androidx.annotation.VisibleForTesting
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.Job
import kotlinx.coroutines.channels.Channel
import kotlinx.coroutines.channels.consumeEach
import kotlinx.coroutines.launch
import kotlin.math.abs
import kotlin.math.max
import kotlin.math.min

class InterceptInputConnection(
    private val baseInputConnection: InputConnection,
    private val editorEditText: TextView,
    private val inputProcessor: InputProcessor,
) : BaseInputConnection(editorEditText, true) {

    private val keyboardEventQueue = Channel<KeyEvent>(capacity = Channel.UNLIMITED)

    private var keyEventJob: Job? = null

    init {
        keyEventJob = processKeyEvents()
        // Used to try to catch spell checker actions. Sadly, this would only work if we always
        // used TextUpdate.ReplaceAll instead of TextUpdate.Keep.
//        editorEditText.addTextChangedListener(onTextChanged = { text, start, before, count ->
//            println("Changed: $text | $expectedEditable")
//            if (expectedEditable != null && text.contentEquals(expectedEditable)) return@addTextChangedListener
//            println("Actually changed.")
//        })
    }

    override fun getEditable(): Editable {
        return editorEditText.editableText
    }

    private val inputMethodManager: InputMethodManager? = editorEditText.context.getSystemService(Context.INPUT_METHOD_SERVICE) as? InputMethodManager

    override fun beginBatchEdit(): Boolean {
        return baseInputConnection.beginBatchEdit()
    }

    override fun endBatchEdit(): Boolean {
        return baseInputConnection.endBatchEdit()
    }

    override fun closeConnection() {
        super.closeConnection()
        baseInputConnection.closeConnection()

        keyEventJob?.cancel()
        keyEventJob = null
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

    /**
     * Hack to have keyboard input events work as IME ones.
     */
    fun sendHardwareKeyboardInput(keyEvent: KeyEvent) {
        keyboardEventQueue.trySend(keyEvent)
    }

    private fun processKeyEvents() =
        CoroutineScope(Dispatchers.Main).launch {
            keyboardEventQueue.consumeEach { keyEvent ->
                val content = editable
                val start = Selection.getSelectionStart(content)
                val end = Selection.getSelectionEnd(content)
                val (cStart, cEnd) = getCurrentComposition()
                when {
                    keyEvent.isPrintingKey || keyEvent.keyCode == KeyEvent.KEYCODE_SPACE -> {
                        withProcessor {
                            updateSelection(start, end)
                            val newText = if (keyEvent.keyCode == KeyEvent.KEYCODE_SPACE) {
                                " "
                            } else {
                                Char(keyEvent.unicodeChar).toString()
                            }
                            val result = processInput(EditorInputAction.InsertText(newText))?.let {
                                processUpdate(it)
                            }
                            val selectionLength = end-start
                            beginBatchEdit()
                            if (result != null) {
                                editable.replace(0, editable.length, result)
                            } else {
                                editable.replace(start, end, newText)
                            }
                            setComposingRegion(cStart, cEnd - selectionLength)
                            Selection.setSelection(editable, start+1)
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
                            editable.replace(0, editable.length, result)
                        } else {
                            editable.replace(start, end, "\n")
                        }
                        endBatchEdit()
                    }
                    keyEvent.keyCode == KeyEvent.KEYCODE_DEL -> {
                        inputProcessor.updateSelection(start, end)
                        backspace()
                    }
                }
            }
        }

    // Called when started typing
    override fun setComposingText(text: CharSequence?, newCursorPosition: Int): Boolean {
        val (start, end) = getCurrentComposition()
        inputProcessor.updateSelection(start, end)
        val result = withProcessor {
            processInput(EditorInputAction.InsertText(text.toString()))?.let { processUpdate(it) }
        }

        return if (result != null) {
            val compositionEnd = text?.length?.let { it + start } ?: end
            // Here we restore the background color spans from the IME input. This seems to be
            // important for Japanese input.
            if (text is Spannable && result is Spannable) {
                copyImeHighlightSpans(text, result, start)
            }
            replaceAll(result, newCursorPosition)
            setComposingRegion(start, compositionEnd)
            true
        } else {
            super.setComposingText(text, newCursorPosition)
        }
    }

    // Called for suggestion from IME selected
    override fun commitText(text: CharSequence?, newCursorPosition: Int): Boolean {
        val (start, end) = getCurrentComposition()
        inputProcessor.updateSelection(start, end)
        val result = withProcessor {
            if (text.contentEquals("\n")) {
                processInput(EditorInputAction.InsertParagraph)
            } else {
                processInput(EditorInputAction.InsertText(text.toString()))
            }?.let { processUpdate(it) }
        }

        return if (result != null) {
            replaceAll(result, newCursorPosition)
            setComposingRegion(end, end)
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
            updateSelection(start, end)
            processInput(EditorInputAction.BackPress)?.let { processUpdate(it) }
        }

        return if (result != null) {
            val newSelection = if (start == end) end-1 else min(start, end)
            beginBatchEdit()
            editable.replace(0, editable.length, result)
            Selection.setSelection(editable, newSelection)
            endBatchEdit()
            true
        } else {
            // Workaround for keyboard input
            val maxValue = max(end, start)
            setSelection(maxValue, maxValue)
            val toDelete = if (start == end) 1 else abs(start - end)
            super.deleteSurroundingText(toDelete, 0)
        }
    }

    // FIXME: it's not working as intended
    override fun deleteSurroundingText(beforeLength: Int, afterLength: Int): Boolean {
        if (beforeLength == 0 && afterLength == 0) return false
        val end = Selection.getSelectionEnd(editable)
        val start = end - beforeLength + afterLength
        val deleteFrom = (start - beforeLength).coerceAtLeast(0)
        val deleteTo = end + afterLength

        var handled = false
        beginBatchEdit()
        if (afterLength > 0) {
            val result = withProcessor {
                updateSelection(end, deleteTo)
                processInput(EditorInputAction.BackPress)?.let { processUpdate(it) }
            }
            if (result != null) {
                editable.replace(0, editable.length, result)
            }
            // TODO: handle result == null
            handled = true
        }

        if (beforeLength > 0) {
            val result = withProcessor {
                updateSelection(deleteFrom, start)
                processInput(EditorInputAction.BackPress)?.let { processUpdate(it) }
            }
            if (result != null) {
                editable.replace(0, editable.length, result)
            }
            // TODO: handle result == null
            handled = true
        }
        endBatchEdit()

        return handled
    }

    fun applyInlineFormat(format: InlineFormat) {
        val start = Selection.getSelectionStart(editable)
        val end = Selection.getSelectionEnd(editable)
        withProcessor { updateSelection(start, end) }

        val result = withProcessor {
            processInput(EditorInputAction.ApplyInlineFormat(format))?.let { processUpdate(it) }
        }

        result?.let { replaceAll(result, newCursorPosition = 0) }
    }

    override fun requestCursorUpdates(cursorUpdateMode: Int): Boolean {
        return baseInputConnection.requestCursorUpdates(cursorUpdateMode)
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

    private fun copyImeHighlightSpans(from: Spannable, to: Spannable, offset: Int) {
        val highlightSpans = from.getSpans(0, from.count(), BackgroundColorSpan::class.java)
            .orEmpty()
        for (span in highlightSpans) {
            val spanStart = from.getSpanStart(span) + offset
            val spanEnd = from.getSpanEnd(span) + offset
            to.setSpan(span, spanStart, spanEnd, 0)
        }
    }

    private fun <T> withProcessor(block: InputProcessor.() -> T): T {
        return inputProcessor.run(block)
    }
}

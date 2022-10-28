package io.element.android.wysiwyg.inputhandlers

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
import io.element.android.wysiwyg.inputhandlers.models.EditorInputAction
import io.element.android.wysiwyg.utils.EditorIndexMapper
import io.element.android.wysiwyg.utils.HtmlToSpansParser.FormattingSpans.removeFormattingSpans
import io.element.android.wysiwyg.viewmodel.EditorViewModel
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.Job
import kotlinx.coroutines.channels.Channel
import kotlinx.coroutines.channels.consumeEach
import kotlinx.coroutines.launch
import kotlin.math.abs
import kotlin.math.min

internal class InterceptInputConnection(
    private val baseInputConnection: InputConnection,
    private val editorEditText: TextView,
    private val viewModel: EditorViewModel,
) : BaseInputConnection(editorEditText, true) {

    private val keyboardEventQueue = Channel<KeyEvent>(capacity = Channel.UNLIMITED)

    private var keyEventJob: Job? = null

    init {
        keyEventJob = processKeyEvents()
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
        // This should be enough as it will internally call baseInputConnection methods anyway
        super.closeConnection()

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

    // Used for 'Extract UI' of EditText (aka: full screen EditText in landscape)
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

    /**
     * Hack to have keyboard input events work as IME ones.
     */
    fun sendHardwareKeyboardInput(keyEvent: KeyEvent) {
        keyboardEventQueue.trySend(keyEvent)
    }

    private fun processKeyEvents() =
        CoroutineScope(Dispatchers.Main).launch {
            keyboardEventQueue.consumeEach { keyEvent ->
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
                }
            }
        }

    // Called when started typing
    override fun setComposingText(text: CharSequence?, newCursorPosition: Int): Boolean {
        val (start, end) = getCurrentCompositionOrSelection()
        viewModel.updateSelection(editable, start, end)
        val result = withProcessor {
            processInput(EditorInputAction.ReplaceText(text.toString()))
        }

        return if (result != null) {
            val newText = result.text.subSequence(start, start + (text?.length ?: 0))

            // Calculate the new composition range.
            // If the composer has inserted a zero width whitespace as a list delimiter,
            // shift the composition indices so that it is not included.
            val compositionStart = start
                .let { if (newText.startsWith("\u200b")) it + 1 else it }
            val compositionEnd = (text?.length?.let { it + start } ?: end)
                .let { if (newText.startsWith("\u200b")) it + 1 else it }

            // Here we restore the background color spans from the IME input. This seems to be
            // important for Japanese input.
            if (text is Spannable && result.text is Spannable) {
                copyImeHighlightSpans(text, result.text, start)
            }
            replaceAll(result.text, compositionStart = compositionStart, compositionEnd = compositionEnd)
            setSelectionOnEditable(editable, result.selection.last, result.selection.last)
            true
        } else {
            super.setComposingText(text, newCursorPosition)
        }
    }

    // Called for suggestion from IME selected
    override fun commitText(text: CharSequence?, newCursorPosition: Int): Boolean {
        val (start, end) = getCurrentCompositionOrSelection()
        val result = withProcessor {
            if (text?.lastOrNull() == '\n') {
                processInput(EditorInputAction.InsertParagraph)
            } else {
                viewModel.updateSelection(editable, start, end)
                processInput(EditorInputAction.ReplaceText(text.toString()))
            }
        }

        return if (result != null) {
            replaceAll(result.text, compositionStart = end, compositionEnd = end)
            setSelectionOnEditable(editable, result.selection.last, result.selection.last)
            true
        } else {
            super.commitText(text, newCursorPosition)
        }
    }

    @VisibleForTesting(otherwise = VisibleForTesting.PRIVATE)
    internal fun onHardwareBackspaceKey(): Boolean {
        val start = Selection.getSelectionStart(editable)
        val end = Selection.getSelectionEnd(editable)
        if (start == 0 && end == 0) return false

        val toDelete = if (start == end) 1 else abs(start - end)

        // Imitate the software key backspace which updates the selection start to match the end.
        Selection.setSelection(editable, end, end)

        return deleteSurroundingText(toDelete, 0)
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

        return setComposingText(newChars, 1)
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
                processInput(EditorInputAction.BackPress)
            }
            if (result != null) {
                replaceAll(result.text, 0, editable.length)
                setSelectionOnEditable(editable, result.selection.first, result.selection.last)
                setComposingRegion(result.selection.first, result.selection.last)
            }
            // TODO: handle result == null
            handled = true
        }

        if (beforeLength > 0) {
            val result = withProcessor {
                updateSelection(editable, deleteFrom, start)
                processInput(EditorInputAction.BackPress)
            }
            if (result != null) {
                replaceAll(result.text, 0, editable.length)
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

    fun getCurrentCompositionOrSelection(): Pair<Int, Int> {
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
        compositionStart: Int,
        compositionEnd: Int,
    ) {
        beginBatchEdit()
        editable.removeFormattingSpans()
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
        val newStart = min(EditorIndexMapper.editorIndexFromComposer(start, editable), editable.length)
        val newEnd = min(EditorIndexMapper.editorIndexFromComposer(end, editable), editable.length)
        Selection.setSelection(editable, newStart, newEnd)
    }

    private fun <T> withProcessor(block: EditorViewModel.() -> T): T {
        return viewModel.run(block)
    }
}

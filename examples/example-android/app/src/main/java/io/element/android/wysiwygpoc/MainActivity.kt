package io.element.android.wysiwygpoc

import android.os.Bundle
import android.text.*
import androidx.appcompat.app.AppCompatActivity
import androidx.core.text.HtmlCompat
import io.element.android.wysiwygpoc.databinding.ActivityMainBinding
import kotlinx.coroutines.*
import kotlinx.coroutines.channels.Channel
import kotlinx.coroutines.channels.Channel.Factory.UNLIMITED
import kotlinx.coroutines.channels.consumeEach
import uniffi.wysiwyg_composer.ComposerModel
import uniffi.wysiwyg_composer.TextUpdate
import kotlin.coroutines.CoroutineContext

class MainActivity : AppCompatActivity() {

    private val composer: ComposerModel = uniffi.wysiwyg_composer.newComposerModel()
    private val inputProcessor = InputProcessor(composer, Dispatchers.IO)

    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        val binding = ActivityMainBinding.inflate(layoutInflater)
        setContentView(binding.root)

        val filter = InterceptInputFilter { inputProcessor.queue(it) }
        inputProcessor.start(binding.editor, filter)

        with (binding.editor) {
            filters += filter
            requestFocus()
            selectionChangeListener = EditorEditText.OnSelectionChangeListener { start, end ->
                composer.select(start.toUInt(), end.toUInt())
            }
//            addTextChangedListener(EditorTextWatcher(inputProcessor))
        }

        binding.buttonBold.setOnClickListener {
            inputProcessor.queue(
                EditorInputAction.ApplyInlineFormat(InlineFormat.Bold)
            )
//            result?.let { binding.editor.setText(it, TextView.BufferType.SPANNABLE) }
        }
    }

    override fun onDestroy() {
        super.onDestroy()

        inputProcessor.stop()
    }

    class InputProcessor(
        private val composer: ComposerModel,
        private val backgroundCoroutineContext: CoroutineContext,
    ) {

        private lateinit var editText: EditorEditText
        private lateinit var filter: InterceptInputFilter

        private val actionChannel = Channel<EditorInputAction>(capacity = UNLIMITED)
        private val updateChannel = Channel<TextUpdate>(capacity = UNLIMITED)
        private var actionJob: Job? = null
        private var updateJob: Job? = null

        fun start(editText: EditorEditText, filter: InterceptInputFilter) {
            this.editText = editText
            this.filter = filter

            actionJob = CoroutineScope(backgroundCoroutineContext).launch {
                actionChannel.consumeEach { action ->
                    val update = processInput(action)
                    update?.let { updateChannel.trySend(it) }
                }
            }
            updateJob = CoroutineScope(backgroundCoroutineContext).launch {
                updateChannel.consumeEach { update ->
                    processUpdate(update)
                }
            }
        }

        fun stop() {
            actionJob?.cancel()
            actionJob = null

            updateJob?.cancel()
            updateJob = null
        }

        fun queue(action: EditorInputAction) {
            actionChannel.trySend(action)
        }

        fun processInput(action: EditorInputAction): TextUpdate? {
            return when (action) {
                is EditorInputAction.InsertText -> {
                    // This conversion to a plain String might be too simple
                    composer.replaceTextIn(action.value.toString(), action.start.toULong(), action.end.toULong())
                }
                is EditorInputAction.InsertParagraph -> {
                    composer.enter()
                }
                is EditorInputAction.BackPress -> {
                    composer.backspace()
                }
                is EditorInputAction.ApplyInlineFormat -> {
                    when (action.format) {
                        is InlineFormat.Bold -> composer.bold()
                    }
                }
                is EditorInputAction.Delete -> {
                    composer.deleteIn(action.start.toULong(), action.end.toULong())
                }
                is EditorInputAction.ReplaceAll -> return null
            }.textUpdate()
        }

        private suspend fun processUpdate(update: TextUpdate) {
            when (update) {
                is TextUpdate.Keep -> return
                is TextUpdate.ReplaceAll -> {
                    val text = stringToSpans(update.replacementHtml.string())
                    withContext(Dispatchers.Main) {
                        val editableText = editText.editableText as SpannableStringBuilder
                        filter.isReentrant = true
                        editableText.replace(0, editableText.length, text)
                        editText.invalidate()
                        editText.requestLayout()
                    }
                }
            }
        }

        private fun stringToSpans(string: String): Spanned {
            // TODO: Check parsing flags
            val preparedString = string.replace(" ", "&nbsp;")
            return HtmlCompat.fromHtml(preparedString, 0)
        }
    }

//    class EditorTextWatcher(
//        private val inputProcessor: InputProcessor,
//    ) : TextWatcher {
//        private var replacement: CharSequence? = null
//
//        override fun beforeTextChanged(source: CharSequence?, start: Int, count: Int, after: Int) {}
//
//        override fun onTextChanged(source: CharSequence?, start: Int, before: Int, count: Int) {
//            // When we make any changes to the editor's text using `replacement` the TextWatcher
//            // will be called again. When this happens, clean `replacement` and just return.
//            if (replacement != null) {
//                replacement = null
//                return
//            }
//            // When all text is deleted, clean `replacement` and early return.
//            if (source == null) {
//                replacement = null
//                return
//            }
//
//            // TODO: instead of using `replaced` + `ReplaceAll`, add a new replace operation with
//            //  indexes in Rust to modify the underlying buffer. Otherwise, we're going to have to
//            //  fight the IME's autocorrect feature.
//            val replaced = source.substring(start until start+count)
//            when {
//                start == 0 && count == before -> {
//                    inputProcessor.queue(EditorInputAction.ReplaceAll(replaced))
//                }
//                before > count -> {
//                    if (before - count == 1) {
//                        inputProcessor.queue(EditorInputAction.BackPress)
//                    } else {
//                        // I think this case (deleting a selection) should be automatically handled
//                        // by `backpress` in the Rust lib, but that's not the case at the moment.
//                        inputProcessor.queue(EditorInputAction.Delete(start, start+before))
//                    }
//                }
//                count != 0 && replaced != "\n" -> {
//                    inputProcessor.queue(EditorInputAction.InsertText(replaced, start, start + before))
//                }
//                replaced == "\n" -> {
//                    inputProcessor.queue(EditorInputAction.InsertParagraph)
//                }
//                else -> {}
//            }
//        }
//
//        override fun afterTextChanged(s: Editable?) {
//            replacement?.let {
//                // Note: this is reentrant, it will call the TextWatcher again
//                s?.replace(0, s.length, it, 0, it.length)
//                if (s?.length == 0) {
//                    replacement = null
//                }
//            }
//        }
//    }
}

class InterceptInputFilter(
    private val editorActionEmitter: (EditorInputAction) -> Unit,
): InputFilter {

    // Used to avoid emitting actions in a loop
    var isReentrant = false

    override fun filter(
        source: CharSequence,
        start: Int,
        end: Int,
        dest: Spanned,
        dstart: Int,
        dend: Int
    ): CharSequence {
        if (isReentrant) {
            isReentrant = false
            return source
        }
        when {
            source.isNotEmpty() && source != "\n" -> {
                editorActionEmitter(EditorInputAction.InsertText(source, dstart, dend))
            }
            source == "\n" -> {
                editorActionEmitter(EditorInputAction.InsertParagraph)
            }
            source.isEmpty() && dend > dstart -> {
                if (dend - dstart == 1) {
                    editorActionEmitter(EditorInputAction.BackPress)
                } else {
                    // I think this case (deleting a selection) should be automatically handled
                    // by `backpress` in the Rust lib, but that's not the case at the moment.
                    editorActionEmitter(EditorInputAction.Delete(dstart, dend))
                }
            }
            else -> {}
        }
        return source
    }
}

sealed interface EditorInputAction {
    data class InsertText(val value: CharSequence, val start: Int, val end: Int): EditorInputAction
    data class ReplaceAll(val value: CharSequence): EditorInputAction
    data class Delete(val start: Int, val end: Int): EditorInputAction
    object InsertParagraph: EditorInputAction
    object BackPress: EditorInputAction
    data class ApplyInlineFormat(val format: InlineFormat): EditorInputAction
}

sealed interface InlineFormat {
    object Bold: InlineFormat
}

private fun List<UShort>.string() = with(StringBuffer()) {
    this@string.forEach {
        appendCodePoint(it.toInt())
    }
    toString()
}

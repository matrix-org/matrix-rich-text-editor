package io.element.android.wysiwygpoc

import android.os.Bundle
import android.text.*
import android.widget.TextView
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
    private val inputProcessor = InputProcessor(composer)

    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        val binding = ActivityMainBinding.inflate(layoutInflater)
        setContentView(binding.root)

        with (binding.editor) {
            requestFocus()
            selectionChangeListener = EditorEditText.OnSelectionChangeListener { start, end ->
                composer.select(start.toUInt(), end.toUInt())
            }
            addTextChangedListener(EditorTextWatcher(inputProcessor))
        }

        binding.buttonBold.setOnClickListener {
            val update = inputProcessor.processInput(
                EditorInputAction.ApplyInlineFormat(InlineFormat.Bold)
            ) ?: return@setOnClickListener
            val text = inputProcessor.processUpdate(update)
            text?.let { binding.editor.setText(it, TextView.BufferType.SPANNABLE) }
        }
    }

    class InputProcessor(
        private val composer: ComposerModel,
    ) {

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

        fun processUpdate(update: TextUpdate): CharSequence? {
            return when (update) {
                is TextUpdate.Keep -> null
                is TextUpdate.ReplaceAll -> {
                    stringToSpans(update.replacementHtml.string())
                }
            }
        }

        private fun stringToSpans(string: String): Spanned {
            // TODO: Check parsing flags
            val preparedString = string.replace(" ", "&nbsp;")
            return HtmlCompat.fromHtml(preparedString, 0)
        }
    }
}

class EditorTextWatcher(
    private val inputProcessor: MainActivity.InputProcessor,
) : TextWatcher {
    private var replacement: CharSequence? = null

    override fun beforeTextChanged(source: CharSequence?, start: Int, count: Int, after: Int) {}

    override fun onTextChanged(source: CharSequence?, start: Int, before: Int, count: Int) {
        // When we make any changes to the editor's text using `replacement` the TextWatcher
        // will be called again. When this happens, clean `replacement` and just return.
        if (replacement != null) {
            replacement = null
            return
        }
        // When all text is deleted, clean `replacement` and early return.
        if (source == null) {
            replacement = null
            return
        }

        val replaced = source.substring(start until start+count)
        val update = when {
            start == 0 && count == before -> {
                inputProcessor.processInput(EditorInputAction.ReplaceAll(replaced))
            }
            before > count -> {
                if (before - count == 1) {
                    inputProcessor.processInput(EditorInputAction.BackPress)
                } else {
                    // I think this case (deleting a selection) should be automatically handled
                    // by `backpress` in the Rust lib, but that's not the case at the moment.
                    inputProcessor.processInput(EditorInputAction.Delete(start, start+before))
                }
            }
            count != 0 && replaced != "\n" -> {
                inputProcessor.processInput(EditorInputAction.InsertText(replaced, start, start + before))
            }
            replaced == "\n" -> {
                inputProcessor.processInput(EditorInputAction.InsertParagraph)
            }
            else -> null
        }
        replacement = update?.let { inputProcessor.processUpdate(update) }
    }

    override fun afterTextChanged(s: Editable?) {
        replacement?.let {
            // Note: this is reentrant, it will call the TextWatcher again
            s?.replace(0, s.length, it, 0, it.length)
            if (s?.length == 0) {
                replacement = null
            }
        }
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

package io.element.android.wysiwygpoc

import android.os.Bundle
import androidx.appcompat.app.AppCompatActivity
import io.element.android.wysiwygpoc.databinding.ActivityMainBinding

class MainActivity : AppCompatActivity() {

    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        val binding = ActivityMainBinding.inflate(layoutInflater)
        setContentView(binding.root)

        binding.editor.requestFocus()

        binding.buttonBold.setOnClickListener {
            val inputProcessor = binding.editor.inputConnection
            inputProcessor.applyInlineFormat(InlineFormat.Bold)
        }
    }

}

sealed interface EditorInputAction {
    data class InsertText(val value: CharSequence): EditorInputAction
    data class ReplaceAll(val value: CharSequence): EditorInputAction
    data class Delete(val start: Int, val end: Int): EditorInputAction
    object InsertParagraph: EditorInputAction
    object BackPress: EditorInputAction
    data class ApplyInlineFormat(val format: InlineFormat): EditorInputAction
}

sealed interface InlineFormat {
    object Bold: InlineFormat
}

package io.element.android.wysiwygpoc

import androidx.appcompat.app.AppCompatActivity
import android.os.Bundle

class MainActivity : AppCompatActivity() {
    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        setContentView(R.layout.activity_main)

        val composer = uniffi.wysiwyg_composer.newComposerModel()
        composer.replaceText("Some text")
        composer.backspace()
    }
}

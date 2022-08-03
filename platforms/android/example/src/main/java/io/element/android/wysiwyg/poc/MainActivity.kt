package io.element.android.wysiwyg.poc

import android.os.Bundle
import androidx.appcompat.app.AppCompatActivity
import io.element.android.wysiwyg.poc.databinding.ActivityMainBinding

class MainActivity : AppCompatActivity() {

    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        val binding = ActivityMainBinding.inflate(layoutInflater)
        setContentView(binding.root)

        binding.editor.requestFocus()
    }

}

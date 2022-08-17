package io.element.android.wysiwyg.poc

import android.os.Bundle
import android.view.LayoutInflater
import androidx.appcompat.app.AlertDialog
import androidx.appcompat.app.AppCompatActivity
import io.element.android.wysiwyg.OnSetLinkListener
import io.element.android.wysiwyg.databinding.DialogSetLinkBinding
import io.element.android.wysiwyg.poc.databinding.ActivityMainBinding

class MainActivity : AppCompatActivity() {

    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        val binding = ActivityMainBinding.inflate(layoutInflater)
        setContentView(binding.root)

        binding.editor.requestFocus()

        val context = this

        binding.editor.onSetLinkListener = object: OnSetLinkListener {
            override fun openLinkDialog(link: String?, callback: (String) -> Unit) {
                val dialogBinding = DialogSetLinkBinding.inflate(LayoutInflater.from(context))
                AlertDialog.Builder(context)
                    .setTitle("Set a link to:")
                    .setView(dialogBinding.root)
                    .setPositiveButton("OK") { _, _ ->
                        callback(dialogBinding.editText.text.toString())
                    }
                    .setNegativeButton("Cancel", null)
                    .show()

                dialogBinding.editText.performClick()
            }
        }
    }

}

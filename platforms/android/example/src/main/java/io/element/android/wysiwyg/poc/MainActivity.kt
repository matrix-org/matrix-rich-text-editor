package io.element.android.wysiwyg.poc

import android.os.Bundle
import android.view.LayoutInflater
import android.view.View
import androidx.appcompat.app.AlertDialog
import androidx.appcompat.app.AppCompatActivity
import io.element.android.wysiwyg.poc.databinding.DialogSetLinkBinding
import io.element.android.wysiwyg.poc.databinding.ActivityMainBinding

class MainActivity : AppCompatActivity() {

    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        val binding = ActivityMainBinding.inflate(layoutInflater)
        setContentView(binding.root)

        binding.editor.requestFocus()

        val context = this

        binding.editor.onSetLinkListener = object: OnSetLinkListener {
            override fun openSetLinkDialog(callback: (url: String) -> Unit) {
                val dialogBinding = DialogSetLinkBinding.inflate(LayoutInflater.from(context))
                dialogBinding.text.visibility = View.GONE
                AlertDialog.Builder(context)
                    .setTitle(R.string.add_link)
                    .setView(dialogBinding.root)
                    .setPositiveButton(android.R.string.ok) { _, _ ->
                        callback(dialogBinding.link.text.toString())
                    }
                    .setNegativeButton(android.R.string.cancel, null)
                    .show()

                dialogBinding.link.performClick()
            }
            override fun openEditLinkDialog(text: String, url: String, callback: (text: String?, url: String?) -> Unit) {
                val dialogBinding = DialogSetLinkBinding.inflate(LayoutInflater.from(context))
                dialogBinding.link.setText(url)
                dialogBinding.text.setText(text)
                AlertDialog.Builder(context)
                    .setTitle(R.string.edit_link)
                    .setView(dialogBinding.root)
                    .setPositiveButton(android.R.string.ok) { _, _ ->
                        callback(dialogBinding.text.text.toString(), dialogBinding.link.text.toString())
                    }
                    .setNeutralButton(R.string.remove_link) { _, _ ->
                        callback(null, null)
                    }
                    .setNegativeButton(android.R.string.cancel, null)
                    .show()

                dialogBinding.link.performClick()
            }
            override fun openInsertLinkDialog(callback: (text: String, url: String) -> Unit) {
                val dialogBinding = DialogSetLinkBinding.inflate(LayoutInflater.from(context))
                AlertDialog.Builder(context)
                    .setTitle(R.string.insert_link)
                    .setView(dialogBinding.root)
                    .setPositiveButton(android.R.string.ok) { _, _ ->
                        callback(dialogBinding.text.text.toString(), dialogBinding.link.text.toString())
                    }
                    .setNegativeButton(android.R.string.cancel, null)
                    .show()

                dialogBinding.link.performClick()
            }
        }
    }

}

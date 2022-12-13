package io.element.android.wysiwyg.poc

import android.os.Bundle
import android.view.LayoutInflater
import android.view.View
import androidx.appcompat.app.AlertDialog
import androidx.appcompat.app.AppCompatActivity
import androidx.core.view.isGone
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
            override fun openCreateLinkDialog(
                callback: (url: String?) -> Unit
            ) {
                val dialogBinding = DialogSetLinkBinding.inflate(LayoutInflater.from(context))
                dialogBinding.text.isGone = true
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

            override fun openEditLinkDialog(
                currentLink: String?,
                currentText: String,
                callback: (url: String, text: String) -> Unit,
                removeLink: () -> Unit
            ) {
                val dialogBinding = DialogSetLinkBinding.inflate(LayoutInflater.from(context))
                dialogBinding.text.setText(currentText)
                dialogBinding.link.setText(currentLink)
                AlertDialog.Builder(context)
                    .setTitle(R.string.edit_link)
                    .setView(dialogBinding.root)
                    .setPositiveButton(android.R.string.ok) { _, _ ->
                        callback(dialogBinding.link.text.toString(), dialogBinding.text.text.toString())
                    }
                    .setNeutralButton(R.string.remove_link) { _, _ ->
                        removeLink()
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

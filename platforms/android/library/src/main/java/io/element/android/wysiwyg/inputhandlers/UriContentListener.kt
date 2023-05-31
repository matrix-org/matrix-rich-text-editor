package io.element.android.wysiwyg.inputhandlers

import android.content.ClipData
import android.net.Uri
import android.view.View
import androidx.core.view.ContentInfoCompat
import androidx.core.view.OnReceiveContentListener

class UriContentListener(
    private val onContent: (uri: Uri) -> Unit
) : OnReceiveContentListener {
    override fun onReceiveContent(view: View, payload: ContentInfoCompat): ContentInfoCompat? {
        val split = payload.partition { item -> item.uri != null }
        val uriContent = split.first
        val remaining = split.second

        if (uriContent != null) {
            val clip: ClipData = uriContent.clip
            for (i in 0 until clip.itemCount) {
                val uri = clip.getItemAt(i).uri
                // ... app-specific logic to handle the URI ...
                onContent(uri)
            }
        }
        // Return anything that we didn't handle ourselves. This preserves the default platform
        // behavior for text and anything else for which we are not implementing custom handling.
        return remaining
    }
}

package io.element.android.wysiwyg.utils

import android.app.Application
import android.content.Context
import io.element.android.wysiwyg.display.MentionDisplayHandler
import io.element.android.wysiwyg.internal.utils.AndroidHtmlConverter
import io.element.android.wysiwyg.view.StyleConfig

interface HtmlConverter {

    fun fromHtmlToSpans(html: String): CharSequence

    object Factory {
        fun create(
            context: Context,
            styleConfig: StyleConfig,
            mentionDisplayHandler: MentionDisplayHandler?,
            isMention: ((text: String, url: String) -> Boolean)? = null,
        ): HtmlConverter {
            val resourcesProvider =
                AndroidResourcesHelper(context.applicationContext as Application)
            return AndroidHtmlConverter(provideHtmlToSpansParser = { html ->
                HtmlToSpansParser(
                    resourcesHelper = resourcesProvider,
                    html = html,
                    styleConfig = styleConfig,
                    mentionDisplayHandler = mentionDisplayHandler,
                    isMention = isMention,
                )
            })
        }
    }


}

package io.element.android.wysiwyg.suggestions

/**
 * Convenience implementation of a [MentionUrlFilter] that detects Matrix mentions.
 */
class MatrixMentionUrlFilter : MentionUrlFilter {
    override fun isMention(url: String): Boolean =
        url.startsWith("https://matrix.to/#/")
}
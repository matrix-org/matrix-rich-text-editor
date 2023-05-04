package io.element.android.wysiwyg.poc

/**
 * Utility model class for the sample app to represent a mention to a
 * matrix.org user or room
 */
sealed class Mention(
    val display: String,
) {
    abstract val key: String
    abstract val mentionType: MentionType
    val link get() = "https://matrix.to/#/$key$display:matrix.org"

    class Room(
        display: String
    ): Mention(display) {
        override val mentionType: MentionType = MentionType.Room
        override val key: String = "#"
    }

    class User(
        display: String
    ): Mention(display) {
        override val mentionType: MentionType = MentionType.User
        override val key: String = "@"
    }
}
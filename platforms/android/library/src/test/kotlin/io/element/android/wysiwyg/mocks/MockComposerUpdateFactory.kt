package io.element.android.wysiwyg.mocks

import io.mockk.every
import io.mockk.mockk
import uniffi.wysiwyg_composer.ComposerUpdate
import uniffi.wysiwyg_composer.MenuState
import uniffi.wysiwyg_composer.TextUpdate

object MockComposerUpdateFactory {
    fun create(
        menuState: MenuState = MenuState.Keep,
        textUpdate: TextUpdate = TextUpdate.Keep,
    ): ComposerUpdate = mockk {
        every { menuState() } returns menuState
        every { textUpdate() } returns textUpdate
    }
}

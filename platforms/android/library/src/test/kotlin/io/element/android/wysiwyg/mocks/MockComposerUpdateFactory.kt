package io.element.android.wysiwyg.mocks

import io.mockk.every
import io.mockk.mockk
import uniffi.wysiwyg_composer.ComposerUpdate
import uniffi.wysiwyg_composer.MenuAction
import uniffi.wysiwyg_composer.MenuState
import uniffi.wysiwyg_composer.TextUpdate

object MockComposerUpdateFactory {
    fun create(
        menuAction: MenuAction = MenuAction.Keep,
        menuState: MenuState = MenuState.Keep,
        textUpdate: TextUpdate = TextUpdate.Keep,
    ): ComposerUpdate = mockk {
        every { menuAction() } returns menuAction
        every { menuState() } returns menuState
        every { textUpdate() } returns textUpdate
    }
}

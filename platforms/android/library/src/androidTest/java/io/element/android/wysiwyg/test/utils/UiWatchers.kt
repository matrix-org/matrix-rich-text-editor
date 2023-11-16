package io.element.android.wysiwyg.test.utils
import androidx.test.platform.app.InstrumentationRegistry.getInstrumentation
import androidx.test.uiautomator.UiDevice
import androidx.test.uiautomator.UiObject
import androidx.test.uiautomator.UiSelector
import org.junit.rules.TestWatcher
import org.junit.runner.Description

class DismissAnrRule : TestWatcher() {
    override fun starting(description: Description) {
        dismissAnr()
    }
}

private fun dismissAnr() {
    val device = UiDevice.getInstance(getInstrumentation())
    val dialog = device.findAnrDialog()
    if (dialog.exists()) {
        device.findWaitButton().click()
    }
}

private fun UiDevice.findAnrDialog(): UiObject =
    findObject(UiSelector().textContains("isn't responding"))

private fun UiDevice.findWaitButton(): UiObject =
    findObject(UiSelector().text("Wait").enabled(true))
        .apply { waitForExists(5000) }
package io.element.android.wysiwyg.test.utils

import org.junit.Assert

class FakeLinkClickedListener: (String) -> Unit {
    private val clickedLinks: MutableList<String> = mutableListOf()

    override fun invoke(link: String) {
        clickedLinks.add(link)
    }

    fun assertLinkClicked(url: String) {
        Assert.assertTrue(clickedLinks.size == 1)
        Assert.assertTrue(clickedLinks.contains(url))
    }
}
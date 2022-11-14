package io.element.android.wysiwyg.utils

/**
 * Callback for catching and dealing with Rust-related issues.
 */
fun interface RustErrorCollector {
    fun onRustError(throwable: Throwable)
}

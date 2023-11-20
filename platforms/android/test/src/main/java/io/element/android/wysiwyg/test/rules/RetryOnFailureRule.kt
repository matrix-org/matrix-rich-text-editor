package io.element.android.wysiwyg.test.rules

import org.junit.rules.TestRule
import org.junit.runner.Description
import org.junit.runners.model.Statement

internal class RetryOnFailureRule : TestRule {
    override fun apply(
        base: Statement,
        description: Description
    ): Statement =
        RetryStatement(base)
}

private class RetryStatement(private val base: Statement) : Statement() {
    override fun evaluate() {
        try {
            base.evaluate()
            return
        } catch (t: Throwable) {
            base.evaluate()
        }
    }
}

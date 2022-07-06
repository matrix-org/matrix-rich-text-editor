package net.artificialworlds.exampleandroid

import org.junit.Test

import org.junit.Assert.*

class ExampleUnitTest {
    @Test
    fun canGetStringFromRust() {
        assertEquals("I am in Rust", stringFromRust())
    }
}
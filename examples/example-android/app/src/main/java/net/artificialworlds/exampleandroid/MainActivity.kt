package net.artificialworlds.exampleandroid

import android.app.Activity
import android.os.Bundle
import net.artificialworlds.exampleandroid.databinding.ActivityMainBinding

class MainActivity : Activity() {

    private lateinit var binding: ActivityMainBinding

    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)

        binding = ActivityMainBinding.inflate(layoutInflater)
        setContentView(binding.root)

        val y = uniffi.my_rust_code.newComposerModel()
        binding.textview.text = y.identifyThyself()
    }
}
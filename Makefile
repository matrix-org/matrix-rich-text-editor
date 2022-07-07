all: android ios web

android: android-aarch64 android-x86_64

android-aarch64:
	cd bindings/wysiwyg-ffi && cargo build --target aarch64-linux-android
	echo Outputs for android-aarch64:
	echo - target/aarch64-linux-android/debug/libwysiwyg_ffi.so
	echo - bindings/wysiwyg-ffi/src/wysiwyg_composer.udl

android-x86_64:
	cd bindings/wysiwyg-ffi && cargo build --target x86_64-linux-android
	echo Outputs for android-x86_64:
	echo - target/x86_64-linux-android/debug/libwysiwyg_ffi.so
	echo - bindings/wysiwyg-ffi/src/wysiwyg_composer.udl

ios:
	# TODO

web:
	cd bindings/wysiwyg-wasm && \
	npm install && \
	npm run build
	echo Outputs for web:
	echo - pkg/*

clean:
	cargo clean
	rm -rf bindings/wysiwyg-wasm/node_modules
	rm -rf bindings/wysiwyg-wasm/pkg

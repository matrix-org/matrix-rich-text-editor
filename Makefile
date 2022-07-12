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
	cd bindings/ffi && \
	cargo build --target aarch64-apple-ios && \
	cargo build --target aarch64-apple-ios-sim && \
	cargo build --target x86_64-apple-ios && \
	mkdir -p ../../target/ios-combined && \
	lipo -create \
	  ../../target/x86_64-apple-ios/debug/libwysiwyg_composer.a \
	  ../../target/aarch64-apple-ios-sim/debug/libwysiwyg_composer.a \
	  -output ../../target/ios-combined/libwysiwyg_composer.a
	echo Outputs for iOS:
	echo - target/ios-combined/libwysiwyg_composer.a

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

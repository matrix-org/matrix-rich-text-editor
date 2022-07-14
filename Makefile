all: android ios web

android: android-aarch64 android-x86_64

android-aarch64:
	cd bindings/wysiwyg-ffi && cargo build --release --target aarch64-linux-android
	echo Outputs for android-aarch64:
	echo - target/aarch64-linux-android/release/libwysiwyg_ffi.so
	echo - bindings/wysiwyg-ffi/src/wysiwyg_composer.udl

android-x86_64:
	cd bindings/wysiwyg-ffi && cargo build --release --target x86_64-linux-android
	echo Outputs for android-x86_64:
	echo - target/x86_64-linux-android/release/libwysiwyg_ffi.so
	echo - bindings/wysiwyg-ffi/src/wysiwyg_composer.udl

ios:
	cd bindings/ffi && \
	cargo build -release --target aarch64-apple-ios && \
	cargo build -release --target aarch64-apple-ios-sim && \
	cargo build -release --target x86_64-apple-ios && \
	mkdir -p ../../target/ios-combined && \
	lipo -create \
	  ../../target/x86_64-apple-ios/release/libwysiwyg_composer.a \
	  ../../target/aarch64-apple-ios-sim/release/libwysiwyg_composer.a \
	  -output ../../target/ios-combined/libwysiwyg_composer.a
	echo Outputs for iOS:
	echo - target/ios-combined/libwysiwyg_composer.a

web:
	cd bindings/wysiwyg-wasm && \
	npm install && \
	npm run build && \
	mkdir -p ../../examples/example-web/generated && \
	cp \
		pkg/wysiwyg_bg.wasm \
		pkg/wysiwyg_bg.wasm.d.ts \
		pkg/wysiwyg.d.ts \
		pkg/wysiwyg.js \
		../../examples/example-web/generated/

clean:
	cargo clean
	rm -rf bindings/wysiwyg-wasm/node_modules
	rm -rf bindings/wysiwyg-wasm/pkg

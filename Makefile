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

IOS_GENERATED_DIR := ../../examples/example-ios/Generated

ios:
	cd bindings/wysiwyg-ffi && \
	cargo build --release --target aarch64-apple-ios && \
	cargo build --release --target aarch64-apple-ios-sim && \
	cargo build --release --target x86_64-apple-ios && \
	mkdir -p ../../target/ios-simulator && \
	lipo -create \
	  ../../target/x86_64-apple-ios/release/libwysiwyg_ffi.a \
	  ../../target/aarch64-apple-ios-sim/release/libwysiwyg_ffi.a \
	  -output ../../target/ios-simulator/libwysiwyg_ffi.a && \
	rm -rf ${IOS_GENERATED_DIR} && \
	mkdir -p ${IOS_GENERATED_DIR} && \
	uniffi-bindgen \
		generate src/wysiwyg_composer.udl \
		--language swift \
		--config uniffi.toml \
		--out-dir ${IOS_GENERATED_DIR} && \
	mkdir -p ${IOS_GENERATED_DIR}/headers && \
	mkdir -p ${IOS_GENERATED_DIR}/Sources && \
	mv ${IOS_GENERATED_DIR}/*.h         ${IOS_GENERATED_DIR}/headers/ && \
	mv ${IOS_GENERATED_DIR}/*.modulemap ${IOS_GENERATED_DIR}/headers/module.modulemap && \
	mv ${IOS_GENERATED_DIR}/*.swift     ${IOS_GENERATED_DIR}/Sources && \
	xcodebuild -create-xcframework \
	  -library ../../target/aarch64-apple-ios/release/libwysiwyg_ffi.a \
	  -headers ${IOS_GENERATED_DIR}/headers \
	  -library ../../target/ios-simulator/libwysiwyg_ffi.a \
	  -headers ${IOS_GENERATED_DIR}/headers \
	  -output ${IOS_GENERATED_DIR}/WysiwygComposerFFI.xcframework && \
	rm -rf ${IOS_GENERATED_DIR}/headers
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

test:
	cargo test

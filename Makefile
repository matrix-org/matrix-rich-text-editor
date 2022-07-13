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

EXAMPLE_IOS := ../../examples/example-ios

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
	mkdir -p ${EXAMPLE_IOS} && \
	rm -rf ${EXAMPLE_IOS}/Sources && \
	rm -rf ${EXAMPLE_IOS}/headers && \
	rm -rf ${EXAMPLE_IOS}/LibWysiwyg.xcframework && \
	uniffi-bindgen \
		generate src/wysiwyg_composer.udl \
		--language swift \
		--config uniffi.toml \
		--out-dir ${EXAMPLE_IOS} && \
	mkdir -p ${EXAMPLE_IOS}/headers && \
	mkdir -p ${EXAMPLE_IOS}/Sources && \
	mv ${EXAMPLE_IOS}/*.h         ${EXAMPLE_IOS}/headers/ && \
	mv ${EXAMPLE_IOS}/*.modulemap ${EXAMPLE_IOS}/headers/module.modulemap && \
	mv ${EXAMPLE_IOS}/*.swift     ${EXAMPLE_IOS}/Sources/ && \
	xcodebuild -create-xcframework \
	  -library ../../target/aarch64-apple-ios/release/libwysiwyg_ffi.a \
	  -headers ${EXAMPLE_IOS}/headers \
	  -library ../../target/ios-simulator/libwysiwyg_ffi.a \
	  -headers ${EXAMPLE_IOS}/headers \
	  -output ${EXAMPLE_IOS}/LibWysiwyg.xcframework

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

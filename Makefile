all: android ios web

android: android-aarch64 android-x86_64

android-aarch64:
	cd bindings/wysiwyg-ffi && \
	cargo build --target aarch64-linux-android && \
	mkdir -p ../../examples/example-android/app/src/main/jniLibs/aarch64 && \
	cp ../../target/aarch64-linux-android/debug/libwysiwyg_ffi.so \
		../../examples/example-android/app/src/main/jniLibs/aarch64/libuniffi_my_rust_code.so

android-x86_64:
	cd bindings/wysiwyg-ffi && \
	cargo build --target x86_64-linux-android && \
	mkdir -p ../../examples/example-android/app/src/main/jniLibs/x86_64 && \
	cp ../../target/x86_64-linux-android/debug/libwysiwyg_ffi.so \
		../../examples/example-android/app/src/main/jniLibs/x86_64/libuniffi_my_rust_code.so

ios:
	# TODO

web:
	cd bindings/wysiwyg-wasm && \
	npm install && \
	npm run build

# --- Prerequisites (Ubuntu/Debian only) ---

prerequisites:


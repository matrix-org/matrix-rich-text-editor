all: android ios web

# The gradle plugin will take care of building the bindings too
android: targets-android
	cd platforms/android && \
		./gradlew publishToMavenLocal

android-bindings: android-bindings-armv7 android-bindings-aarch64 android-bindings-x86_64

android-bindings-armv7:
	cd bindings/wysiwyg-ffi && \
		cargo build --release --target armv7-linux-androideabi

android-bindings-aarch64:
	cd bindings/wysiwyg-ffi && \
		cargo build --release --target aarch64-linux-android

android-bindings-x86_64:
	cd bindings/wysiwyg-ffi && \
		cargo build --release --target x86_64-linux-android
	# Not copying into the Android project here, since the gradle plugin
	# actually performs this build itself.

ios: targets-ios
	@sh build_xcframework.sh

web:
	cd bindings/wysiwyg-wasm && \
	npm install && \
	npm run build && \
	mkdir -p ../../platforms/web/generated && \
	cp \
		pkg/wysiwyg_bg.wasm \
		pkg/wysiwyg_bg.wasm.d.ts \
		pkg/wysiwyg.d.ts \
		pkg/wysiwyg.js \
		../../platforms/web/generated/
	cd platforms/web && yarn install && yarn build

web-format:
	cd platforms/web && \
	yarn prettier --write .

targets-android:
	rustup target add aarch64-linux-android
	rustup target add x86_64-linux-android
	rustup target add i686-linux-android
	rustup target add armv7-linux-androideabi

targets-ios:
	rustup target add aarch64-apple-ios
	rustup target add aarch64-apple-ios-sim
	rustup target add x86_64-apple-ios

clean:
	cargo clean
	rm -rf bindings/wysiwyg-wasm/node_modules
	rm -rf bindings/wysiwyg-wasm/pkg
	rm -rf bindings/wysiwyg-ffi/src/generated
	rm -rf platforms/android/out
	cd platforms/android && ./gradlew clean

test:
	cargo test
	cd platforms/web && yarn tsc && yarn test

coverage:
	@echo "Requires `rustup component add llvm-tools-preview`"
	@echo "Requires `cargo install cargo-llvm-cov`"
	cargo llvm-cov --open

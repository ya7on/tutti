all: build_linux_x86_musl build_linux_x86_gnu build_linux_arm_musl build_linux_arm_gnu build_apple_intel build_apple_silicon

build_linux_x86_musl:
	mkdir dist && \
	cargo zigbuild --release --target x86_64-unknown-linux-musl && \
	cp target/x86_64-unknown-linux-musl/release/tutti-cli dist/ && \
	cp LICENSE README.md dist/ && \
	tar -czvf tutti-0.1.4-x86_64-linux-musl.tar.gz -C dist . && \
	rm -rf dist

build_linux_x86_gnu:
	mkdir dist && \
	cargo zigbuild --release --target x86_64-unknown-linux-gnu && \
	cp target/x86_64-unknown-linux-gnu/release/tutti-cli dist/ && \
	cp LICENSE README.md dist/ && \
	tar -czvf tutti-0.1.4-x86_64-linux-gnu.tar.gz -C dist . && \
	rm -rf dist

build_linux_arm_musl:
	mkdir dist && \
	cargo zigbuild --release --target aarch64-unknown-linux-musl && \
	cp target/aarch64-unknown-linux-musl/release/tutti-cli dist/ && \
	cp LICENSE README.md dist/ && \
	tar -czvf tutti-0.1.4-aarch64-linux-musl.tar.gz -C dist . && \
	rm -rf dist

build_linux_arm_gnu:
	mkdir dist && \
	cargo zigbuild --release --target aarch64-unknown-linux-gnu && \
	cp target/aarch64-unknown-linux-gnu/release/tutti-cli dist/ && \
	cp LICENSE README.md dist/ && \
	tar -czvf tutti-0.1.4-aarch64-linux-gnu.tar.gz -C dist . && \
	rm -rf dist

build_apple_intel:
	mkdir dist && \
	cargo zigbuild --release --target x86_64-apple-darwin && \
	cp target/x86_64-apple-darwin/release/tutti-cli dist/ && \
	cp LICENSE README.md dist/ && \
	tar -czvf tutti-0.1.4-apple-intel.tar.gz -C dist . && \
	rm -rf dist

build_apple_silicon:
	mkdir dist && \
	cargo zigbuild --release --target aarch64-apple-darwin && \
	cp target/aarch64-apple-darwin/release/tutti-cli dist/ && \
	cp LICENSE README.md dist/ && \
	tar -czvf tutti-0.1.4-apple-silicon.tar.gz -C dist . && \
	rm -rf dist

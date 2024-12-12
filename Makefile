build:
	cargo build --release

install:
	cargo install --path "."

uninstall:
	cargo uninstall provok

clean:
	cargo clean

release-mac:
	strip target/release/provok
	mkdir -p release
	tar -C ./target/release/ -czvf ./release/provok-mac.tar.gz ./provok

release-win:
	mkdir -p release
	tar -C ./target/release/ -czvf ./release/provok-win.tar.gz ./provok.exe

release-linux:
	strip target/release/provok
	mkdir -p release
	tar -C ./target/release/ -czvf ./release/provok-linux.tar.gz ./provok

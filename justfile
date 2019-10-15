build:
	wasm-pack build web --target web
	rm web/fstree-web.tar.gz || true
	cd web/pkg && tar czf ../fstree-web.tar.gz .
	cargo build

run: build
	RUST_LOG=debug cargo run

web: build
	RUST_LOG=debug cargo run -- --web-only

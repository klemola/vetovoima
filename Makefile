build-release-win:
	cargo.exe build --release

build-release:
	cargo build --release

prepare-zip-win: build-release-win
	@echo ">> Setup"
	@mkdir -p ./build
	@rm -rf ./build/**

	@echo ">> Copying files"
	cp ./target/release/vetovoima.exe ./build
	cp -r ./assets ./build

	@echo ">> Creating an archive"
	cd build && zip -rq vetovoima.zip ./*
	@echo ">> Done!"

prepare-zip: build-release
	@echo ">> Setup"
	@mkdir -p ./build
	@rm -rf ./build/**

	@echo ">> Copying files"
	cp ./target/release/vetovoima ./build
	cp -r ./assets ./build

	@echo ">> Creating an archive"
	cd build && zip -rq vetovoima.zip ./*
	@echo ">> Done!"

run-dev-win:
	cargo.exe run

run-dev:
	cargo run

.PHONY: build-release-win build-release prepare-zip-win prepare-zip run-dev-win run-dev

# NOTE: Must be first recipe to be default
@_default:
	just --list

@_check-in-obscura-nix-shell:
	./contrib/bin/check-in-obscura-nix-shell.bash

lint: _check-in-obscura-nix-shell
	actionlint -color
	@# `cargo clippy` runs `cargo check`
	cargo --offline clippy --all-features --all-targets -- -Dwarnings
	./contrib/bin/shellcheck-auto-files.bash

format-check: _check-in-obscura-nix-shell
	cargo --offline fmt --all --check
	./contrib/bin/nixfmt-auto-files.bash --check

format-fix: _check-in-obscura-nix-shell
	cargo --offline fmt --all
	./contrib/bin/nixfmt-auto-files.bash

build *FLAGS:
	cargo --locked build {{FLAGS}}

# build with `--workspace --all-targets --all-features`
build-all *FLAGS:
	just build --workspace --all-targets --all-features {{FLAGS}}

test *FLAGS:
	cargo --locked test {{FLAGS}}

# test with `--workspace --all-targets --all-features`
test-all *FLAGS:
	just test --workspace --all-targets --all-features {{FLAGS}}

flake-check:
	nix flake check --all-systems --no-build

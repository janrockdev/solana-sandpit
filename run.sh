#!/bin/bash

function build_bpf() {
    cargo build-bpf --manifest-path=contract/Cargo.toml --bpf-out-dir=dist/contract
}

case $1 in
    "build-bpf")
	build_bpf
	;;
    "deploy")
	build_bpf
	solana program deploy dist/contract/helloworld.so
	;;
    "client")
	(cd client/; cargo run ../dist/contract/helloworld-keypair.json)
	;;
    "clean")
	(cd program/; cargo clean)
	(cd client/; cargo clean)
	rm -rf dist/
	;;
    *)
	echo "usage: $0 build-bpf"
	;;
esac
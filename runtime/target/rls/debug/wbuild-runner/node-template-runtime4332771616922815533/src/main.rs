
				use substrate_wasm_builder::build_project_with_default_rustflags;

				fn main() {
					build_project_with_default_rustflags(
						"/home/ipfs/substrate-node-template/runtime/target/rls/debug/build/node-template-runtime-320d1bc1b5582b54/out/wasm_binary.rs",
						"/home/ipfs/substrate-node-template/runtime/Cargo.toml",
						"-Clink-arg=--export=__heap_base -C link-arg=--import-memory ",
					)
				}
			
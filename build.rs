fn main() {
    tonic_build::compile_protos("proto/solana.proto").unwrap();
}
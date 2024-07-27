use solana_sdk::bs58;

fn main() {
    let secret_key: Vec<u8> = vec![
        0x1a, 0x2b, 0x3c, 0x4d, 0x5e, 0x6f, 0x7a, 0x8b, 0x9c, 0xad, 0xbe, 0xcf, 0xd0, 0xe1, 0xf2, 0x03, //replace with your secret key
    ];
    let secret_key_base58 = bs58::encode(&secret_key).into_string();
    println!("{}", secret_key_base58);
}
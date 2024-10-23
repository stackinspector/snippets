use cshake::Squeeze;

fn main() {
    let mut rng = cshake::rand::thread_rng();
    let entropy = rng.squeeze_to_array::<12>();
    println!("{}", data_encoding::BASE64URL.encode(&entropy));
}

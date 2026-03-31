use rsa::pkcs1v15::{Signature, SigningKey, VerifyingKey};
use rsa::rand_core::OsRng;
use rsa::signature::{RandomizedSigner, Verifier as _};
use rsa::{Pkcs1v15Encrypt, RsaPrivateKey, RsaPublicKey};
use sha2::Sha256;

pub fn generate_keys() -> (RsaPrivateKey, RsaPublicKey) {
    let mut rng = OsRng;
    let private_key = RsaPrivateKey::new(&mut rng, 2048).expect("Failed to generate private key");
    let public_key = RsaPublicKey::from(&private_key);
    (private_key, public_key)
}

pub fn encrypt_data(public_key: &RsaPublicKey, data: &[u8]) -> Vec<u8> {
    let mut rng = OsRng;
    public_key
        .encrypt(&mut rng, Pkcs1v15Encrypt, data)
        .expect("Failed to encrypt")
}

pub fn decrypt_data(private_key: &RsaPrivateKey, encrypted_data: &[u8]) -> Vec<u8> {
    private_key
        .decrypt(Pkcs1v15Encrypt, encrypted_data)
        .expect("Failed to decrypt")
}

pub fn sign_data(private_key: &RsaPrivateKey, data: &[u8]) -> Signature {
    let signing_key = SigningKey::<Sha256>::new_unprefixed(private_key.clone());
    let mut rng = OsRng;
    signing_key.sign_with_rng(&mut rng, data)
}

pub fn verify_signature(public_key: &RsaPublicKey, data: &[u8], signature: &Signature) -> bool {
    let verifying_key = VerifyingKey::<Sha256>::new_unprefixed(public_key.clone());
    verifying_key.verify(data, signature).is_ok()
}

#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn test_encrypt_and_decrypt() {
        let (private_key, public_key) = generate_keys();
        let data = b"Hello, RSA in Rust!";
        let encrypted_data = encrypt_data(&public_key, data);
        dbg!("Encrypted Data: {:?}", &encrypted_data);
        let decrypted_data = decrypt_data(&private_key, &encrypted_data);
        dbg!(
            "Decrypted Data: {:?}",
            String::from_utf8_lossy(&decrypted_data)
        );
        assert!(decrypted_data == data)
    }

    #[test]
    fn test_sign_and_verify() {
        let (private_key, public_key) = generate_keys();
        let data = b"Hello, RSA in Rust!";
        let signature = sign_data(&private_key, data);
        println!("Signature: {:?}", signature);
        let is_valid = verify_signature(&public_key, data, &signature);
        assert!(is_valid)
    }
}

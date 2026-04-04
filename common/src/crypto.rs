use rsa::pkcs1v15::{Signature, SigningKey, VerifyingKey};
use rsa::rand_core::OsRng;
use rsa::signature::{RandomizedSigner, Verifier as _};
use rsa::{Pkcs1v15Encrypt, RsaPrivateKey, RsaPublicKey};
use sha2::Sha256;

pub fn generate_keys() -> Result<(RsaPrivateKey, RsaPublicKey), rsa::Error> {
    let mut rng = OsRng;
    let private_key = RsaPrivateKey::new(&mut rng, 2048)?;
    let public_key = RsaPublicKey::from(&private_key);
    Ok((private_key, public_key))
}

pub fn encrypt_data(public_key: &RsaPublicKey, data: &[u8]) -> Result<Vec<u8>, rsa::Error> {
    let mut rng = OsRng;
    public_key.encrypt(&mut rng, Pkcs1v15Encrypt, data)
}

pub fn decrypt_data(
    private_key: &RsaPrivateKey,
    encrypted_data: &[u8],
) -> Result<Vec<u8>, rsa::Error> {
    private_key.decrypt(Pkcs1v15Encrypt, encrypted_data)
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

pub fn public_key_to_bytes(public_key: &RsaPublicKey) -> Result<Vec<u8>, postcard::Error> {
    postcard::to_allocvec(public_key)
}

pub fn bytes_to_public_key(bytes: &[u8]) -> Result<RsaPublicKey, postcard::Error> {
    postcard::from_bytes(bytes)
}

#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn test_encrypt_and_decrypt() {
        let (private_key, public_key) = generate_keys().unwrap();
        let data = b"Hello, RSA in Rust!";
        let encrypted_data = encrypt_data(&public_key, data).unwrap();
        dbg!("Encrypted Data: {:?}", &encrypted_data);
        let decrypted_data = decrypt_data(&private_key, &encrypted_data);
        assert!(decrypted_data.unwrap() == data)
    }

    #[test]
    fn test_sign_and_verify() {
        let (private_key, public_key) = generate_keys().unwrap();
        let data = b"Hello, RSA in Rust!";
        let signature = sign_data(&private_key, data);
        dbg!("Signature: {:?}", &signature);
        let is_valid = verify_signature(&public_key, data, &signature);
        assert!(is_valid)
    }
}

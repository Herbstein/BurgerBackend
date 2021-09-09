use std::fs;

use anyhow::Result;
use bytes::BytesMut;
use chrono::{Duration, Local};
use crypto::{blake2b::Blake2b, digest::Digest};
use ed25519_dalek::{
    ed25519::signature::Signature as _, Keypair, Signature, SignatureError, Signer,
};
use once_cell::sync::Lazy;
use rand_core::OsRng;

use crate::errors::ServiceError;

static KEYPAIR_AUTHN: Lazy<KeyPair> = Lazy::new(|| {
    KeyPair::from_file_or_new("keypair_tkn_sign").expect("failed to generate keypair")
});

const KEYS_FOLDER: &str = "./cache/keys";

#[derive(Debug, Clone)]
pub struct Claims {
    pub iat: i64,
    pub exp: i64,
    pub user_id: i64,
}

impl Claims {
    fn from_user_id(user_id: i64) -> Self {
        Self {
            user_id,
            iat: Local::now().timestamp(),
            exp: (Local::now() + Duration::hours(24)).timestamp(),
        }
    }

    fn hash(&self) -> [u8; 32] {
        let mut ret = [0u8; 32];
        let mut hasher = Blake2b::new(32);
        hasher.input(&self.user_id.to_be_bytes());
        hasher.input(&self.iat.to_be_bytes());
        hasher.input(&self.exp.to_be_bytes());
        hasher.result(&mut ret);
        ret
    }

    fn sign(self) -> Result<AuthnToken, ServiceError> {
        let sig = KEYPAIR_AUTHN.sign(&self.hash());
        Ok(AuthnToken { claims: self, sig })
    }
}

#[derive(Debug)]
pub struct AuthnToken {
    pub claims: Claims,
    pub sig: Signature,
}

impl AuthnToken {
    pub fn from_user_id(user_id: i64) -> Result<AuthnToken, ServiceError> {
        Claims::from_user_id(user_id).sign()
    }

    pub fn verify(&self) -> Result<(), ServiceError> {
        if self.claims.exp < Local::now().timestamp() {
            return Err(ServiceError::Unauthorized);
        }
        KEYPAIR_AUTHN
            .verify(&self.claims.hash(), &self.sig)
            .map_err(|_| ServiceError::Unauthorized)
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        let mut b = BytesMut::new();
        b.extend_from_slice(&self.claims.iat.to_be_bytes());
        b.extend_from_slice(&self.claims.exp.to_be_bytes());
        b.extend_from_slice(&self.claims.user_id.to_be_bytes());
        b.extend_from_slice(&self.sig.to_bytes());
        // b.len is 88
        b.to_vec()
    }

    pub fn from_bytes(bytes: &[u8]) -> Result<Self> {
        let mut buf = [0u8; 8];
        buf.copy_from_slice(&bytes[0..8]);
        let iat = i64::from_be_bytes(buf);

        buf.copy_from_slice(&bytes[8..16]);
        let exp = i64::from_be_bytes(buf);

        buf.copy_from_slice(&bytes[16..24]);
        let user_id = i64::from_be_bytes(buf);

        let sig = Signature::from_bytes(&bytes[24..])?;

        Ok(AuthnToken {
            claims: Claims { iat, exp, user_id },
            sig,
        })
    }

    pub fn to_str(&self) -> String {
        base64::encode(&self.to_bytes())
    }

    pub fn from_str(token: &str) -> Result<Self, ServiceError> {
        let bytes = base64::decode(&token)?;
        Ok(Self::from_bytes(&bytes)?)
    }

    pub fn header_val(&self) -> String {
        format!(
            "token={};Path=/;SameSite=Strict;Secure;HttpOnly",
            self.to_str()
        )
    }
}

pub struct KeyPair(Keypair);

impl KeyPair {
    pub fn generate() -> Self {
        Self(Keypair::generate(&mut OsRng))
    }

    pub fn sign(&self, message: &[u8]) -> Signature {
        self.0.sign(message)
    }

    pub fn verify(&self, message: &[u8], signature: &Signature) -> Result<(), SignatureError> {
        self.0.verify(message, signature)
    }

    pub fn from_bytes<'a>(bytes: &'a [u8]) -> Result<Self> {
        Ok(Self(Keypair::from_bytes(bytes)?))
    }

    pub fn to_bytes(&self) -> [u8; 64] {
        self.0.to_bytes()
    }

    pub fn from_str(s: &str) -> Result<Self> {
        Ok(Self::from_bytes(&base64::decode(s)?.to_vec())?)
    }

    pub fn to_str(&self) -> String {
        base64::encode(self.to_bytes().to_vec())
    }

    fn to_file(&self, keyfile: &str) -> Result<&Self> {
        fs::create_dir_all(KEYS_FOLDER)?;
        fs::write(keyfile, self.to_str())?;
        Ok(self)
    }

    fn from_file(keyfile: &str) -> Result<Self> {
        let content_str = fs::read_to_string(keyfile)?;
        Ok(Self::from_str(&content_str)?)
    }

    fn from_file_or_new(keyfile: &str) -> Result<Self> {
        let keyfile = format!("{}/{}", KEYS_FOLDER, keyfile);
        match Self::from_file(&keyfile) {
            Ok(identity) => Ok(identity),
            Err(_) => {
                let new_wallet = Self::generate();
                new_wallet.to_file(&keyfile)?;
                Ok(new_wallet)
            }
        }
    }
}

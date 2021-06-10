use crdts::{CmRDT, VClock};
use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;
use ursa::bls::*;
use eyre::{eyre, Result};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Versioned {
    pub clock: VClock<String>,
    pub data: Vec<u8>,
}

impl Versioned {
    pub fn new(data: Vec<u8>) -> Self {
        Self {
            clock: VClock::new(),
            data,
        }
    }

    pub fn apply(&mut self, actor: String) {
        self.clock.apply(self.clock.inc(actor))
    }
}

struct User {
    ver_key: VerKey,
    sign_key: SignKey,
}

impl User {
    fn new() -> Result<Self> {
        let gen = Generator::new().map_err(|e| eyre!("{}", e))?;
        let sign_key = SignKey::new(None).map_err(|e| eyre!("{}", e))?;
        let ver_key = VerKey::new(&gen, &sign_key).map_err(|e| eyre!("{}", e))?;

        Ok(Self {
            ver_key,
            sign_key,
        })
    }

    fn sign(&self, data: &[u8]) -> Result<Signature> {
        Bls::sign(data, &self.sign_key).map_err(|e| eyre!("{}", e))
    }

    fn verify(&self, data: &[u8]) -> Result<Signature> {
        Bls::sign(data, &self.sign_key).map_err(|e| eyre!("{}", e))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ursa_verify() -> Result<()> {
        let user_a = User::new()?;
        let counter = Versioned::new(vec![0]);
        let signature = user_a.sign(&counter.data)?;

        dbg!(signature);

        Ok(())
    }
}
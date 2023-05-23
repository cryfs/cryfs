use anyhow::{ensure, Result};
use binrw::{binrw, until_eof, BinRead, BinWrite};
use rand::{thread_rng, RngCore};
use std::io::Cursor;

use super::super::KDFParameters;
use super::ScryptSettings;

#[binrw]
#[brw(little)]
pub struct ScryptParams {
    #[br(try_map = |x: u64| parse_log_n(x))]
    #[bw(map = |x: &u8| write_log_n(*x))]
    log_n: u8,

    r: u32,

    p: u32,

    #[br(parse_with = until_eof)]
    salt: Vec<u8>,
}

impl ScryptParams {
    pub fn generate(settings: &ScryptSettings) -> Result<Self> {
        ensure!(
            settings.log_n < 64,
            "Scrypt parameter log_n is {} but must be smaller than 64",
            settings.log_n,
        );
        let mut salt = vec![0; settings.salt_len];
        thread_rng().fill_bytes(&mut salt);
        Ok(Self {
            log_n: settings.log_n,
            r: settings.r,
            p: settings.p,
            salt,
        })
    }

    pub fn salt(&self) -> &[u8] {
        &self.salt
    }

    pub fn params(&self) -> scrypt::Params {
        scrypt::Params::new(
            self.log_n,
            self.r,
            self.p,
            // scrypt::Params::len is an ignored field so shouldn't really matter what we give it
            scrypt::Params::RECOMMENDED_LEN,
        )
        .expect("Invalid scrypt parameters")
    }
}

fn write_log_n(log_n: u8) -> u64 {
    assert!(
        log_n < 64,
        "Scrypt parameter log_n is {log_n} but must be smaller than 64",
    );
    1 << log_n
}

fn parse_log_n(n: u64) -> Result<u8> {
    let log_n: u8 =
        u8::try_from(n.ilog2()).expect("log2(64 bit value) cannot be larger than u8::MAX");
    ensure!(
        write_log_n(log_n) == n,
        "Scrypt parameter n={n} must be a power of 2 but isn't"
    );
    Ok(log_n)
}

impl KDFParameters for ScryptParams {
    fn serialize(&self) -> Vec<u8> {
        let mut result = Cursor::new(vec![]);
        self.write(&mut result)
            .expect("Writing can't fail because our serializer shouldn't throw anywhere");
        result.into_inner()
    }

    fn deserialize(serialized: &[u8]) -> Result<Self> {
        let mut cursor = Cursor::new(serialized);
        let result = Self::read(&mut cursor)?;
        Ok(result)
    }
}

// TODO Tests

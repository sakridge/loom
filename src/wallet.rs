//! wallet library

use std::slice::from_raw_parts;
use std::fs::File;
use std::mem::transmute;
use std::mem::size_of;
use std::io::Read;
use std::io::Write;

use crypto::ed25519;
use rand::Rng;
use rand::os::OsRng;

use data;
use result::Result;
use result::Error;
use serde_json;
use aes;

type Keypair = ([u64; 8], [u64; 4]);

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub struct EncryptedWallet {
    pub iv: [u8; 16],
    pub pubkeys: Vec<[u64; 4]>,
    pub privkeys: Vec<u8>,
}

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub struct Wallet {
    pub iv: [u8; 16],
    pub pubkeys: Vec<[u64; 4]>,
    pub privkeys: Vec<[u64; 8]>,
}

impl EncryptedWallet {
    pub fn new() -> EncryptedWallet {
        let mut rnd: OsRng = OsRng::new().unwrap();
        let mut seed = [0u8; 16];
        rnd.fill_bytes(&mut seed);
        EncryptedWallet {
            iv: seed,
            pubkeys: Vec::new(),
            privkeys: Vec::new(),
        }
    }
    pub fn from_file(path: &str) -> Result<EncryptedWallet> {
        let mut file = File::open(path)?;
        let mut e = Vec::new();
        let _sz = file.read_to_end(&mut e)?;
        let ew: EncryptedWallet = serde_json::from_slice(&e)?;
        Ok(ew)
    }
    pub fn to_file(&self, path: &str) -> Result<()> {
        let mut file = File::create(path)?;
        let d = serde_json::to_vec(self)?;
        file.write_all(&d)?;
        Ok(())
    }
    pub fn decrypt(&self, pass: &[u8]) -> Result<Wallet> {
        let d = aes::decrypt(&self.privkeys, pass, &self.iv)?;
        let pks = serde_json::from_slice(&d)?;
        let w = Wallet {
            iv: self.iv.clone(),
            pubkeys: self.pubkeys.clone(),
            privkeys: pks,
        };
        Ok(w)
    }
}

impl Wallet {
    pub fn new() -> Wallet {
        let mut rnd: OsRng = OsRng::new().unwrap();
        let mut seed = [0u8; 16];
        rnd.fill_bytes(&mut seed);
        Wallet {
            iv: seed,
            pubkeys: Vec::new(),
            privkeys: Vec::new(),
        }
    }
    pub fn add_keypair(&mut self, pk: Keypair) {
        self.privkeys.push(pk.0);
        self.pubkeys.push(pk.1);
    }
    pub fn encrypt(self, pass: &[u8]) -> Result<EncryptedWallet> {
        let pks = serde_json::to_vec(&self.privkeys)?;
        let e = aes::encrypt(&pks, pass, &self.iv)?;
        let ew = EncryptedWallet {
            iv: self.iv.clone(),
            pubkeys: self.pubkeys,
            privkeys: e,
        };
        Ok(ew)
    }
    pub fn new_keypair() -> Keypair {
        let mut rnd: OsRng = OsRng::new().unwrap();
        let mut seed = [0u8; 64];
        rnd.fill_bytes(&mut seed);
        let (a, b) = ed25519::keypair(&seed);
        assert!(cfg!(target_endian = "little"));
        let ap = unsafe { transmute::<[u8; 64], [u64; 8]>(a) };
        let bp = unsafe { transmute::<[u8; 32], [u64; 4]>(b) };
        (ap, bp)
    }
    pub fn sign(kp: Keypair, msg: &mut data::Message) {
        let sz = size_of::<data::Payload>();
        let p = &msg.pld as *const data::Payload;
        assert!(cfg!(target_endian = "little"));
        let buf = unsafe { transmute(from_raw_parts(p as *const u8, sz)) };
        let pk = unsafe { transmute::<[u64; 8], [u8; 64]>(kp.0) };
        msg.sig = ed25519::signature(buf, &pk);
    }
    pub fn find(&self, from: [u8; 32]) -> Result<usize> {
        let fk = unsafe { transmute::<[u8; 32], [u64; 4]>(from) };
        for (i, k) in self.pubkeys.iter().enumerate() {
            if *k == fk {
                return Ok(i);
            }
        }
        Err(Error::PubKeyNotFound)
    }
    pub fn tx(&self, key: usize, to: [u8; 32], amnt: u64, fee: u64) -> data::Message {
        let data = data::MessageData {
            tx: data::Transaction {
                to: to,
                amount: amnt,
            },
        };
        let k = self.pubkeys[key];
        let mut msg = data::Message::default();
        msg.pld.from = unsafe { transmute::<[u64; 4], [u8; 32]>(k) };
        msg.pld.fee = fee;
        msg.pld.data = data;
        msg.pld.kind = data::Kind::Transaction;
        Self::sign((self.privkeys[key], self.pubkeys[key]), &mut msg);
        msg
    }
    pub fn check_balance(&self, key: usize, acc: [u8; 32], fee: u64) -> data::Message {
        let data = data::MessageData {
            bal: data::GetBalance {
                key: acc,
                amount: 0,
            },
        };
        let k = self.pubkeys[key];
        let mut msg = data::Message::default();
        msg.pld.kind = data::Kind::GetBalance;
        msg.pld.from = unsafe { transmute::<[u64; 4], [u8; 32]>(k) };
        msg.pld.fee = fee;
        msg.pld.data = data;
        Self::sign((self.privkeys[key], self.pubkeys[key]), &mut msg);
        msg
    }
}

#[cfg(test)]
mod test {
    use wallet::Wallet;
    use wallet::EncryptedWallet;
    use std::fs::remove_file;

    #[test]
    fn test_roundtrip() {
        let mut w = Wallet::new();
        let kp = Wallet::new_keypair();
        w.add_keypair(kp);
        let ow = w.clone();
        let pass = "foobar".as_bytes();
        let ew = w.encrypt(pass).expect("encrypted");
        let nw = ew.decrypt(pass).expect("decrypted");
        assert_eq!(nw, ow);
    }
    #[test]
    fn test_file() {
        let mut w = Wallet::new();
        let kp = Wallet::new_keypair();
        w.add_keypair(kp);
        let ow = w.clone();
        let pass = "foobar".as_bytes();
        let ew = w.encrypt(pass).expect("encrypted");
        ew.to_file("TESTWALLET").expect("to_file");
        let new = EncryptedWallet::from_file("TESTWALLET").expect("from_file");
        let nw = new.decrypt(pass).expect("decrypted");
        //remove_file("TESTWALLET").expect("remove");
        assert_eq!(nw, ow);
    }
    #[test]
    fn test_saved() {
        let path = "testdata/loom.wallet";
        let ew = EncryptedWallet::from_file(&path).expect("from file");
        let _w = ew.decrypt("foobar".as_bytes()).expect("decrypt wallet");
    }
}


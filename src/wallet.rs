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

#[derive(Serialize, Deserialize, Debug)]
pub struct EncryptedWallet {
    pub iv: [u8; aes::KEYSIZE],
    pub pubkeys: Vec<[u64; 4]>,
    pub privkeys: Vec<u8>,
}

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub struct Wallet {
    pub iv: [u8; aes::KEYSIZE],
    pub pubkeys: Vec<[u64; 4]>,
    pub privkeys: Vec<[u64; 8]>,
}

impl EncryptedWallet {
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
    pub fn decrypt(&self, ipass: &[u8]) -> Result<Wallet> {
        let mut pass = Vec::from(ipass);
        pass.resize(aes::KEYSIZE, 0);
        let d = aes::decrypt(&self.privkeys, &pass, &self.iv)?;
        let pks = serde_json::from_slice(&d)?;
        let w = Wallet {
            iv: self.iv.clone(),
            pubkeys: self.pubkeys.clone(),
            privkeys: pks,
        };
        Ok(w)
    }
}

pub fn to32b(k: [u64; 4]) -> [u8; 32] {
    unsafe { transmute::<[u64; 4], [u8; 32]>(k) }
}
pub fn from32b(k: [u8; 32]) -> [u64; 4] {
    unsafe { transmute::<[u8; 32], [u64; 4]>(k) }
}

pub fn to64b(k: [u64; 8]) -> [u8; 64] {
    unsafe { transmute::<[u64; 8], [u8; 64]>(k) }
}
pub fn from64b(k: [u8; 64]) -> [u64; 8] {
    unsafe { transmute::<[u8; 64], [u64; 8]>(k) }
}

impl Wallet {
    pub fn new() -> Wallet {
        let mut rnd: OsRng = OsRng::new().unwrap();
        let mut seed = [0u8; aes::KEYSIZE];
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
    pub fn encrypt(self, ipass: &[u8]) -> Result<EncryptedWallet> {
        let mut pass = Vec::from(ipass);
        pass.resize(aes::KEYSIZE, 0);
        let pks = serde_json::to_vec(&self.privkeys)?;
        let e = aes::encrypt(&pks, &pass, &self.iv)?;
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
        let ap = from64b(a);
        let bp = from32b(b);
        (ap, bp)
    }
    pub fn sign(kp: Keypair, msg: &mut data::Message) {
        let sz = size_of::<data::Payload>();
        let p = &msg.pld as *const data::Payload;
        assert!(cfg!(target_endian = "little"));
        let buf = unsafe { transmute(from_raw_parts(p as *const u8, sz)) };
        let pk = to64b(kp.0);
        msg.sig = ed25519::signature(buf, &pk);
    }
    pub fn find(&self, from: [u8; 32]) -> Result<usize> {
        let fk = from32b(from);
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
        msg.pld.from = to32b(k);
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
        msg.pld.from = to32b(k);
        msg.pld.fee = fee;
        msg.pld.data = data;
        Self::sign((self.privkeys[key], self.pubkeys[key]), &mut msg);
        msg
    }
}

#[cfg(test)]
mod test {
    use wallet::Wallet;
    use wallet::to32b;
    use wallet::EncryptedWallet;
    use std::fs::remove_file;
    use result::Error;
    use std::io;
    use std::io::Write;

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
    fn test_find() {
        let mut w = Wallet::new();
        let kp1 = Wallet::new_keypair();
        w.add_keypair(kp1);
        let f1 = w.find(to32b(kp1.1)).expect("find k1");
        let kp2 = Wallet::new_keypair();
        assert_matches!(w.find(to32b(kp2.1)), Err(Error::PubKeyNotFound));
        w.add_keypair(kp2);
        let f2 = w.find(to32b(kp2.1)).expect("find k2");
        assert!(f1 != f2);
        assert_eq!(kp2.1, w.pubkeys[f2]);
        assert_eq!(kp2.0, w.privkeys[f2]);
    }
    #[test]
    fn test_bad_file() {
        let e = EncryptedWallet::from_file("testdata/test_accounts.json");
        assert_matches!(e, Err(Error::JSON(_)));
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
        write!(io::sink(), "{:?}", ew).expect("force debug trait");
        let new = EncryptedWallet::from_file("TESTWALLET").expect("from_file");
        let nw = new.decrypt(pass).expect("decrypted");
        remove_file("TESTWALLET").expect("remove");
        assert_eq!(nw, ow);
    }
    #[test]
    fn test_saved() {
        let path = "testdata/loom.wallet";
        let ew = EncryptedWallet::from_file(&path).expect("from file");
        let w = ew.decrypt("foobar".as_bytes()).expect("decrypt wallet");
        assert_eq!(w.pubkeys.len(), 1);
        assert_eq!(w.privkeys.len(), 1);
    }
}

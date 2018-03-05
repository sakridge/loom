use rpassword;
use getopts::Options;
use std::string::String;
use data_encoding::BASE32HEX_NOPAD;
use wallet::{EncryptedWallet, Wallet, to32b};
use net;
use result::Result;

struct Cfg {
    host: String,
    wallet: String,
}

fn getpass<T>(r: Option<T>) -> String
where T: ::std::io::BufRead
{
    println!("loom wallet password: ");
    let pass = rpassword::read_password_with_reader(r).expect("read password");
    println!("pass is {:?} long", pass.len());
    return pass;
}

fn vec_to_array(v: Vec<u8>) -> [u8; 32] {
    let mut a = [0; 32];
    a.copy_from_slice(&v);
    return a;
}

fn print_usage(program: &str, opts: Options) {
    let brief = format!("Usage: {} FILE [options]", program);
    print!("{}", opts.usage(&brief));
}

fn load_wallet(cfg: &Cfg, pass: String) -> Wallet
{
    println!("loading from {:?}", cfg.wallet);
    match EncryptedWallet::from_file(&cfg.wallet) {
        Ok(ew) => ew.decrypt(pass.as_bytes()).expect("decrypt wallet"),
        _ => Wallet::new(),
    }
}


fn new_key_pair<T>(cfg: &Cfg, r: Option<T>)
    where T: ::std::io::BufRead 
{
    let pass = getpass(r);
    let mut w = load_wallet(cfg, pass.clone());
    println!("wallet has {:?} keys", w.pubkeys.len());
    let kp = Wallet::new_keypair();
    w.add_keypair(kp);
    w.encrypt(pass.as_bytes())
        .expect("encrypt")
        .to_file(&cfg.wallet)
        .expect("write");
}

fn transfer<T>(cfg: &Cfg, r: Option<T>, from: String, to: String, amnt: u64) -> Result<()>
    where T: ::std::io::BufRead 
{
    let pass = getpass(r);
    let w = load_wallet(cfg, pass);
    let fpk = BASE32HEX_NOPAD.decode(from.as_bytes()).expect("from key");
    let tpk = BASE32HEX_NOPAD.decode(to.as_bytes()).expect("to key");
    let kix = w.find(vec_to_array(fpk))?;
    let msg = w.tx(kix, vec_to_array(tpk), amnt, 1);
    let s = net::socket()?;
    s.connect(cfg.host.clone())?;
    let mut num = 0;
    while num < 1 {
        net::write(&s, &[msg], &mut num)?;
    }
    Ok(())
}

fn balance(_addr: String) {}

fn list<T>(cfg: &Cfg, r: Option<T>)
    where T: ::std::io::BufRead 
{
    let pass = getpass(r);
    let w = load_wallet(cfg, pass);
    println!("wallet has {:?} keys", w.pubkeys.len());
    for k in w.pubkeys {
        let pretty = BASE32HEX_NOPAD.encode(&to32b(k));
        println!("key {:?}", pretty);
    }
}

pub fn rund(args: Vec<String>) {
    let nopass = None::<::std::io::Empty>;
    run(args, nopass);
}

pub fn run<T>(args: Vec<String>, reader: Option<T>)
    where T: ::std::io::BufRead 
{
    let program = args[0].clone();
    let mut cfg = Cfg {
        host: "loom.loomprotocol.com:12345".to_string(),
        wallet: "loom.wallet".to_string(),
    };
    let mut opts = Options::new();
    opts.optflag("c", "", "create a new address");
    opts.optflag("x", "", "transfer");
    opts.optflag("b", "", "check the balance of destination address");
    opts.optflag("l", "list", "list your addresses and balances");
    opts.optflag("h", "help", "print this help menu");
    opts.optopt(
        "H",
        "",
        "loom node address to use instead of loom.looprotocol.com:12345",
        "HOST:PORT",
    );
    opts.optopt("W", "", "loom wallet instead of loom.wallet", "PATH");
    opts.optopt("t", "", "destination address", "ADDRESS");
    opts.optopt("f", "", "source address", "ADDRESS");
    opts.optopt("a", "", "amount", "AMOUNT");
    let matches = match opts.parse(&args[1..]) {
        Ok(m) => m,
        Err(f) => panic!(f.to_string()),
    };
    if matches.opt_present("h") {
        print_usage(&program, opts);
        return;
    }
    if matches.opt_present("H") {
        cfg.host = matches.opt_str("H").expect("loom host address");
    }
    if matches.opt_present("W") {
        cfg.wallet = matches.opt_str("W").expect("loom wallet path");
    }
    if matches.opt_present("c") {
        new_key_pair(&cfg, reader);
        return;
    } else if matches.opt_present("x") {
        let to = matches.opt_str("t").expect("missing destination address");
        let from = matches.opt_str("f").expect("missing source address");
        let astr = matches.opt_str("a").expect("missing ammount");
        let a = astr.parse().expect("ammount is not a number");
        transfer(&cfg, reader, to, from, a).expect("transfer");
        return;
    } else if matches.opt_present("b") {
        let to = matches.opt_str("t").expect("missing destination address");
        balance(to);
        return;
    } else if matches.opt_present("l") {
        list(&cfg, reader);
    }
}

#[cfg(test)]
mod tests {
    use client;
    use std::io::Cursor;

    #[test]
    fn help_test() {
        client::rund(vec!["loom".into(), "-h".into()]);
        client::rund(vec!["loom".into()]);
    }
    fn pass() -> Option<Cursor<&'static [u8]>> {
        Some(Cursor::new(&b"foobar\n"[..]))
    }
    #[test]
    fn list_test() {
        let args = vec!["loom".into(), "-W".into(),
                        "testdata/loom.wallet".into()];
        client::run(args, pass());
    }
}

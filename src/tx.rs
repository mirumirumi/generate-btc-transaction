use std::str::FromStr;

use bitcoin::{
    absolute::LockTime,
    address::Address,
    blockdata::{
        script::ScriptBuf,
        transaction::{Sequence, Transaction, TxIn, TxOut},
        witness::Witness,
    },
    consensus::encode::serialize,
    hashes::{hex::FromHex, Hash},
    secp256k1::{self, ecdsa::Signature, Context, Secp256k1, SecretKey, Signing},
    sighash::SighashCache,
    OutPoint,
    PrivateKey,
    PublicKey,
    Txid,
};

use crate::args::Args;

const SIGHASH_ALL: u8 = 0x01;
const INPUT_INDEX: usize = 0;
const FEE: u64 = 1000; // sathoshi

pub struct Tx(Transaction);

impl Tx {
    pub fn output(&self) -> String {
        let raw: Vec<u8> = serialize(&self.0);
        format!("0x{}", hex::encode(raw))
    }
}

pub struct TxBuilder<C: Context + Signing> {
    transaction: Option<Transaction>,
    private_key: PrivateKey,
    public_key: PublicKey,
    secp: Secp256k1<C>,
    utxo_txid: Txid,
    utxo_tx_index: u32,
    utxo_script_pubkey: ScriptBuf,
    output_script_pubkey: ScriptBuf,
    change_script_pubkey: ScriptBuf,
    send_amount: u64,
    utxo_amount: u64,
}

impl<C: Context + Signing> TxBuilder<C> {
    pub fn new(args: &Args) -> Result<Self, anyhow::Error> {
        let private_key = PrivateKey::from_wif(&args.private_key)?;

        let secp = Secp256k1::gen_new();
        let public_key = private_key.public_key(&secp);

        let mut bytes = Vec::<u8>::from_hex(&args.utxo_txid)?;
        bytes.reverse();
        let utxo_txid = Txid::from_slice(&bytes)?;
        let utxo_tx_index = args.utxo_tx_index;

        let bytes = Vec::<u8>::from_hex(&args.utxo_script_pubkey)?;
        let utxo_script_pubkey = ScriptBuf::from_bytes(bytes);

        // ScriptPubKey for destination output
        let dest_address = Address::from_str(&args.destination_address)?.assume_checked();
        let output_script_pubkey = dest_address.script_pubkey();

        // ScriptPubKey for change output
        let source_address = Address::from_str(&args.source_address)?.assume_checked();
        let change_script_pubkey = source_address.script_pubkey();

        Ok(Self {
            transaction: None,
            private_key,
            public_key,
            secp,
            utxo_txid,
            utxo_tx_index,
            utxo_script_pubkey,
            output_script_pubkey,
            change_script_pubkey,
            send_amount: args.send_amount,
            utxo_amount: args.utxo_amount,
        })
    }

    pub fn create_without_sig(&mut self) -> Result<&mut Self, anyhow::Error> {
        self.transaction = Some(Transaction {
            version: 1,
            lock_time: LockTime::ZERO,
            input: vec![TxIn {
                previous_output: OutPoint {
                    txid: self.utxo_txid,
                    vout: self.utxo_tx_index,
                },
                script_sig: ScriptBuf::new(),
                sequence: Sequence::MAX,
                witness: Witness::new(),
            }],
            output: vec![
                TxOut {
                    value: self.send_amount,
                    script_pubkey: self.output_script_pubkey.clone(),
                },
                // Change output
                TxOut {
                    value: self.calc_change_amount(),
                    script_pubkey: self.change_script_pubkey.clone(),
                },
            ],
        });

        Ok(self)
    }

    pub fn sign(&mut self) -> Result<&mut Self, anyhow::Error> {
        let transaction = self.transaction.clone().unwrap();
        let sighash = SighashCache::new(&transaction).legacy_signature_hash(
            INPUT_INDEX,
            &self.utxo_script_pubkey,
            SIGHASH_ALL as u32,
        )?;
        let message = secp256k1::Message::from_slice(&sighash[..])?;
        let secret_key = SecretKey::from_slice(&self.private_key.to_bytes())?;
        let signature = self.secp.sign_ecdsa(&message, &secret_key);

        let script_sig = Self::create_script_sig(&signature, &self.public_key);
        self.transaction.as_mut().unwrap().input[0].script_sig = ScriptBuf::from(script_sig);

        Ok(self)
    }

    pub fn build(&self) -> Tx {
        Tx(self.transaction.clone().unwrap())
    }

    fn calc_change_amount(&self) -> u64 {
        self.utxo_amount - self.send_amount - FEE
    }

    fn create_script_sig(signature: &Signature, public_key: &PublicKey) -> Vec<u8> {
        let mut script_sig = Vec::new();

        let serialized_sig = signature.serialize_der();
        script_sig.push((serialized_sig.len() as u8) + SIGHASH_ALL);
        script_sig.extend_from_slice(&serialized_sig);
        script_sig.push(SIGHASH_ALL);

        let serialized_pubkey = public_key.to_bytes();
        script_sig.push(serialized_pubkey.len() as u8);
        script_sig.extend_from_slice(&serialized_pubkey);

        script_sig
    }
}

#[cfg(test)]
mod tests {
    use bitcoin::secp256k1::All;
    use rand::{Rng, SeedableRng};
    use rand_pcg::Pcg64;
    use rstest::*;

    use super::*;

    #[rstest]
    #[case(Args {
        source_address: "mm8Wx3H3b3est26kxN1XY6sTnYNkxX16Lx".to_string(),
        destination_address: "mvygY8USGWGp3pnTRzfgWPzoaarZ9q74gn".to_string(),
        private_key: "cNmBYajCpAPzGL4VdxjM3qUGWpeasGu2RSAk5QjHnujZVVRuDLJP".to_string(),
        send_amount: 100,
        utxo_txid: "d73ebea9ad590316b5fbae5a176937178cdba72c1422a1636817a8f864a9c331".to_string(),
        utxo_tx_index: 1,
        utxo_amount: 4847873,
        utxo_script_pubkey: "76a9143d927250d4a4744f5f99b499f750d85054dbf9fc88ac".to_string(),
    }, true)]
    #[case(Args {
        source_address: "あ".to_string(),
        destination_address: "mvygY8USGWGp3pnTRzfgWPzoaarZ9q74gn".to_string(),
        private_key: "cNmBYajCpAPzGL4VdxjM3qUGWpeasGu2RSAk5QjHnujZVVRuDLJP".to_string(),
        send_amount: 100,
        utxo_txid: "d73ebea9ad590316b5fbae5a176937178cdba72c1422a1636817a8f864a9c331".to_string(),
        utxo_tx_index: 1,
        utxo_amount: 4847873,
        utxo_script_pubkey: "76a9143d927250d4a4744f5f99b499f750d85054dbf9fc88ac".to_string(),
    }, false)]
    #[case(Args {
        source_address: "mm8Wx3H3b3est26kxN1XY6sTnYNkxX16Lx".to_string(),
        destination_address: "mvygY8USGWGp3pnTRzfgWPzoaarZ9q74gn".to_string(),
        private_key: "い".to_string(),
        send_amount: 100,
        utxo_txid: "d73ebea9ad590316b5fbae5a176937178cdba72c1422a1636817a8f864a9c331".to_string(),
        utxo_tx_index: 1,
        utxo_amount: 4847873,
        utxo_script_pubkey: "76a9143d927250d4a4744f5f99b499f750d85054dbf9fc88ac".to_string(),
    }, false)]
    #[case(Args {
        source_address: "mm8Wx3H3b3est26kxN1XY6sTnYNkxX16Lx".to_string(),
        destination_address: "mvygY8USGWGp3pnTRzfgWPzoaarZ9q74gn".to_string(),
        private_key: "cNmBYajCpAPzGL4VdxjM3qUGWpeasGu2RSAk5QjHnujZVVRuDLJP".to_string(),
        send_amount: 100,
        utxo_txid: "う".to_string(),
        utxo_tx_index: 1,
        utxo_amount: 4847873,
        utxo_script_pubkey: "76a9143d927250d4a4744f5f99b499f750d85054dbf9fc88ac".to_string(),
    }, false)]
    #[case(Args {
        source_address: "mm8Wx3H3b3est26kxN1XY6sTnYNkxX16Lx".to_string(),
        destination_address: "mvygY8USGWGp3pnTRzfgWPzoaarZ9q74gn".to_string(),
        private_key: "cNmBYajCpAPzGL4VdxjM3qUGWpeasGu2RSAk5QjHnujZVVRuDLJP".to_string(),
        send_amount: 100,
        utxo_txid: "d73ebea9ad590316b5fbae5a176937178cdba72c1422a1636817a8f864a9c331".to_string(),
        utxo_tx_index: 1,
        utxo_amount: 4847873,
        utxo_script_pubkey: "え".to_string(),
    }, false)]
    fn test_new(#[case] args: Args, #[case] expected: bool) {
        assert_eq!(TxBuilder::<All>::new(&args).is_ok(), expected)
    }

    #[rstest]
    #[case(10_000, 500, 8_500)]
    #[case(1_500, 500, 0)]
    #[should_panic]
    #[case(10, 100, 0)]
    fn test_calc_change_amount(
        #[case] utxo_amount: u64,
        #[case] send_amount: u64,
        #[case] expected: u64,
    ) {
        let args = Args {
            source_address: "mm8Wx3H3b3est26kxN1XY6sTnYNkxX16Lx".to_string(),
            destination_address: "mvygY8USGWGp3pnTRzfgWPzoaarZ9q74gn".to_string(),
            private_key: "cNmBYajCpAPzGL4VdxjM3qUGWpeasGu2RSAk5QjHnujZVVRuDLJP".to_string(),
            send_amount,
            utxo_txid: "d73ebea9ad590316b5fbae5a176937178cdba72c1422a1636817a8f864a9c331"
                .to_string(),
            utxo_tx_index: 1,
            utxo_amount,
            utxo_script_pubkey: "76a9143d927250d4a4744f5f99b499f750d85054dbf9fc88ac".to_string(),
        };
        let tx_builder = TxBuilder::<All>::new(&args).unwrap();
        assert_eq!(tx_builder.calc_change_amount(), expected)
    }

    #[rstest]
    // ECDSA Signature: 70-72 bytes
    // SIGHASH_ALL: 1 byte
    // Public Key: 33 bytes or 65 bytes
    // MIN: 70 + 1 + 33 = 104
    #[case(prepare_test_create_script_sig(1), 104)]
    #[case(prepare_test_create_script_sig(2), 104)]
    #[case(prepare_test_create_script_sig(3), 104)]
    fn test_create_script_sig(
        #[case] params: (Signature, PublicKey),
        #[case] expected_min_len: usize,
    ) {
        assert!(expected_min_len <= TxBuilder::<All>::create_script_sig(&params.0, &params.1).len())
    }

    fn prepare_test_create_script_sig(seed: u64) -> (Signature, PublicKey) {
        // Make random `Args` based on seed value for test cases

        let mut rng = Pcg64::seed_from_u64(seed);
        let hexadecimal_chars = "abcdef0123456789";

        let args = Args {
            source_address: "mm8Wx3H3b3est26kxN1XY6sTnYNkxX16Lx".to_string(),
            destination_address: "mvygY8USGWGp3pnTRzfgWPzoaarZ9q74gn".to_string(),
            private_key: "cNmBYajCpAPzGL4VdxjM3qUGWpeasGu2RSAk5QjHnujZVVRuDLJP".to_string(),
            send_amount: rng.gen_range(100..1000),
            utxo_txid: random_string(&mut rng, 64, hexadecimal_chars),
            utxo_tx_index: rng.gen::<u32>(),
            utxo_amount: rng.gen_range(5000..20000),
            utxo_script_pubkey: random_string(&mut rng, 50, hexadecimal_chars),
        };

        let mut tx_builder = TxBuilder::<All>::new(&args).unwrap();
        let tx_builder = tx_builder.create_without_sig().unwrap();
        let tx_builder = tx_builder.sign().unwrap();

        let private_key = tx_builder.private_key;
        let public_key = private_key.public_key(&tx_builder.secp);

        let transaction = tx_builder.transaction.as_ref().unwrap();
        let sighash = SighashCache::new(transaction)
            .legacy_signature_hash(
                INPUT_INDEX,
                &tx_builder.utxo_script_pubkey,
                SIGHASH_ALL as u32,
            )
            .unwrap();
        let message = secp256k1::Message::from_slice(&sighash[..]).unwrap();
        let secret_key = SecretKey::from_slice(&private_key.to_bytes()).unwrap();
        let signature = tx_builder.secp.sign_ecdsa(&message, &secret_key);

        (signature, public_key)
    }

    fn random_string(rng: &mut Pcg64, length: usize, chars: &str) -> String {
        let mut result = String::with_capacity(length);
        for _ in 0..length {
            let index = rng.gen_range(0..chars.len());
            let character = chars.chars().nth(index).unwrap();
            result.push(character);
        }
        result
    }
}

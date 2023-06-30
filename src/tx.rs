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
    secp256k1::{self, Context, Secp256k1, SecretKey, Signing},
    sighash::SighashCache,
    OutPoint,
    PrivateKey,
    PublicKey,
    Txid,
};

use crate::args::Args;

const SIGHASH_ALL: u8 = 0x01;
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
                    value: self.utxo_amount - self.send_amount - FEE,
                    script_pubkey: self.change_script_pubkey.clone(),
                },
            ],
        });

        Ok(self)
    }

    pub fn sign(&mut self) -> Result<&mut Self, anyhow::Error> {
        let transaction = self.transaction.clone().unwrap();
        let sighash_cache = SighashCache::new(&transaction);
        let sighash =
            sighash_cache.legacy_signature_hash(0, &self.utxo_script_pubkey, SIGHASH_ALL as u32)?;

        let message = secp256k1::Message::from_slice(&sighash[..])?;
        let secret_key = SecretKey::from_slice(&self.private_key.to_bytes())?;
        let signature = self.secp.sign_ecdsa(&message, &secret_key);

        let mut script_sig = Vec::new();

        let serialized_sig = signature.serialize_der();
        script_sig.push((serialized_sig.len() as u8) + SIGHASH_ALL);
        script_sig.extend_from_slice(&serialized_sig);
        script_sig.push(SIGHASH_ALL);

        let serialized_pubkey = self.public_key.to_bytes();
        script_sig.push(serialized_pubkey.len() as u8);
        script_sig.extend_from_slice(&serialized_pubkey);

        self.transaction.as_mut().unwrap().input[0].script_sig = ScriptBuf::from(script_sig);

        Ok(self)
    }

    pub fn build(&self) -> Tx {
        Tx(self.transaction.clone().unwrap())
    }
}


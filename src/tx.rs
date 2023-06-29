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
    secp256k1::{self, Secp256k1, SecretKey},
    sighash::SighashCache,
    OutPoint,
    PrivateKey,
    Txid,
};

use crate::args::Args;

const SIGHASH_ALL: u8 = 0x01;
const FEE: u64 = 1000; // sathoshi

pub struct Tx(Transaction);

impl Tx {
    pub fn new(args: Args) -> Result<Self, anyhow::Error> {
        // 送信者の秘密鍵
        let private_key = PrivateKey::from_wif(&args.private_key)?;

        // 送信者の公開鍵
        let secp = Secp256k1::new();
        let public_key = private_key.public_key(&secp);

        // UTXO のトランザクション ID
        let mut bytes = Vec::<u8>::from_hex(&args.utxo_txid)?;
        bytes.reverse();
        let utxo_txid = Txid::from_slice(&bytes)?;

        // UTXO の ScriptPubKey
        let bytes = Vec::<u8>::from_hex(&args.utxo_script_pubkey)?;
        let utxo_script_pubkey = ScriptBuf::from_bytes(bytes);

        // 今回作成するアウトプット用の ScriptPubKey
        let dest_address = Address::from_str(&args.destination_address)?.assume_checked();
        let output_script_pubkey = dest_address.script_pubkey();

        // お釣りアウトプット用の ScriptPubKey
        let source_address = Address::from_str(&args.source_address)?.assume_checked();
        let change_script_pubkey = source_address.script_pubkey();

        let mut transaction = Transaction {
            version: 1,
            lock_time: LockTime::ZERO,
            input: vec![TxIn {
                previous_output: OutPoint {
                    txid: utxo_txid,
                    vout: args.utxo_tx_index,
                },
                script_sig: ScriptBuf::new(),
                sequence: Sequence::MAX,
                witness: Witness::new(),
            }],
            output: vec![
                TxOut {
                    value: args.send_amount,
                    script_pubkey: output_script_pubkey,
                },
                TxOut {
                    value: args.utxo_amount - args.send_amount - FEE,
                    script_pubkey: change_script_pubkey,
                },
            ],
        };

        // 署名処理
        let sighash_cache = SighashCache::new(&transaction);
        let sighash =
            sighash_cache.legacy_signature_hash(0, &utxo_script_pubkey, SIGHASH_ALL as u32)?;

        let message = secp256k1::Message::from_slice(&sighash[..])?;
        let secret_key = SecretKey::from_slice(&private_key.to_bytes())?;
        let signature = secp.sign_ecdsa(&message, &secret_key);

        // ScriptSig に署名結果および自分の公開鍵を組み込む
        let mut script_sig = Vec::new();

        let serialized_sig = signature.serialize_der();
        script_sig.push((serialized_sig.len() as u8) + SIGHASH_ALL);
        script_sig.extend_from_slice(&serialized_sig);
        script_sig.push(SIGHASH_ALL);

        let serialized_pubkey = public_key.to_bytes();
        script_sig.push(serialized_pubkey.len() as u8);
        script_sig.extend_from_slice(&serialized_pubkey);

        transaction.input[0].script_sig = ScriptBuf::from(script_sig);

        Ok(Tx(transaction))
    }

    pub fn generate(&self) -> String {
        let raw: Vec<u8> = serialize(&self.0);
        format!("0x{}", hex::encode(raw))
    }
}


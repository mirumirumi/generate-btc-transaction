use anyhow::ensure;
use clap::Parser;

const BASE58_CHARS: &str = "ABCDEFGHJKLMNPQRSTUVWXYZabcdefghijkmnopqrstuvwxyz123456789";

#[derive(Debug, Parser, Default)]
#[command(author, version, about, long_about = None)]
pub struct Args {
    /// Source BTC address
    #[arg(short = 's', long)]
    pub source_address: String,

    /// Destination BTC address
    #[arg(short = 'd', long)]
    pub destination_address: String,

    /// Your Private key (WIF/P2PKH, without `p2pkh:` prefix)
    #[arg(short = 'p', long)]
    pub private_key: String,

    /// Amount to send (satoshi)
    #[arg(short = 'a', long)]
    pub send_amount: u64,

    /// UTXO transaction ID
    #[arg(short = 't', long)]
    pub utxo_txid: String,

    /// UTXO transaction index
    #[arg(short = 'i', long)]
    pub utxo_tx_index: u32,

    /// Amount in UTXO (satoshi)
    #[arg(short = 'u', long)]
    pub utxo_amount: u64,

    /// ScriptPubKey in UTXO
    #[arg(short = 'k', long)]
    pub utxo_script_pubkey: String,
}

impl Args {
    pub fn validate(&self) -> Result<(), anyhow::Error> {
        // Check Base58 encoding
        ensure!(
            self.is_base58(&self.source_address),
            "`--source-address` must be a base58 encoded"
        );
        ensure!(
            self.is_base58(&self.destination_address),
            "`--destination-address` must be a base58 encoded"
        );
        ensure!(
            self.is_base58(&self.private_key),
            "`--private-key` must be a base58 encoded"
        );

        // Check string length
        ensure!(
            (27 <= self.source_address.len() && self.source_address.len() <= 34)
                && (27 <= self.destination_address.len() && self.destination_address.len() <= 34),
            "BTC address must have between 27 and 34 characters"
        );
        ensure!(
            (51 <= self.private_key.len()) && (self.private_key.len() <= 52),
            "`--private-key` must have between 51 and 52 characters"
        );
        ensure!(
            self.utxo_txid.len() == 64,
            "`--utxo-txid` must have 64 characters"
        );

        // Check hexadecimal encoding
        ensure!(
            self.is_hexadecimals(self.utxo_txid.as_str()),
            "`--utxo-txid` must be a hexadecimal string"
        );
        ensure!(
            self.is_hexadecimals(self.utxo_script_pubkey.as_str()),
            "`--utxo-script-pubkey` must be a hexadecimal string"
        );

        Ok(())
    }

    fn is_hexadecimals(&self, value: &str) -> bool {
        value.chars().all(|c| c.is_ascii_hexdigit())
    }

    fn is_base58(&self, value: &str) -> bool {
        value.chars().all(|c| BASE58_CHARS.contains(c))
    }
}

#[cfg(test)]
mod tests {
    use rstest::*;

    use super::*;

    #[rstest]
    #[case("0123456789abcdef", true)]
    #[case("z", false)]
    #[case("$", false)]
    fn test_is_hexadecimals(#[case] value: &str, #[case] expected: bool) {
        let args = Args::default();
        assert_eq!(args.is_hexadecimals(value), expected)
    }

    #[rstest]
    #[case("1PMycacnJaSqwwJqjawXBErnLsZ7RkXUAs", true)]
    #[case("O", false)]
    #[case("l", false)]
    #[case("$", false)]
    fn test_is_base58(#[case] value: &str, #[case] expected: bool) {
        let args = Args::default();
        assert_eq!(args.is_base58(value), expected)
    }
}

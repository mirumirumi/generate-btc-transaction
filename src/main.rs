use bitcoin::secp256k1::All;
use clap::Parser;

mod args;
mod tx;

use crate::{args::Args, tx::TxBuilder};

fn main() -> Result<(), anyhow::Error> {
    let args = Args::parse();
    args.validate()?;

    let tx = TxBuilder::<All>::new(&args)?
        .create_without_sig()?
        .sign()?
        .build();

    println!("{}", tx.output());

    Ok(())
}

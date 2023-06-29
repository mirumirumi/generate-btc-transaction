use clap::Parser;

mod args;
mod tx;

use crate::{args::Args, tx::Tx};

fn main() -> Result<(), anyhow::Error> {
    let args = Args::parse();
    args.validate()?;

    let tx = Tx::new(args)?;

    println!("{}", tx.generate());

    Ok(())
}

use std::{
    fs::File,
    io::{self, Write},
    path::PathBuf,
};

use anyhow::{Context, Result};
use clap::Parser;

use crate::{data_structures::Transaction, services::AccountService};

mod data_structures;
mod services;

#[derive(Debug, Parser)]
struct Args {
    #[arg(
        help = "Path to the transactions .csv file",
        index = 1,
        required = true
    )]
    pub transactions_file: PathBuf,
    #[arg(
        help = "Whether to log errors to the stdout",
        short = 'e',
        long = "log-errors",
        default_value = "false"
    )]
    pub log_errors: bool,
}

fn main() -> Result<()> {
    let args = Args::parse();

    let stdout = io::stdout();
    let mut stdout_writer = io::BufWriter::new(stdout);

    if !args.transactions_file.exists() {
        panic!(
            "transaction file '{}' doesn't exist",
            args.transactions_file.display()
        );
    }
    if !args.transactions_file.is_file() {
        panic!("'{}' is not a file", args.transactions_file.display());
    }

    let mut account_service = AccountService::new();

    let transactions_file =
        File::open(args.transactions_file).context("failed to open transactions file")?;

    let mut transactions_reader = csv::ReaderBuilder::new()
        .trim(csv::Trim::All)
        .from_reader(transactions_file);

    for (idx, transaction_res) in transactions_reader.deserialize::<Transaction>().enumerate() {
        // we add 1 to the index because the first line is the header
        let row_number = idx + 1;

        let transaction = match transaction_res {
            Ok(v) => v,
            Err(err) => {
                if args.log_errors {
                    let _ = writeln!(stdout_writer, "error parsing row {row_number}: {err:?}");
                    let _ = stdout_writer.flush();
                }
                continue;
            }
        };

        account_service.record_transaction(transaction);
    }

    let mut csv_writer = csv::WriterBuilder::new()
        .has_headers(true)
        .from_writer(stdout_writer);
    for (_, account) in account_service.summary() {
        csv_writer.serialize(account)?;
    }
    csv_writer.flush().context("flush account summary as csv")?;

    Ok(())
}

use std::env;
use std::io::{read_to_string, BufReader, stdin};
use std::fs::File;

use color_print::cprintln;

#[derive(Debug)]
struct TransactionLog {
    transaction: String,
    contents: Vec<String>,
}

impl  TransactionLog {
    fn show(&self) {
        cprintln!("<yellow>Transaction is:</yellow>");
        cprintln!("<cyan>{}</cyan>", self.transaction);
        cprintln!("<yellow>Transaction Logs</yellow>");
        for line in &self.contents {
            if line.contains("dump_onode") || line.contains("dump_extent_map") {
                cprintln!("<magenta>{:?}</magenta>", line);
            }
        }
    }
    
}

fn transaction_bisect(transactions: &Vec<TransactionLog>) {
    let mut l = 0;
    let mut r = transactions.len();
    let input = stdin();

    let mut buf = String::new();
    while l < r {
        let m: usize = (l + r) / 2;
        let mut bad_input = true;
        let transaction = &transactions[m];
        transaction.show();
        cprintln!("Please type <red>\"bad\"</red> if the transaction did bad stuff or <green>\"good\"</green> if everything looks normal");
        while bad_input {
            buf.clear();
            input.read_line(&mut buf).unwrap();
            println!("{}",buf);
            match buf.as_str() {
                "good\n" => {
                    l = m;
                    bad_input = false;
                }, 
                "bad\n" => {
                    r = m;
                    bad_input = false;
                }, 
                _ => {
                    println!("Please type \"bad\" or \"good\"");
                    bad_input = true;
                }
            }
        }
    }

}

fn main() {
    let args: Vec<String> = env::args().collect();
    let file: File = File::open(args[1].to_string()).unwrap();
    let buf_reader = BufReader::new(file);
    let contents = read_to_string(buf_reader).unwrap();
    let mut current_transaction = TransactionLog{transaction: String::new(), contents: Vec::new()};
    let mut transactions: Vec<TransactionLog> = Vec::new();
    let mut transaction_start = false;
    for line in contents.split('\n') {
        if line.contains("dump_transaction") {
            transactions.push(current_transaction);
            current_transaction = TransactionLog{transaction: String::new(), contents: Vec::new()};
        }
        if line == "{" {
            transaction_start = true;
            current_transaction.transaction.push_str(line);
            current_transaction.transaction.push('\n');
        }
        if transaction_start && line == "}" {
            transaction_start = false;
            current_transaction.transaction.push_str(line);
            current_transaction.transaction.push('\n');
            
        }
        if transaction_start {
            current_transaction.transaction.push_str(line);
            current_transaction.transaction.push('\n');
        } else {
            current_transaction.contents.push(line.to_string());

        }
    }
    // push last
    transactions.push(current_transaction);

    transaction_bisect(&transactions);
    dbg!(args);
}

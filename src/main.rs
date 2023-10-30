use std::env;
use std::fs::File;
use std::io::{read_to_string, stdin, BufReader};

use color_print::cprintln;
use serde_json::Value;

#[derive(Debug)]
struct TransactionLog {
    transaction: Value,
    contents: Vec<String>,
}

impl TransactionLog {
    fn show(&self, function_filters: &Vec<String>) {
        cprintln!("<yellow>Transaction is:</yellow>");
        cprintln!("<yellow>Filters are: {:?}</yellow>", function_filters);
        let cute_transaction = serde_json::to_string_pretty(&self.transaction).unwrap();
        cprintln!("<cyan>{}</cyan>", cute_transaction);
        cprintln!("<yellow>Transaction Logs</yellow>");
        for line in &self.contents {
            let do_print = false;
            for filter in function_filters {
                if line.contains(filter) {
                    cprintln!("<magenta>{:?}</magenta>", line);
                    break;
                }
            }
        }
    }
}

fn transaction_bisect(transactions: &Vec<TransactionLog>) {
    let mut l = 0;
    let mut r = transactions.len();
    let input = stdin();

    let mut buf = String::new();
    let mut function_filters: Vec<String> = Vec::new();
    while l < r {
        let m: usize = (l + r) / 2;
        let mut bad_input = true;
        let transaction = &transactions[m];
        transaction.show(&function_filters);
        cprintln!("Commands:");
        cprintln!("\t<red>bad</red>\t mark transaction as looks problematic");
        cprintln!("\t<green>good</green>\t mark transaction as looks good");
        cprintln!("\t<bold>filter [function,]*</bold>");
        buf.clear();
        input.read_line(&mut buf).unwrap();
        println!("{}", buf);
        match buf.as_str() {
            "good\n" => {
                l = m;
            }
            "bad\n" => {
                r = m;
            }
            v => {
                if v.starts_with("filter") {
                    if let Some(function) = v.split(' ').nth(1) {
                        let filters = function.to_string().strip_suffix("\n").unwrap().to_string();
                        function_filters = filters.split(',').map(|v| v.to_string()).collect();
                    }
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
    let mut current_transaction = TransactionLog {
        transaction: Value::Null,
        contents: Vec::new(),
    };
    let mut transactions: Vec<TransactionLog> = Vec::new();
    let mut transaction_start = false;
    let mut current_transaction_str = String::new();
    for line in contents.split('\n') {
        if line.contains("dump_transaction") {
            transactions.push(current_transaction);
            current_transaction = TransactionLog {
                transaction: Value::Null,
                contents: Vec::new(),
            };
        }
        if line == "{" {
            transaction_start = true;
            current_transaction_str.clear();
            current_transaction_str.push_str(line);
            current_transaction_str.push('\n');
        } else if transaction_start && line == "}" {
            transaction_start = false;
            current_transaction_str.push_str(line);
            current_transaction_str.push('\n');
            println!("sdf {}", current_transaction_str);
            println!("sdf");
            current_transaction.transaction =
                serde_json::from_str(&current_transaction_str.as_str()).unwrap();
        } else {
            if transaction_start {
                current_transaction_str.push_str(line);
                current_transaction_str.push('\n');
            } else {
                current_transaction.contents.push(line.to_string());
            }
        }
    }
    // push last
    transactions.push(current_transaction);

    transaction_bisect(&transactions);
    dbg!(args);
}

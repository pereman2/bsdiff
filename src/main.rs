use std::cmp::max;
use std::env;
use std::fs::File;
use std::io::{read_to_string, stdin, BufReader};
use std::str::FromStr;

use chrono::{DateTime, FixedOffset, Utc};
use color_print::cprintln;
use serde_json::Value;

#[derive(Debug)]
struct LogLine {
    date: DateTime<FixedOffset>,
    idk: String,
    log_level: String,
    context: String,
    function: String,
    log: String,
}
#[derive(Debug)]
struct TransactionLog {
    transaction: Value,
    contents: Vec<LogLine>,
}

impl TransactionLog {
    fn show(&self, function_filters: &Vec<String>) {
        cprintln!("<yellow>Transaction is:</yellow>");
        cprintln!("<yellow>Filters are: {:?}</yellow>", function_filters);
        let cute_transaction = serde_json::to_string_pretty(&self.transaction).unwrap();
        cprintln!("<cyan>{}</cyan>", cute_transaction);
        cprintln!("<yellow>Transaction Logs</yellow>");
        for line in &self.contents {
            if function_filters.len() > 0 {
                for filter in function_filters {
                    // a line might have context or not :()
                    if line.function.contains(filter) || line.context.contains(filter) {
                        cprintln!(
                            "<cyan>{}</> <magenta>{}</> {}",
                            line.context,
                            line.function,
                            line.log
                        );
                        break;
                    }
                }
            } else {
                cprintln!(
                    "<cyan>{}</> <magenta>{}</> {}",
                    line.context,
                    line.function,
                    line.log
                );
            }
        }
    }
}

fn explore(transactions: &Vec<TransactionLog>) {
    let mut run = true;
    let mut current: usize = transactions.len() - 1;

    let input = stdin();
    let mut buf = String::new();
    let mut function_filters: Vec<String> = Vec::new();

    while run {
        let transaction = &transactions[current];
        transaction.show(&function_filters);
        cprintln!("Trasaction: {} / {}", current + 1, transactions.len());
        cprintln!("Commands:");
        cprintln!("\t<bold>next [jump]</bold>\t jump is the amount of jumps you want to perform");
        cprintln!("\t<bold>prev [jump]</bold>\t jump is the amount of jumps you want to perform");
        cprintln!("\t<bold>filter [function,]*</bold>");
        buf.clear();
        input.read_line(&mut buf).unwrap();
        println!("{}", buf);
        match buf.as_str() {
            "next\n" => {}
            "prev\n" => {}
            v => {
                if v.starts_with("filter") {
                    if let Some(function) = v.split(' ').nth(1) {
                        let filters = function.to_string().strip_suffix("\n").unwrap().to_string();
                        function_filters = filters.split(',').map(|v| v.to_string()).collect();
                    }
                } else if v.starts_with("next") {
                    let mut jump: usize = 1;
                    if let Some(j) = v.split(' ').nth(1) {
                        let mut j = j.to_string();
                        j.truncate(j.len() - 1);
                        match j.parse() {
                            Ok(v) => jump = v,
                            Err(_) => {
                                println!("invalid jump value, should an number");
                            }
                        }
                    }
                    current = (current + jump).min(transactions.len() - 1);
                } else if v.starts_with("prev") {
                    let mut jump: usize = 1;
                    if let Some(j) = v.split(' ').nth(1) {
                        let mut j = j.to_string();
                        j.truncate(j.len() - 1);
                        match j.parse() {
                            Ok(v) => jump = v,
                            Err(_) => {
                                println!("invalid jump value, should an number");
                            }
                        }
                    }
                    current = (current.saturating_sub(jump)).max(0);
                }
            }
        }
    }
}

fn biscect(transactions: &Vec<TransactionLog>) {
    let mut l = 0;
    let mut r = transactions.len();
    let input = stdin();

    let mut buf = String::new();
    let mut function_filters: Vec<String> = Vec::new();
    while l < r {
        let m: usize = (l + r) / 2;
        let transaction = &transactions[m];
        transaction.show(&function_filters);
        cprintln!("Trasaction: {} / {}", m + 1, transactions.len());
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
    assert!(args.len() == 3);
    let action: String = args[1].to_string();
    let file: File = File::open(args[2].to_string()).unwrap();
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
                let mut split = line.split(' ');
                if let Some(date) = split.next() {
                    println!("{}", date);
                    if let Ok(date_parsed) =
                        DateTime::parse_from_str(date, "%Y-%m-%dT%H:%M:%S%.3f%:z")
                    {
                        println!("{:?}", date_parsed);
                        let idk = split.next().unwrap();
                        let log_level = split.next().unwrap();
                        let context = split.next().unwrap();
                        let function = split.next().unwrap();
                        let log = split.fold("".to_string(), |acc, v| acc + " " + v);
                        current_transaction.contents.push(LogLine {
                            date: date_parsed,
                            idk: idk.to_string(),
                            log_level: log_level.to_string(),
                            context: context.to_string(),
                            function: function.to_string(),
                            log: log,
                        });
                    }
                }
            }
        }
    }
    // push last
    transactions.push(current_transaction);
    match action.as_str() {
        "explore" => {
            explore(&transactions);
        }
        "bisect" => {
            biscect(&transactions);
        }
        _ => {
            println!("Unknown action");
        }
    }

    dbg!(args);
}

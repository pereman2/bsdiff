use std::collections::{HashSet};
use std::env;
use std::fs::File;
use std::io::{read_to_string, stdin, BufReader};


use chrono::{DateTime, FixedOffset};
use color_print::cprintln;
use serde_json::{json, Value};

#[derive(Debug, Clone)]
struct LogLine {
    date: DateTime<FixedOffset>,
    idk: String,
    log_level: String,
    context: String,
    function: String,
    log: String,
}
#[derive(Debug, Clone)]
struct TransactionOp {
    op_name: String,
    oid: String,

    src_oid: String,
    dst_oid: String,

    new_oid: String,
    old_oid: String,

    offset: usize,
    length: usize,

    src_offset: usize,
    dst_offset: usize,
    len: usize,
}

#[derive(Debug, Clone)]
struct TransactionLog {
    id: usize,
    transaction: Vec<TransactionOp>,
    raw_transaction: Value,
    contents: Vec<LogLine>,
}

impl TransactionLog {
    fn show(&self, function_filters: &Vec<String>) {
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
        cprintln!("<yellow>Transaction is:</yellow>");
        let cute_transaction = serde_json::to_string_pretty(&self.raw_transaction).unwrap();
        cprintln!("<cyan>{}</cyan>", cute_transaction);
    }
}

fn traceback_seen(
    target_oid: &String,
    transactions: &Vec<TransactionLog>,
    out: &mut HashSet<String>,
) {
    for log in transactions {
        for op in &log.transaction {
            if op.oid != "" {
                if target_oid.contains(&op.oid) {
                    out.insert(op.oid.to_string());
                }
            }
            if op.src_oid != "" {
                let oid = op.src_oid.to_string();
                if target_oid.contains(&oid) {
                    if !out.contains(&oid) {
                        out.insert(oid);
                    }
                    let other = op.dst_oid.to_string();
                    if !out.contains(&other) {
                        out.insert(other.to_string());
                        traceback_seen(&other, transactions, out);
                    }
                }
            }
            if op.dst_oid != "" {
                let oid = op.src_oid.to_string();
                if target_oid.contains(&oid) {
                    if !out.contains(&oid) {
                        out.insert(oid);
                    }
                    let other = op.src_oid.to_string();
                    if !out.contains(&other) {
                        out.insert(other.to_string());
                        traceback_seen(&other, transactions, out);
                    }
                }
            }
            if op.old_oid != "" {
                let oid = op.old_oid.to_string();
                if target_oid.contains(&oid) {
                    if !out.contains(&oid) {
                        out.insert(oid);
                    }
                    let other = op.new_oid.to_string();
                    if !out.contains(&other) {
                        out.insert(other.to_string());
                        traceback_seen(&other, transactions, out);
                    }
                }
            }
            if op.new_oid != "" {
                let oid = op.new_oid.to_string();
                if target_oid.contains(&oid) {
                    if !out.contains(&oid) {
                        out.insert(oid);
                    }
                    let other = op.old_oid.to_string();
                    if !out.contains(&other) {
                        out.insert(other.to_string());
                        traceback_seen(&other, transactions, out);
                    }
                }
            }
        }
    }
}

fn traceback(target_oid: &String, transactions: &Vec<TransactionLog>) -> Vec<String> {
    let mut out: HashSet<String> = HashSet::new();
    traceback_seen(target_oid, transactions, &mut out);
    let out: Vec<String> = out.iter().map(|v| v.to_owned()).collect();
    return out;
}

fn explore(transactions: &Vec<TransactionLog>) {
    let mut run = true;
    let mut current: usize = transactions.len() - 1;

    let input = stdin();
    let mut buf = String::new();
    let mut function_filters: Vec<String> = Vec::new();
    let mut oid_filters: Vec<String> = Vec::new();
    let mut filter_oids: bool = false;

    let mut filtered_transactions: Vec<TransactionLog> = transactions.clone();
    let mut skip_print = false;

    let mut bisect_mode = false;
    let mut start_bisect_mode = false;
    let mut continue_bisect = false;
    let mut m: usize = 0;
    let mut l: usize = 0;
    let mut r: usize = transactions.len();

    while run {
        if filter_oids {
            filtered_transactions = transactions
                .iter()
                .filter(|log| {
                    if oid_filters.len() == 0 {
                        return true;
                    }
                    for op in &log.transaction {
                        if op.oid != "" {
                            if oid_filters.contains(&op.oid) {
                                return true;
                            }
                        }
                        if op.new_oid != "" {
                            if oid_filters.contains(&op.new_oid) {
                                return true;
                            }
                        }
                        if op.old_oid != "" {
                            if oid_filters.contains(&op.old_oid) {
                                return true;
                            }
                        }
                        if op.src_oid != "" {
                            if oid_filters.contains(&op.src_oid) {
                                return true;
                            }
                        }
                        if op.dst_oid != "" {
                            if oid_filters.contains(&op.dst_oid) {
                                return true;
                            }
                        }
                    }
                    return false;
                })
                .map(|v| v.clone())
                .collect();
            current = filtered_transactions.len().saturating_sub(1);
            filter_oids = false;
        }

        if start_bisect_mode {
            l = 0;
            r = filtered_transactions.len() - 1;
            start_bisect_mode = false;
        }
        if bisect_mode && continue_bisect {
            m = (l + r) / 2;
            current = m;
            continue_bisect = false;
        }

        if filtered_transactions.len() > 0 && !skip_print {
            let transaction = &filtered_transactions[current];
            transaction.show(&function_filters);
        }

        if skip_print {
            skip_print = false;
        }
        cprintln!(
            "<yellow>Trasaction: {} / {}</>",
            current + 1,
            filtered_transactions.len()
        );
        cprintln!(
            "<yellow>Function filters are: {:?}</yellow>",
            function_filters
        );
        cprintln!("<yellow>Oid filters are: {:?}</yellow>", oid_filters);
        cprintln!("Explore commands:");
        cprintln!("\t<bold>next [jump]</bold>\t jump is the amount of jumps you want to perform.");
        cprintln!("\t<bold>prev [jump]</bold>\t jump is the amount of jumps you want to perform");

        println!();
        cprintln!("Filtering commands: (might reset bisect mode) ");
        cprintln!("\t<bold>filter [function,]*</bold>\t function filter of the log line. This won't reset bisect");
        cprintln!("\t<bold>oids [oid,]*</bold>\t oids to append to the oid filter");
        cprintln!("\t<bold>oids clear</bold>\t remove all oids");
        cprintln!("\t<bold>traceback</bold>\t trace oids back following clones, move, renames...");
        cprintln!("\t<bold>dump</bold>\t dumps all filtered transactions");

        println!();
        cprintln!("Bisect commands:");
        cprintln!("\t<red,bold>bad</>\t mark transaction as looks problematic");
        cprintln!("\t<green,bold>good</>\t mark transaction as looks good");
        cprintln!("\t<bold>bisect start</>");
        cprintln!("\t<bold>bisect end</>");

        buf.clear();
        input.read_line(&mut buf).unwrap();
        println!("{}", buf);
        match buf.as_str() {
            "traceback\n" => {
                let mut new_oid_filters = Vec::new();
                oid_filters.get(0).and_then(|v| {
                    Some({
                        new_oid_filters = traceback(&v, transactions);
                        filter_oids = true;
                        current = filtered_transactions.len() - 1;
                    })
                });
                oid_filters = new_oid_filters;
                if bisect_mode {
                    start_bisect_mode = true;
                }
            }
            "dump\n" => {
                cprintln!("<yellow,bold>Dump start</>");
                for transaction in &filtered_transactions {
                    transaction.show(&function_filters);
                }
                cprintln!("<yellow,bold>Dump end</>");
                skip_print = true;
            }
            "oids clear\n" => {
                oid_filters.clear();
                filter_oids = true;
                if bisect_mode {
                    start_bisect_mode = true;
                }
            }
            "bisect end\n" => {
                bisect_mode = false;
                current = filtered_transactions.len() - 1;
            }
            "bisect start\n" => {
                bisect_mode = true;
                if bisect_mode {
                    start_bisect_mode = true;
                    continue_bisect = true;
                }
            }
            "good\n" => {
                if bisect_mode {
                    l = m;
                    continue_bisect = true;
                } else {
                    cprintln!("<red>Please start bisect mode</>");
                    skip_print = true;
                }
            }
            "bad\n" => {
                if bisect_mode {
                    r = m;
                    continue_bisect = true;
                } else {
                    cprintln!("<red>Please start bisect mode</>");
                    skip_print = true;
                }
            }
            v => {
                if v.starts_with("oids") {
                    if let Some(oids) = v.split(' ').nth(1) {
                        let oids = oids.to_string().strip_suffix("\n").unwrap().to_string();
                        let mut extra_oid_filters: Vec<String> =
                            oids.split(',').map(|v| v.to_string()).collect();
                        oid_filters.append(&mut extra_oid_filters);
                        filter_oids = true;
                    }
                    if bisect_mode {
                        start_bisect_mode = true;
                    }
                } else if v.starts_with("filter") {
                    if let Some(function) = v.split(' ').nth(1) {
                        let filters = function.to_string().strip_suffix("\n").unwrap().to_string();
                        function_filters = filters.split(',').map(|v| v.to_string()).collect();
                    }
                } else if v.starts_with("next") {
                    if bisect_mode {
                        cprintln!("Please end bisect mode to run this operation");
                        continue;
                    }
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
                    current = (current + jump).min(filtered_transactions.len() - 1);
                } else if v.starts_with("prev") {
                    if bisect_mode {
                        cprintln!("Please end bisect mode to run this operation");
                        continue;
                    }
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

fn main() {
    let args: Vec<String> = env::args().collect();
    assert!(args.len() == 2);
    let file: File = File::open(args[1].to_string()).unwrap();
    let buf_reader = BufReader::new(file);
    let contents = read_to_string(buf_reader).unwrap();
    let mut current_transaction = TransactionLog {
        id: 0,
        transaction: Vec::new(),
        raw_transaction: Value::Null,
        contents: Vec::new(),
    };
    let mut transactions: Vec<TransactionLog> = Vec::new();
    let mut transaction_start = false;
    let mut current_transaction_str = String::new();
    let mut id = 1;
    for line in contents.split('\n') {
        if line.contains("dump_transaction") {
            transactions.push(current_transaction);
            current_transaction = TransactionLog {
                id: id,
                transaction: Vec::new(),
                raw_transaction: Value::Null,
                contents: Vec::new(),
            };
            id += 1;
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
            let transaction: Value =
                serde_json::from_str(&current_transaction_str.as_str()).unwrap();
            println!("transaction parse {}", current_transaction_str);
            current_transaction.raw_transaction = transaction.clone();
            if transaction.is_object() {
                let obj = transaction.as_object().unwrap();
                let ops = obj.get("ops").unwrap().as_array().unwrap();
                for op in ops {
                    let op = op.as_object().unwrap();

                    let default_int = json!(0 as u64);
                    let offset =
                        op.get("offset").unwrap_or(&default_int).as_u64().unwrap() as usize;
                    let dst_offset = op
                        .get("dst_offset")
                        .unwrap_or(&default_int)
                        .as_u64()
                        .unwrap() as usize;
                    let src_offset = op
                        .get("src_offset")
                        .unwrap_or(&default_int)
                        .as_u64()
                        .unwrap() as usize;
                    let length =
                        op.get("length").unwrap_or(&default_int).as_u64().unwrap() as usize;
                    let len = op.get("len").unwrap_or(&default_int).as_u64().unwrap() as usize;

                    let default_string = json!("");
                    let oid = op
                        .get("oid")
                        .unwrap_or(&default_string)
                        .as_str()
                        .unwrap()
                        .to_string();
                    let dst_oid = op
                        .get("dst_oid")
                        .unwrap_or(&default_string)
                        .as_str()
                        .unwrap()
                        .to_string();
                    let src_oid = op
                        .get("src_oid")
                        .unwrap_or(&default_string)
                        .as_str()
                        .unwrap()
                        .to_string();
                    let new_oid = op
                        .get("new_oid")
                        .unwrap_or(&default_string)
                        .as_str()
                        .unwrap()
                        .to_string();
                    let old_oid = op
                        .get("old_oid")
                        .unwrap_or(&default_string)
                        .as_str()
                        .unwrap()
                        .to_string();
                    let op_name = op
                        .get("op_name")
                        .unwrap_or(&default_string)
                        .as_str()
                        .unwrap()
                        .to_string();
                    let transaction_op = TransactionOp {
                        op_name: op_name,
                        oid: oid,
                        src_oid: src_oid,
                        dst_oid: dst_oid,
                        new_oid: new_oid,
                        old_oid: old_oid,
                        offset: offset,
                        length: length,
                        src_offset: src_offset,
                        dst_offset: dst_offset,
                        len: len,
                    };
                    current_transaction.transaction.push(transaction_op);
                }
            }
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
    explore(&transactions);
}

#![forbid(unsafe_code)]

fn main() {
    match ai::validation_harness::validate_root() {
        Ok(receipt) => {
            println!("{}", receipt.to_json_line());
            if !receipt.passed() {
                std::process::exit(1);
            }
        }
        Err(err) => {
            eprintln!("{err}");
            std::process::exit(1);
        }
    }
}
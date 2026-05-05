#![cfg_attr(feature = "rustc-driver", feature(rustc_private))]

#[cfg(feature = "rustc-driver")]
extern crate rustc_driver;

#[cfg(feature = "rustc-driver")]
use canon_rustc_v3::wrapper::AnalysisCallbacks;

fn exec_real_rustc(real_rustc: &str, args: &[String]) -> ! {
    let status = match std::process::Command::new(real_rustc).args(args).status() {
        Ok(status) => status,
        Err(err) => {
            eprintln!("canon-rustc-v3: failed to exec rustc at {real_rustc}: {err}");
            std::process::exit(1);
        }
    };
    std::process::exit(status.code().unwrap_or(1));
}

fn main() {
    let argv: Vec<String> = std::env::args().collect();

    if argv.len() < 2 {
        eprintln!("canon-rustc-v3: missing rustc path argument");
        std::process::exit(1);
    }

    let real_rustc = &argv[1];

    // Pass through probe/introspection invocations untouched.
    let is_probe = argv.iter().any(|a| a.starts_with("--print="))
        || argv.iter().any(|a| a == "-")
        || argv.iter().any(|a| a == "-vV" || a == "--version");

    if is_probe {
        exec_real_rustc(real_rustc, &argv[2..]);
    }

    #[cfg(not(feature = "rustc-driver"))]
    {
        eprintln!(
            "canon-rustc-v3: built without rustc-driver feature; forwarding to real rustc without witness capture"
        );
        exec_real_rustc(real_rustc, &argv[2..]);
    }

    #[cfg(feature = "rustc-driver")]
    {
        // Build args with argv[0] = wrapper binary, then argv[2..] = real compiler args.
        let args: Vec<String> = std::iter::once(argv[0].clone())
            .chain(argv.iter().skip(2).cloned())
            .collect();

        let mut callbacks = AnalysisCallbacks::new(&argv);

        let result = rustc_driver::catch_fatal_errors(|| {
            rustc_driver::run_compiler(&args, &mut callbacks);
        });

        if result.is_err() {
            std::process::exit(1);
        }
    }
}

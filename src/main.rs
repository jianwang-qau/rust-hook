// main.rs  (C) 2023 Jian Wang

use std::path::Path;
use std::process::Command;
use is_executable::IsExecutable;
use structopt::StructOpt;

macro_rules! println_err {
    () => {
        eprint!("\n")
    };
    ($fmt:expr, $($arg:expr),*) => {{
        let mut args = Vec::new();
        $(args.push(format!("{}", $arg));)*
        eprint!("\x1b[31m{}\x1b[0m\n", format!($fmt, args.join(" ")))
    }};
}

fn process_auth_hook(auth_hook_path: &str, domain: &str, validation: &str, remaining_challenges: usize, all_domains: &str) -> i32 {
    let mut err_code = 0;

    let output = Command::new(auth_hook_path)
        .env("HOOK_DOMAIN", domain)
        .env("HOOK_VALIDATION", validation)
        .env("HOOK_REMAINING_CHALLENGES", remaining_challenges.to_string())
        .env("HOOK_ALL_DOMAINS", all_domains)
        .output().unwrap();

    let stdout = String::from_utf8(output.stdout).unwrap();
    let stdout = stdout.trim_end();
    let stderr = String::from_utf8(output.stderr).unwrap();
    let stderr = stderr.trim_end();

    if stdout.len() > 0 {
        println!("Hook '--auth-hook' for {} ran with output:", domain);
        println!("{}", stdout);
    }

    if stderr.len() > 0 {
        err_code = 1;
        println_err!("Hook '--auth-hook' for {} ran with error output:", domain);
        println_err!("{}", stderr);
    }

    err_code
}

#[derive(StructOpt)]
struct Cli {
    #[structopt(long = "auth-hook")]
    auth_hook: String,
    #[structopt(short = "d", long = "domain")]
    domain: Vec<String>
}

fn main() {
    let args = Cli::from_args();
    let auth_hook = &args.auth_hook;
    let domain_vec = &args.domain;
    let all_domains = domain_vec.join(",");
    let domain_vec2: Vec<&str> = all_domains.split(",").collect();
    let domain_num = domain_vec2.len();

    let mut domain_str = format!("{}", domain_vec2[0]);
    if domain_num == 2 {
        domain_str = format!("{} and {}", domain_vec2[0], domain_vec2[1]);
    }
    if domain_num > 2 {
        domain_str = format!("{} and 2 more domains", domain_vec2[0]);
    }
    println!("Learning hook process for {}", domain_str);

    let target_path = Path::new(auth_hook);

    if !target_path.exists() {
        println_err!("Unable to find auth-hook command {} in the PATH.", auth_hook);
        println_err!("(PATH is {})", std::env::var("PATH").unwrap());
        std::process::exit(0);
    }

    if !target_path.is_executable() {
        println_err!("auth-hook command {} exists, but is not executable.", auth_hook);
        std::process::exit(0);
    }

    let auth_hook_path_buf = target_path.canonicalize().unwrap();
    let auth_hook_path = auth_hook_path_buf.to_str().unwrap();
    let mut err_total = 0;

    for (i, domain) in domain_vec2.iter().enumerate() {
        let validation = uuid::Uuid::new_v4().to_string();
        let remaining_challenges = domain_num - i - 1;
        err_total += process_auth_hook(auth_hook_path, domain, validation.as_str(), remaining_challenges, all_domains.as_str());
    }

    if err_total == 0 {
        println!("The result was successful.");
    }
}

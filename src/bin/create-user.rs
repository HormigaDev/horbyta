use std::{
    env,
    error::Error,
    io::{self, IsTerminal, Write},
    process::ExitCode,
    time::Duration,
};

use horbyta::{dtos::auth::RegisterPayload, services::users::UserService};
use sea_orm::{ConnectOptions, Database};
use validator::Validate;

const HELP: &str = "Create an active administrator in DATABASE_URL (loaded from .env).

Usage:
  cargo run --bin create-user
  cargo run --bin create-user -- EMAIL [PASSWORD] [--name NAME]
  cargo run --bin create-user -- EMAIL --password-stdin [--name NAME]

Without a password argument, the command prompts with hidden input and confirmation.
--password-stdin reads one line from stdin for automation.
The name defaults to the email prefix. Existing accounts are never overwritten.
Only DATABASE_URL is required; JWT secrets and ALLOW_REGISTRATION are not needed.";

#[derive(Default)]
struct Options {
    email: Option<String>,
    password: Option<String>,
    name: Option<String>,
    password_stdin: bool,
    help: bool,
}

fn parse_options(args: impl IntoIterator<Item = String>) -> Result<Options, String> {
    let mut options = Options::default();
    let mut args = args.into_iter();
    let mut positional_only = false;
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--" if !positional_only => positional_only = true,
            "--help" | "-h" if !positional_only => options.help = true,
            "--password-stdin" if !positional_only => options.password_stdin = true,
            "--name" if !positional_only => {
                if options.name.is_some() {
                    return Err("--name can only be specified once.".into());
                }
                options.name = Some(args.next().ok_or("--name requires a value.")?);
            }
            _ if !positional_only && arg.starts_with('-') => {
                return Err("Unknown option. Use --help; use -- before positional values starting with '-'.".into());
            }
            _ if options.email.is_none() => options.email = Some(arg),
            _ if options.password.is_none() => options.password = Some(arg),
            _ => return Err("Too many arguments. Use --help.".into()),
        }
    }
    if options.password_stdin && options.password.is_some() {
        return Err("Use either a password argument or --password-stdin, not both.".into());
    }
    if options.password_stdin && options.email.is_none() {
        return Err("--password-stdin requires an email argument.".into());
    }
    Ok(options)
}

fn read_line() -> io::Result<String> {
    let mut line = String::new();
    if io::stdin().read_line(&mut line)? == 0 {
        return Err(io::Error::new(io::ErrorKind::UnexpectedEof, "No input received."));
    }
    Ok(line.trim_end_matches(['\r', '\n']).to_string())
}

fn collect_account(options: Options) -> Result<RegisterPayload, Box<dyn Error>> {
    let email = match options.email {
        Some(value) => value,
        None => {
            print!("Email: ");
            io::stdout().flush()?;
            read_line()?
        }
    }
    .trim()
    .to_lowercase();

    let password = if let Some(value) = options.password {
        value
    } else if options.password_stdin {
        read_line()?
    } else {
        if !io::stdin().is_terminal() {
            return Err("Hidden password input requires a terminal. Use --password-stdin for automation.".into());
        }
        let value = rpassword::prompt_password("Password: ")?;
        let confirmation = rpassword::prompt_password("Confirm password: ")?;
        if value != confirmation {
            return Err("Passwords do not match.".into());
        }
        value
    };

    let account = RegisterPayload {
        name: options
            .name
            .unwrap_or_else(|| email.split('@').next().unwrap_or_default().to_string())
            .trim()
            .to_string(),
        email,
        password,
    };
    // Do not format ValidationErrors: their parameters may contain the plaintext password.
    account.validate().map_err(|_| {
        "Invalid account: name must have 2–100 characters; email must be valid; password must have at least 8 characters, uppercase, lowercase, number and symbol."
    })?;
    Ok(account)
}

fn run() -> Result<(), Box<dyn Error>> {
    let options = parse_options(env::args().skip(1))?;
    if options.help {
        println!("{HELP}");
        return Ok(());
    }
    let sources = horbyta::env::load_cli_env()?;
    let database_url = env::var("DATABASE_URL")
        .map_err(|_| horbyta::env::missing_env("DATABASE_URL", &sources))?;
    let account = collect_account(options)?;

    tokio::runtime::Runtime::new()?.block_on(async {
        let mut connection = ConnectOptions::new(database_url);
        connection
            .max_connections(1)
            .connect_timeout(Duration::from_secs(5))
            .acquire_timeout(Duration::from_secs(5))
            .sqlx_logging(false);
        let db = Database::connect(connection)
            .await
            .map_err(|_| "Could not connect to PostgreSQL. Check DATABASE_URL and database availability.")?;
        let result = UserService::provision_admin(&db, account).await;
        db.close().await?;
        let user = result?;
        println!(
            "Created administrator: {} <{}> ({}), status: {}",
            user.name, user.email, user.id, user.status
        );
        Ok(())
    })
}

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("create-user: {error}");
            ExitCode::FAILURE
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse(args: &[&str]) -> Result<Options, String> {
        parse_options(args.iter().map(|value| (*value).to_string()))
    }

    #[test]
    fn accepts_interactive_and_automated_input() {
        assert!(parse(&[]).unwrap().email.is_none());
        let options = parse(&["test@example.com", "--password-stdin", "--name", "UI Tester"]).unwrap();
        assert!(options.password_stdin);
        assert_eq!(options.name.as_deref(), Some("UI Tester"));
        let options = parse(&["--", "test@example.com", "-Secret123!"]).unwrap();
        assert_eq!(options.password.as_deref(), Some("-Secret123!"));
    }

    #[test]
    fn rejects_ambiguous_input_without_exposing_passwords() {
        for args in [
            vec!["test@example.com", "Secret123!", "--password-stdin"],
            vec!["--password-stdin"],
            vec!["--name"],
            vec!["test@example.com", "Secret123!", "extra"],
            vec!["--secret-password"],
        ] {
            let error = parse(&args).err().expect("Arguments should be rejected");
            assert!(!error.contains("Secret123!"));
            assert!(!error.contains("--secret-password"));
        }
    }
}

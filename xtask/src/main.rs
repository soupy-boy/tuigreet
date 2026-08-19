use std::{fs, path::PathBuf};

use clap::CommandFactory;
use clap_mangen::Man;

fn main() -> Result<(), Box<dyn std::error::Error>> {
  match std::env::args().nth(1).as_deref() {
    Some("man") => generate_man_page()?,
    _ => {
      eprintln!("usage: cargo xtask <COMMAND>");
      eprintln!();
      eprintln!("Commands:");
      eprintln!("  man   Generate the tuigreet(1) man page");
    },
  }
  Ok(())
}

fn generate_man_page() -> Result<(), Box<dyn std::error::Error>> {
  let app = tuigreet_config::config::Config::command();

  let about = app
    .get_about()
    .map(|styled| styled.to_string())
    .unwrap_or_default();

  let cmd = app.long_about(about);

  let mut buffer = Vec::<u8>::new();
  Man::new(cmd).render(&mut buffer)?;

  let out = PathBuf::from("contrib/man/tuigreet.1");
  fs::create_dir_all(out.parent().unwrap())?;
  fs::write(&out, &buffer)?;

  eprintln!("Generated {}", out.display());

  Ok(())
}

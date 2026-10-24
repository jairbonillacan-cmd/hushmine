//! Cliente de HUSHMINE.
//!
//! Sin argumentos juega en local: el servidor corre dentro del cliente con
//! el mismo código de red. Con `--connect` se une a un `hush-server`.
//!
//! Uso: `hushmine [--seed N] [--bots N] [--connect HOST:PUERTO] [--name NOMBRE]`.

mod game;
mod session;

use std::net::{SocketAddr, ToSocketAddrs};

use anyhow::{Context, Result, bail};

#[derive(Debug, PartialEq, Eq)]
struct Args {
    seed: u64,
    bots: u32,
    connect: Option<String>,
    name: String,
}

const USAGE: &str = "uso: hushmine [--seed N] [--bots N] [--connect HOST:PUERTO] [--name NOMBRE]";

fn main() -> Result<()> {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();
    let args = parse_args(std::env::args().skip(1))?;
    let session = match &args.connect {
        Some(target) => {
            let addr = resolve(target)?;
            log::info!("conectando a {addr} como {}", args.name);
            session::Session::connect(addr, &args.name)?
        }
        None => {
            log::info!(
                "partida local con semilla {} y {} bots",
                args.seed,
                args.bots
            );
            session::Session::local(args.seed, args.bots, &args.name)?
        }
    };
    grisu_platform::run("HUSHMINE", game::Game::new(session))
}

/// `host:puerto`, o solo `host` con el puerto por defecto.
fn resolve(target: &str) -> Result<SocketAddr> {
    let with_port = if target.contains(':') {
        target.to_owned()
    } else {
        format!("{target}:{}", hush_net::DEFAULT_PORT)
    };
    with_port
        .to_socket_addrs()
        .with_context(|| format!("dirección inválida: {target}"))?
        .next()
        .with_context(|| format!("no se encontró {target}"))
}

fn parse_args(mut args: impl Iterator<Item = String>) -> Result<Args> {
    let mut parsed = Args {
        seed: 42,
        bots: 0,
        connect: None,
        name: "minero".into(),
    };
    while let Some(arg) = args.next() {
        let mut value = || {
            args.next()
                .with_context(|| format!("falta el valor de {arg}\n{USAGE}"))
        };
        match arg.as_str() {
            "--seed" => parsed.seed = value()?.parse().context("semilla inválida")?,
            "--bots" => parsed.bots = value()?.parse().context("número de bots inválido")?,
            "--connect" => parsed.connect = Some(value()?),
            "--name" => parsed.name = value()?,
            other => bail!("argumento desconocido: {other}\n{USAGE}"),
        }
    }
    Ok(parsed)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse(list: &[&str]) -> Result<Args> {
        parse_args(list.iter().map(|s| (*s).to_owned()))
    }

    #[test]
    fn parses_arguments() {
        let a = parse(&[]).unwrap();
        assert_eq!((a.seed, a.bots, a.connect), (42, 0, None));
        let a = parse(&[
            "--seed",
            "7",
            "--bots",
            "3",
            "--connect",
            "1.2.3.4",
            "--name",
            "Ana",
        ])
        .unwrap();
        assert_eq!(a.seed, 7);
        assert_eq!(a.bots, 3);
        assert_eq!(a.connect.as_deref(), Some("1.2.3.4"));
        assert_eq!(a.name, "Ana");
        assert!(parse(&["--seed", "x"]).is_err());
        assert!(parse(&["--seed"]).is_err());
        assert!(parse(&["--fast"]).is_err());
    }

    #[test]
    fn resolves_with_default_port() {
        let addr = resolve("127.0.0.1").unwrap();
        assert_eq!(addr.port(), hush_net::DEFAULT_PORT);
        assert_eq!(resolve("127.0.0.1:9000").unwrap().port(), 9000);
    }
}

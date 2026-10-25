//! Cliente de HUSHMINE.
//!
//! Sin argumentos abre el Launcher integrado sobre una partida local con bots;
//! con argumentos (`--seed`, `--bots`, `--connect`) entra directo a la partida
//! salvo que se pase `--launcher`.
//!
//! Uso: `hushmine [--seed N] [--bots N] [--connect HOST:PUERTO] [--name NOMBRE] [--launcher]`.

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
    launcher: bool,
}

const USAGE: &str =
    "uso: hushmine [--seed N] [--bots N] [--connect HOST:PUERTO] [--name NOMBRE] [--launcher]";

fn main() -> Result<()> {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();
    let args = parse_args(std::env::args().skip(1))?;
    let remote_session = args.connect.is_some();
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
    let cfg = game::GameConfig {
        seed: args.seed,
        bots: args.bots,
        name: args.name,
        start_in_launcher: args.launcher,
        remote_session,
    };
    grisu_platform::run("HUSHMINE", game::Game::new(session, cfg))
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

fn parse_args(args: impl Iterator<Item = String>) -> Result<Args> {
    let raw: Vec<String> = args.collect();
    // Si se ejecuta sin argumentos (doble clic en el ejecutable), abrimos el
    // Launcher integrado con 4 bots por defecto.
    let no_args = raw.is_empty();
    let mut parsed = Args {
        seed: 42,
        bots: if no_args { 4 } else { 0 },
        connect: None,
        name: "minero".into(),
        launcher: no_args,
    };
    let mut iter = raw.into_iter();
    while let Some(arg) = iter.next() {
        let mut value = || {
            iter.next()
                .with_context(|| format!("falta el valor de {arg}\n{USAGE}"))
        };
        match arg.as_str() {
            "--seed" => parsed.seed = value()?.parse().context("semilla inválida")?,
            "--bots" => parsed.bots = value()?.parse().context("número de bots inválido")?,
            "--connect" => parsed.connect = Some(value()?),
            "--name" => parsed.name = value()?,
            "--launcher" => parsed.launcher = true,
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
        assert_eq!((a.seed, a.bots, a.connect, a.launcher), (42, 4, None, true));
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
        assert!(!a.launcher);
        assert!(parse(&["--launcher"]).unwrap().launcher);
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

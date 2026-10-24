//! Cliente de HUSHMINE. En el hito 1 es una partida local de un jugador
//! en la que se puede andar, cavar y poner bloques.
//!
//! Uso: `hushmine [--seed N]`.

mod game;

fn main() -> anyhow::Result<()> {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();
    let seed = parse_seed(std::env::args().skip(1))?;
    grisu_platform::run("HUSHMINE", game::Game::new(seed)?)
}

/// Lee `--seed N`; por defecto, 42.
fn parse_seed(mut args: impl Iterator<Item = String>) -> anyhow::Result<u64> {
    let mut seed = 42;
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--seed" => {
                let value = args.next().unwrap_or_default();
                seed = value
                    .parse()
                    .map_err(|_| anyhow::anyhow!("semilla inválida: {value:?}"))?;
            }
            other => anyhow::bail!("argumento desconocido: {other} (uso: hushmine [--seed N])"),
        }
    }
    Ok(seed)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn args(list: &[&str]) -> impl Iterator<Item = String> {
        list.iter()
            .map(|s| (*s).to_owned())
            .collect::<Vec<_>>()
            .into_iter()
    }

    #[test]
    fn parses_seed() {
        assert_eq!(parse_seed(args(&[])).unwrap(), 42);
        assert_eq!(parse_seed(args(&["--seed", "7"])).unwrap(), 7);
        assert!(parse_seed(args(&["--seed", "x"])).is_err());
        assert!(parse_seed(args(&["--fast"])).is_err());
    }
}

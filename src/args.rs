pub fn seed_from_args(arguments: impl IntoIterator<Item = String>) -> Result<Option<u64>, String> {
    let mut values = arguments.into_iter();
    let _program = values.next();
    match values.next() {
        None => Ok(None),
        Some(flag) if flag == "--seed" => {
            let value = values.next().ok_or("--seed requires an unsigned integer")?;
            if values.next().is_some() {
                return Err("unexpected extra command-line arguments".into());
            }
            value
                .parse()
                .map(Some)
                .map_err(|_| "--seed must be an unsigned integer".into())
        }
        Some(flag) => Err(format!("unknown argument: {flag}")),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn parses_optional_seed() {
        assert_eq!(
            seed_from_args(vec!["game".into(), "--seed".into(), "42".into()]),
            Ok(Some(42))
        );
        assert_eq!(seed_from_args(vec!["game".into()]), Ok(None));
    }
}

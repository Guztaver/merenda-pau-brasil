use chrono::NaiveDate;
pub type R<T> = Result<T, String>;
pub fn sql(e: rusqlite::Error) -> String {
    e.to_string()
}
pub fn date(s: &str) -> R<()> {
    let d = NaiveDate::parse_from_str(s, "%Y-%m-%d")
        .map_err(|_| "Use uma data válida: AAAA-MM-DD.".to_string())?;
    if s.len() != 10 || d.format("%Y-%m-%d").to_string() != s {
        return Err("Use uma data válida com dez caracteres: AAAA-MM-DD.".into());
    }
    Ok(())
}
pub fn grams(s: &str) -> R<i64> {
    let n = s.trim().replace(',', ".");
    let n = n
        .parse::<f64>()
        .map_err(|_| "Informe um peso válido.".to_string())?;
    if !n.is_finite() || n <= 0.0 || n > 1e9 {
        return Err("O peso deve ser positivo e menor que 1.000.000.000 g.".into());
    }
    let g = n.round() as i64;
    if g < 1 {
        return Err("O mínimo é 1 grama.".into());
    }
    Ok(g)
}
pub fn kg(g: i64) -> String {
    format!("{:.3}", g as f64 / 1000.).replace('.', ",")
}
pub fn currency(cents: i64) -> String {
    format!("{:.2}", cents as f64 / 100.).replace('.', ",")
}
pub fn money(input: &str) -> R<i64> {
    let s = input.trim();
    if s.is_empty() || s.contains(['-', '+', 'e', 'E']) {
        return Err("Informe um valor monetário positivo, por exemplo 1234,56.".into());
    }
    let normalized = if s.contains(',') {
        s.replace('.', "").replace(',', ".")
    } else {
        s.to_string()
    };
    let parts: Vec<_> = normalized.split('.').collect();
    if parts.len() > 2 || !parts[0].bytes().all(|b| b.is_ascii_digit()) || parts[0].is_empty() {
        return Err("Valor monetário inválido.".into());
    }
    let decimals = parts.get(1).copied().unwrap_or("");
    if decimals.len() > 2 || !decimals.bytes().all(|b| b.is_ascii_digit()) {
        return Err("Use no máximo duas casas decimais no valor.".into());
    }
    let whole = parts[0]
        .parse::<i64>()
        .map_err(|_| "Valor muito alto.".to_string())?;
    let fraction = format!("{decimals:0<2}")
        .parse::<i64>()
        .map_err(|_| "Valor inválido.".to_string())?;
    let cents = whole
        .checked_mul(100)
        .and_then(|n| n.checked_add(fraction))
        .ok_or("Valor muito alto.")?;
    if cents <= 0 || cents > 1_000_000_000_000 {
        return Err("Informe um valor maior que zero e até R$ 10 bilhões.".into());
    }
    Ok(cents)
}

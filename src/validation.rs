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

pub trait MLProcessable {
    /// Identificatore univoco dell’entità
    fn id(&self) -> i64;
    /// Rappresentazione canonica (normalizzata) dell’entità
    fn canonical_key(&self) -> String;
    /// Tutte le varianti note (alias, typo, ecc.)
    fn variants(&self) -> Vec<String>;
    /// Aggiorna le varianti (ad es. dopo clustering ML)
    fn set_variants(&mut self, variants: Vec<String>);
}

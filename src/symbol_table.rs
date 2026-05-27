pub struct SymbolTableEntry {
    typ: String,
}

impl SymbolTableEntry {
    pub fn getType(&self) -> &String {
        &self.typ
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Field {
    Title,
    Description,
    Type,
    Doi,
    Isbn,
    PublicationDate,
    Author,
    Tag,
    Collection,
}

impl Field {
    /// Fields backed by a joined, aggregated relation rather
    /// than a plain column on `items`.
    /// These are the only fields where a list value (`field=[a,b]`) is meaningful for
    pub fn is_multi_valued(&self) -> bool {
        matches!(self, Field::Author | Field::Collection | Field::Tag)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Op {
    /// Exact match on a scalar field; "has all of" on a multi-value field.
    Eq,
    /// Substring match on a scalar field; "had any of" on a multi-value field.
    Fuzzy,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Value {
    Single(String),
    List(Vec<String>),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Expr {
    Compare { field: Field, op: Op, value: Value },
    And(Box<Expr>, Box<Expr>),
    Or(Box<Expr>, Box<Expr>),
    Not(Box<Expr>),
}

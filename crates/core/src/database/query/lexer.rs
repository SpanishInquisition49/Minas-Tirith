use logos::Logos;

#[derive(Logos, PartialEq, Eq, Debug)]
#[logos(skip r"[ \t\r\n\f]+")]
pub enum QueryToken {
    #[token("(")]
    LParen,
    #[token(")")]
    RParen,
    #[token("[")]
    LSquare,
    #[token("]")]
    RSquare,
    #[token(",")]
    Comma,
    #[token("!")]
    Not,
    #[token("&&")]
    And,
    #[token("||")]
    Or,
    #[token("=")]
    Equal,
    #[token("~")]
    FuzzyEqual,
    // NOTE: fields
    #[token("title")]
    FieldTitle,
    #[token("description")]
    FieldDesc,
    #[token("type")]
    FieldType,
    #[token("doi")]
    FieldDoi,
    #[token("isbn")]
    FieldIsbn,
    #[token("published")]
    FieldPublicationDate,
    #[token("author")]
    FieldAuthor,
    #[token("tag")]
    FieldTag,
    #[token("collection")]
    FieldCollection,
    // NOTE: values
    #[regex(r#""([^"\\]|\\.)*""#, |lex| unescape(lex.slice()))]
    Str(String),
    #[regex(r"[^\s()\[\],~=!&|]+", |lex| lex.slice().to_string())]
    Ident(String),
}

fn unescape(raw: &str) -> Result<String, ()> {
    serde_json::from_str(raw).map_err(|_| ())
}

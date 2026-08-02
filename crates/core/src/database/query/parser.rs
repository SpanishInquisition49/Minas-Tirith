use std::{iter::Peekable, vec::IntoIter};

use color_eyre::eyre::{Result, bail, eyre};
use logos::Logos;

use crate::database::query::{
    ast::{Expr, Field, Op, Value},
    lexer::QueryToken,
};

/// Try to parse a raw filter clause.
/// # Errors
/// if the given filter is not valid an error is returned
/// explaining why the parsing failed.
pub fn parse(input: &str) -> Result<Expr> {
    let tokens = tokenize(input)?;
    let mut parser = Parser {
        tokens: tokens.into_iter().peekable(),
    };
    let expr = parser.parse_or()?;
    if let Some(tok) = parser.tokens.next() {
        bail!("unexpected trailing input near {tok:?}");
    }
    Ok(expr)
}

fn tokenize(input: &str) -> Result<Vec<QueryToken>> {
    let mut lexer = QueryToken::lexer(input);
    let mut tokens = Vec::new();
    while let Some(result) = lexer.next() {
        match result {
            Ok(tok) => tokens.push(tok),
            Err(()) => {
                bail!(
                    "unexpected input {:?} at position {:?}",
                    lexer.slice(),
                    lexer.span().start
                )
            }
        }
    }
    Ok(tokens)
}

struct Parser {
    tokens: Peekable<IntoIter<QueryToken>>,
}

impl Parser {
    fn parse_or(&mut self) -> Result<Expr> {
        let mut lhs = self.parse_and()?;
        while matches!(self.tokens.peek(), Some(QueryToken::Or)) {
            self.tokens.next();
            let rhs = self.parse_and()?;
            lhs = Expr::Or(Box::new(lhs), Box::new(rhs));
        }
        Ok(lhs)
    }

    fn parse_and(&mut self) -> Result<Expr> {
        let mut lhs = self.parse_unary()?;
        while matches!(self.tokens.peek(), Some(QueryToken::And)) {
            self.tokens.next();
            let rhs = self.parse_unary()?;
            lhs = Expr::And(Box::new(lhs), Box::new(rhs));
        }
        Ok(lhs)
    }

    fn parse_unary(&mut self) -> Result<Expr> {
        if matches!(self.tokens.peek(), Some(QueryToken::Not)) {
            self.tokens.next();
            return Ok(Expr::Not(Box::new(self.parse_unary()?)));
        }
        self.parse_atom()
    }

    fn parse_atom(&mut self) -> Result<Expr> {
        match self.tokens.next() {
            Some(QueryToken::LParen) => {
                let expr = self.parse_or()?;
                self.expect(&QueryToken::RParen)?;
                Ok(expr)
            }
            Some(tok) => self.parse_compare(&tok),
            None => bail!("unexpected end of query"),
        }
    }

    fn parse_compare(&mut self, field_tok: &QueryToken) -> Result<Expr> {
        let field = Self::token_to_field(field_tok)
            .ok_or_else(|| eyre!("expected a field name, found {field_tok:?}"))?;

        let op = match self.tokens.next() {
            Some(QueryToken::Equal) => Op::Eq,
            Some(QueryToken::FuzzyEqual) => Op::Fuzzy,
            other => bail!("expected '=' or '~' after field, found {other:?}"),
        };
        let value = self.parse_value()?;
        if !field.is_multi_valued() && matches!(value, Value::List(_)) {
            bail!("field '{field:?}' does not support list values");
        }
        Ok(Expr::Compare { field, op, value })
    }

    fn parse_value(&mut self) -> Result<Value> {
        match self.tokens.next() {
            Some(QueryToken::Str(s) | QueryToken::Ident(s)) => Ok(Value::Single(s)),
            Some(QueryToken::LSquare) => Ok(Value::List(self.parse_value_list()?)),
            other => bail!("expected a value, found {other:?}"),
        }
    }

    fn parse_value_list(&mut self) -> Result<Vec<String>> {
        let mut items = Vec::new();
        loop {
            match self.tokens.next() {
                Some(QueryToken::Str(s) | QueryToken::Ident(s)) => items.push(s),
                other => bail!("expected a value list, found {other:?}"),
            }
            match self.tokens.next() {
                Some(QueryToken::Comma) => {}
                Some(QueryToken::RSquare) => break,
                other => bail!("expected ',' or ']' in list, found {other:?}"),
            }
        }
        Ok(items)
    }

    fn expect(&mut self, expected: &QueryToken) -> Result<()> {
        match self.tokens.next() {
            Some(tok) if tok == *expected => Ok(()),
            other => bail!("expected {expected:?}, found {other:?}"),
        }
    }

    fn token_to_field(tok: &QueryToken) -> Option<Field> {
        Some(match tok {
            QueryToken::FieldTitle => Field::Title,
            QueryToken::FieldDesc => Field::Description,
            QueryToken::FieldType => Field::Type,
            QueryToken::FieldDoi => Field::Doi,
            QueryToken::FieldIsbn => Field::Isbn,
            QueryToken::FieldPublicationDate => Field::PublicationDate,
            QueryToken::FieldAuthor => Field::Author,
            QueryToken::FieldTag => Field::Tag,
            QueryToken::FieldCollection => Field::Collection,
            _ => return None,
        })
    }
}

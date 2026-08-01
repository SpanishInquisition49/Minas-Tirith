use sqlx::{QueryBuilder, Sqlite};

use crate::database::query::ast::{Expr, Field, Op, Value};

/// Compile `expr` and push it onto `builder` as a parenthesized boolen
/// expression, e.g. `(i.title LIKE '%' || ? || '%')`. Caller is responsible
/// for pushing the `WHERE` keyword first.
pub fn push_expr(builder: &mut QueryBuilder<Sqlite>, expr: &Expr) {
    match expr {
        Expr::Compare { field, op, value } => push_compare(builder, *field, *op, value),
        Expr::And(lhs, rhs) => push_binary(builder, lhs, " AND ", rhs),
        Expr::Or(lhs, rhs) => push_binary(builder, lhs, " OR ", rhs),
        Expr::Not(inner) => {
            builder.push("NOT (");
            push_expr(builder, inner);
            builder.push(")");
        }
    }
}

fn push_binary(builder: &mut QueryBuilder<Sqlite>, lhs: &Expr, joiner: &str, rhs: &Expr) {
    builder.push("(");
    push_expr(builder, lhs);
    builder.push(joiner);
    push_expr(builder, rhs);
    builder.push(")");
}

fn push_compare(builder: &mut QueryBuilder<Sqlite>, field: Field, op: Op, value: &Value) {
    if field.is_multi_valued() {
        push_multi_valued(builder, field, op, value);
    } else {
        push_scalar(builder, field, op, value);
    }
}

fn scalar_column(field: Field) -> &'static str {
    match field {
        Field::Title => "i.title",
        Field::Description => "i.description",
        Field::Type => "i.type",
        Field::Doi => "i.doi",
        Field::Isbn => "i.isbn",
        Field::PublicationDate => "i.publication_date",
        Field::Author | Field::Tag | Field::Collection => {
            unreachable!("{field:?} is multi-valued, not a scalar column")
        }
    }
}

fn push_scalar(builder: &mut QueryBuilder<Sqlite>, field: Field, op: Op, value: &Value) {
    // NOTE: The parser rejects list values on scalar fields, so this always holds.
    let Value::Single(text) = value else {
        unreachable!("list value on scalar field {field:?} should have been rejected at parse time")
    };
    builder.push(scalar_column(field));
    match op {
        Op::Eq => {
            builder.push(" = ");
            builder.push_bind(text.clone());
        }
        Op::Fuzzy => {
            builder.push(" LIKE '%' || ");
            builder.push_bind(text.clone());
            builder.push(" || '%'");
        }
    }
}

/// Aggregated JSON column (from a `LEFT JOIN view_*_aggregated`)
/// for a multi-valued field, e.g. `c.collections`.
/// Every element in that JSON array has a `.name` key, which is what `push_multi_valued` matches on
fn aggregated_column(field: Field) -> &'static str {
    match field {
        Field::Author => "a.authors",
        Field::Tag => "t.tags",
        Field::Collection => "c.collections",
        Field::Title
        | Field::Description
        | Field::Type
        | Field::Doi
        | Field::Isbn
        | Field::PublicationDate => unreachable!("{field:?} is scalar, not multi-valued"),
    }
}

fn push_multi_valued(builder: &mut QueryBuilder<Sqlite>, field: Field, op: Op, value: &Value) {
    let values = match value {
        // NOTE: syntactic sugar for single values, we auto wrap them in a list
        Value::Single(v) => vec![v],
        Value::List(vs) => vs.iter().collect(),
    };

    let column = aggregated_column(field);
    // NOTE: both operators compare each value by exact match on `.name`;
    // only the quantifier joining them differs: `=` requires ALL of the listed values
    // to be present (AND), `~` requires ANY of them (OR)
    let joiner = match op {
        Op::Eq => " AND ",
        Op::Fuzzy => " OR ",
    };
    builder.push("(");
    for (i, v) in values.iter().enumerate() {
        if i > 0 {
            builder.push(joiner);
        }
        push_exist_eq(builder, column, v);
    }
    builder.push(")");
}

fn push_exist_eq(builder: &mut QueryBuilder<Sqlite>, column: &str, value: &str) {
    builder.push("EXISTS(SELECT 1 FROM json_each(");
    builder.push(column);
    builder.push(") AS je WHERE json_extract(je.value, '$.name') = ");
    builder.push_bind(value.to_string());
    builder.push(")");
}

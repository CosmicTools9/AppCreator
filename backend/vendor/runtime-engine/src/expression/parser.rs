//! 约束表达式解析器
//!
//! 将字符串表达式解析为 AST (ConstraintExpr)。
//! 支持算术运算 (+, -, *, /, %)、比较运算、逻辑运算 (AND/OR/NOT) 和函数调用。

use runtime_contract::expression::{
    BinaryOp, ConstraintExpr, ConstraintLiteral, ConstraintParseError, UnaryOp,
};

/// 解析约束表达式字符串为 AST
pub fn parse_constraint_expression(input: &str) -> Result<ConstraintExpr, ConstraintParseError> {
    let input = input.trim();
    if input.is_empty() {
        return Err(ConstraintParseError::EmptyExpression);
    }
    parse_or_expr(input)
}

/// 解析 OR 表达式（最低优先级）
fn parse_or_expr(input: &str) -> Result<ConstraintExpr, ConstraintParseError> {
    if let Some(pos) = find_logical_op(input, "OR") {
        let left = &input[..pos].trim();
        let right = &input[pos + 2..].trim();
        return Ok(ConstraintExpr::Or(
            Box::new(parse_or_expr(left)?),
            Box::new(parse_or_expr(right)?),
        ));
    }
    parse_and_expr(input)
}

/// 解析 AND 表达式
fn parse_and_expr(input: &str) -> Result<ConstraintExpr, ConstraintParseError> {
    if let Some(pos) = find_logical_op(input, "AND") {
        let left = &input[..pos].trim();
        let right = &input[pos + 3..].trim();
        return Ok(ConstraintExpr::And(
            Box::new(parse_and_expr(left)?),
            Box::new(parse_and_expr(right)?),
        ));
    }
    parse_comparison(input)
}

/// 解析比较表达式
fn parse_comparison(input: &str) -> Result<ConstraintExpr, ConstraintParseError> {
    let input = input.trim();

    // 括号包裹
    if input.starts_with('(') && input.ends_with(')') {
        let inner = &input[1..input.len() - 1].trim();
        return parse_or_expr(inner);
    }

    // NOT 前缀
    if input.to_uppercase().starts_with("NOT ") {
        let inner = &input[4..].trim();
        return Ok(ConstraintExpr::Not(Box::new(parse_comparison(inner)?)));
    }

    // 比较运算符（从左到右扫描，跳过函数调用内部）
    let ops = [
        ("<=", BinaryOp::Le),
        (">=", BinaryOp::Ge),
        ("!=", BinaryOp::Ne),
        ("==", BinaryOp::Eq),
        ("=", BinaryOp::Eq),
        ("<", BinaryOp::Lt),
        (">", BinaryOp::Gt),
    ];

    let mut depth = 0i32;
    for (i, c) in input.char_indices() {
        match c {
            '(' => depth += 1,
            ')' => depth -= 1,
            _ if depth > 0 => continue,
            _ => {}
        }

        for (op_str, op) in &ops {
            if input[i..].starts_with(op_str) {
                let after_pos = i + op_str.len();
                if after_pos < input.len() {
                    let after_char = input.chars().nth(after_pos).unwrap();
                    if after_char.is_alphanumeric() || after_char == '_' {
                        continue;
                    }
                }

                let left = &input[..i].trim();
                let right = &input[after_pos..].trim();

                return Ok(ConstraintExpr::Binary(
                    Box::new(parse_additive_expr(left)?),
                    *op,
                    Box::new(parse_additive_expr(right)?),
                ));
            }
        }
    }

    // 无比较运算符，按算术表达式解析
    parse_additive_expr(input)
}

/// 解析加减表达式
fn parse_additive_expr(input: &str) -> Result<ConstraintExpr, ConstraintParseError> {
    let input = input.trim();

    // 一元负号
    if let Some(rest) = input.strip_prefix('-') {
        let inner = rest.trim();
        if !inner.is_empty() {
            return Ok(ConstraintExpr::Unary(
                UnaryOp::Neg,
                Box::new(parse_additive_expr(inner)?),
            ));
        }
    }

    let mut depth = 0i32;
    for (i, c) in input.char_indices().rev() {
        match c {
            ')' => depth += 1,
            '(' => depth -= 1,
            _ if depth > 0 => continue,
            '+' => {
                let left = &input[..i].trim();
                let right = &input[i + 1..].trim();
                if !left.is_empty() && !right.is_empty() {
                    return Ok(ConstraintExpr::Binary(
                        Box::new(parse_additive_expr(left)?),
                        BinaryOp::Add,
                        Box::new(parse_multiplicative_expr(right)?),
                    ));
                }
            }
            '-' => {
                let left = &input[..i].trim();
                let right = &input[i + 1..].trim();
                if !left.is_empty() && !right.is_empty() {
                    return Ok(ConstraintExpr::Binary(
                        Box::new(parse_additive_expr(left)?),
                        BinaryOp::Sub,
                        Box::new(parse_multiplicative_expr(right)?),
                    ));
                }
            }
            _ => {}
        }
    }

    parse_multiplicative_expr(input)
}

/// 解析乘除模表达式
fn parse_multiplicative_expr(input: &str) -> Result<ConstraintExpr, ConstraintParseError> {
    let input = input.trim();

    let mut depth = 0i32;
    for (i, c) in input.char_indices().rev() {
        match c {
            ')' => depth += 1,
            '(' => depth -= 1,
            _ if depth > 0 => continue,
            '*' => {
                let left = &input[..i].trim();
                let right = &input[i + 1..].trim();
                if !left.is_empty() && !right.is_empty() {
                    return Ok(ConstraintExpr::Binary(
                        Box::new(parse_multiplicative_expr(left)?),
                        BinaryOp::Mul,
                        Box::new(parse_term_expr(right)?),
                    ));
                }
            }
            '/' => {
                let left = &input[..i].trim();
                let right = &input[i + 1..].trim();
                if !left.is_empty() && !right.is_empty() {
                    return Ok(ConstraintExpr::Binary(
                        Box::new(parse_multiplicative_expr(left)?),
                        BinaryOp::Div,
                        Box::new(parse_term_expr(right)?),
                    ));
                }
            }
            '%' => {
                let left = &input[..i].trim();
                let right = &input[i + 1..].trim();
                if !left.is_empty() && !right.is_empty() {
                    return Ok(ConstraintExpr::Binary(
                        Box::new(parse_multiplicative_expr(left)?),
                        BinaryOp::Mod,
                        Box::new(parse_term_expr(right)?),
                    ));
                }
            }
            _ => {}
        }
    }

    parse_term_expr(input)
}

/// 解析原子项（字面量、字段引用、函数调用、括号表达式）
fn parse_term_expr(input: &str) -> Result<ConstraintExpr, ConstraintParseError> {
    let input = input.trim();

    if input.is_empty() {
        return Err(ConstraintParseError::EmptyExpression);
    }

    // 字符串字面量
    if (input.starts_with('"') && input.ends_with('"'))
        || (input.starts_with('\'') && input.ends_with('\''))
    {
        let value = &input[1..input.len() - 1];
        return Ok(ConstraintExpr::Literal(ConstraintLiteral::String(
            value.to_string(),
        )));
    }

    // 数值字面量
    if let Ok(num) = input.parse::<i64>() {
        return Ok(ConstraintExpr::Literal(ConstraintLiteral::Integer(num)));
    }
    if let Ok(num) = input.parse::<f64>() {
        return Ok(ConstraintExpr::Literal(ConstraintLiteral::Decimal(num)));
    }

    // 布尔/Null 字面量
    if input.eq_ignore_ascii_case("true") {
        return Ok(ConstraintExpr::Literal(ConstraintLiteral::Boolean(true)));
    }
    if input.eq_ignore_ascii_case("false") {
        return Ok(ConstraintExpr::Literal(ConstraintLiteral::Boolean(false)));
    }
    if input.eq_ignore_ascii_case("null") {
        return Ok(ConstraintExpr::Literal(ConstraintLiteral::Null));
    }

    // 括号表达式
    if input.starts_with('(') && input.ends_with(')') {
        let inner = &input[1..input.len() - 1].trim();
        return parse_or_expr(inner);
    }

    // 函数调用: name(arg1, arg2)
    if let Some(paren_pos) = input.find('(') {
        if input.ends_with(')') {
            let name = &input[..paren_pos].trim();
            let args_str = &input[paren_pos + 1..input.len() - 1].trim();
            let args = parse_arguments(args_str)?;
            return Ok(ConstraintExpr::Call(name.to_string(), args));
        }
    }

    // 字段引用
    if is_valid_identifier(input) {
        return Ok(ConstraintExpr::FieldRef(input.to_string()));
    }

    Err(ConstraintParseError::InvalidSyntax(format!(
        "Cannot parse: {}",
        input
    )))
}

/// 解析函数参数列表
fn parse_arguments(input: &str) -> Result<Vec<ConstraintExpr>, ConstraintParseError> {
    if input.is_empty() {
        return Ok(Vec::new());
    }

    let mut args = Vec::new();
    let mut depth = 0;
    let mut start = 0;

    for (i, c) in input.char_indices() {
        match c {
            '(' => depth += 1,
            ')' => depth -= 1,
            ',' if depth == 0 => {
                let arg = &input[start..i].trim();
                args.push(parse_or_expr(arg)?);
                start = i + 1;
            }
            _ => {}
        }
    }

    let arg = &input[start..].trim();
    args.push(parse_or_expr(arg)?);
    Ok(args)
}

/// 在括号外部查找逻辑运算符位置
fn find_logical_op(input: &str, op: &str) -> Option<usize> {
    let upper = input.to_uppercase();
    let mut depth = 0;

    for (i, c) in input.char_indices() {
        match c {
            '(' => depth += 1,
            ')' => depth -= 1,
            _ if depth == 0 && upper[i..].starts_with(op) => {
                let after = i + op.len();
                if after >= input.len() || !input.chars().nth(after).unwrap().is_alphanumeric() {
                    return Some(i);
                }
            }
            _ => {}
        }
    }

    None
}

/// 检查是否为合法标识符
fn is_valid_identifier(s: &str) -> bool {
    if s.is_empty() {
        return false;
    }
    let first = s.chars().next().unwrap();
    if !first.is_alphabetic() && first != '_' {
        return false;
    }
    s.chars().all(|c| c.is_alphanumeric() || c == '_')
}

/// 从表达式字符串中提取字段引用
pub fn extract_field_references(expression: &str) -> Vec<String> {
    let mut fields = Vec::new();
    let tokens: Vec<&str> = expression.split_whitespace().collect();

    for token in tokens {
        let clean = token.trim_matches(|c: char| !c.is_alphanumeric() && c != '_');
        if is_valid_identifier(clean) && !is_keyword(clean) {
            fields.push(clean.to_string());
        }
    }

    fields.sort();
    fields.dedup();
    fields
}

fn is_keyword(s: &str) -> bool {
    let keywords = ["AND", "OR", "NOT", "true", "false", "null"];
    keywords.contains(&s.to_uppercase().as_str())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_comparison() {
        let expr = parse_constraint_expression("price > 0").unwrap();
        match expr {
            ConstraintExpr::Binary(left, BinaryOp::Gt, right) => match (&*left, &*right) {
                (
                    ConstraintExpr::FieldRef(name),
                    ConstraintExpr::Literal(ConstraintLiteral::Integer(0)),
                ) => {
                    assert_eq!(name, "price");
                }
                _ => panic!("Unexpected structure"),
            },
            _ => panic!("Expected Binary"),
        }
    }

    #[test]
    fn test_parse_and_or() {
        let expr = parse_constraint_expression("price > 0 AND price < 100").unwrap();
        assert!(matches!(expr, ConstraintExpr::And(_, _)));

        let expr = parse_constraint_expression("status = \"A\" OR status = \"B\"").unwrap();
        assert!(matches!(expr, ConstraintExpr::Or(_, _)));
    }

    #[test]
    fn test_parse_not() {
        let expr = parse_constraint_expression("NOT deleted").unwrap();
        assert!(matches!(expr, ConstraintExpr::Not(_)));
    }

    #[test]
    fn test_parse_arithmetic() {
        assert!(matches!(
            parse_constraint_expression("a + b").unwrap(),
            ConstraintExpr::Binary(_, BinaryOp::Add, _)
        ));
        assert!(matches!(
            parse_constraint_expression("a - b").unwrap(),
            ConstraintExpr::Binary(_, BinaryOp::Sub, _)
        ));
        assert!(matches!(
            parse_constraint_expression("a * b").unwrap(),
            ConstraintExpr::Binary(_, BinaryOp::Mul, _)
        ));
        assert!(matches!(
            parse_constraint_expression("a / b").unwrap(),
            ConstraintExpr::Binary(_, BinaryOp::Div, _)
        ));
        assert!(matches!(
            parse_constraint_expression("a % b").unwrap(),
            ConstraintExpr::Binary(_, BinaryOp::Mod, _)
        ));
    }

    #[test]
    fn test_parse_complex_formula() {
        let expr =
            parse_constraint_expression("quantity * unit_price * (1 - discount_rate)").unwrap();
        assert!(matches!(expr, ConstraintExpr::Binary(_, BinaryOp::Mul, _)));
    }

    #[test]
    fn test_extract_fields() {
        let fields = extract_field_references("price > 0 AND quantity < stock");
        assert!(fields.contains(&"price".to_string()));
        assert!(fields.contains(&"quantity".to_string()));
        assert!(fields.contains(&"stock".to_string()));
    }

    #[test]
    fn test_parse_string_literal() {
        let expr = parse_constraint_expression(r#"status = "active""#).unwrap();
        match expr {
            ConstraintExpr::Binary(_, BinaryOp::Eq, right) => match &*right {
                ConstraintExpr::Literal(ConstraintLiteral::String(s)) => assert_eq!(s, "active"),
                _ => panic!("Expected String literal"),
            },
            _ => panic!("Expected Binary"),
        }
    }

    #[test]
    fn test_parse_decimal() {
        let expr = parse_constraint_expression("price < 99.99").unwrap();
        match expr {
            ConstraintExpr::Binary(_, BinaryOp::Lt, right) => match &*right {
                ConstraintExpr::Literal(ConstraintLiteral::Decimal(v)) => {
                    assert!((*v - 99.99).abs() < 0.001);
                }
                _ => panic!("Expected Decimal literal"),
            },
            _ => panic!("Expected Binary"),
        }
    }
}

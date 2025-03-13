use crate::{expr::Expr, token::Literal};

pub trait AstPrinter {
    fn print(&self) -> String;
}

impl AstPrinter for Expr {
    fn print(&self) -> String {
        match self {
            Expr::Binary {
                left,
                operator,
                right,
            } => parenthesize(&operator.lexeme, vec![left, right]),
            Expr::Grouping { expression } => parenthesize("group", vec![expression]),
            Expr::Literal { value } => match value {
                Literal::String(s) => s.clone(),
                Literal::Number(n) => n.to_string(),
                Literal::Nil => "nil".to_string(),
                Literal::Boolean(b) => b.to_string(),
            },
            Expr::Unary { operator, right } => parenthesize(&operator.lexeme, vec![right]),
        }
    }
}

fn parenthesize(name: &str, exprs: Vec<&Expr>) -> String {
    let mut result = String::from("(");
    result.push_str(name);

    for expr in exprs {
        result.push(' ');
        result.push_str(&expr.print());
    }

    result.push(')');

    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::token::Token;
    use crate::token_type::TokenType;

    #[test]
    fn test_ast_printer() {
        let expr = Expr::Binary {
            left: Box::new(Expr::Unary {
                operator: Token::new(TokenType::Minus, "-".to_string(), 1),
                right: Box::new(Expr::Literal {
                    value: Literal::Number(123_f64),
                }),
            }),
            operator: Token::new(TokenType::Star, "*".to_string(), 1),
            right: Box::new(Expr::Grouping {
                expression: Box::new(Expr::Literal {
                    value: Literal::Number(45.67),
                }),
            }),
        };

        assert_eq!(expr.print(), "(* (- 123) (group 45.67))");
    }
}

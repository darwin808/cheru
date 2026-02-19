struct Parser {
    chars: Vec<char>,
    pos: usize,
}

impl Parser {
    fn new(input: &str) -> Self {
        Self {
            chars: input.chars().filter(|c| !c.is_whitespace()).collect(),
            pos: 0,
        }
    }

    fn peek(&self) -> Option<char> {
        self.chars.get(self.pos).copied()
    }

    fn next(&mut self) -> Option<char> {
        let c = self.chars.get(self.pos).copied();
        self.pos += 1;
        c
    }

    fn parse_expr(&mut self) -> Option<f64> {
        let mut left = self.parse_term()?;
        while let Some(op) = self.peek() {
            if op == '+' || op == '-' {
                self.next();
                let right = self.parse_term()?;
                left = if op == '+' { left + right } else { left - right };
            } else {
                break;
            }
        }
        Some(left)
    }

    fn parse_term(&mut self) -> Option<f64> {
        let mut left = self.parse_power()?;
        while let Some(op) = self.peek() {
            if op == '*' || op == '/' {
                self.next();
                let right = self.parse_power()?;
                left = if op == '*' { left * right } else { left / right };
            } else {
                break;
            }
        }
        Some(left)
    }

    fn parse_power(&mut self) -> Option<f64> {
        let base = self.parse_unary()?;
        if self.peek() == Some('^') {
            self.next();
            let exp = self.parse_power()?;
            Some(base.powf(exp))
        } else {
            Some(base)
        }
    }

    fn parse_unary(&mut self) -> Option<f64> {
        if self.peek() == Some('-') {
            self.next();
            Some(-self.parse_unary()?)
        } else {
            self.parse_atom()
        }
    }

    fn parse_atom(&mut self) -> Option<f64> {
        if self.peek() == Some('(') {
            self.next();
            let val = self.parse_expr()?;
            if self.next() != Some(')') {
                return None;
            }
            Some(val)
        } else {
            self.parse_number()
        }
    }

    fn parse_number(&mut self) -> Option<f64> {
        let start = self.pos;
        while let Some(c) = self.peek() {
            if c.is_ascii_digit() || c == '.' {
                self.next();
            } else {
                break;
            }
        }
        if self.pos == start {
            return None;
        }
        let s: String = self.chars[start..self.pos].iter().collect();
        s.parse().ok()
    }
}

pub fn evaluate(input: &str) -> Option<String> {
    if input.is_empty() {
        return None;
    }
    // Quick check: must contain at least one digit and one operator or parens
    let has_digit = input.chars().any(|c| c.is_ascii_digit());
    let has_op = input.chars().any(|c| matches!(c, '+' | '-' | '*' | '/' | '^' | '(' | ')'));
    if !has_digit || !has_op {
        return None;
    }
    let mut parser = Parser::new(input);
    let result = parser.parse_expr()?;
    if parser.pos < parser.chars.len() {
        return None;
    }
    format_number(result)
}

fn format_number(v: f64) -> Option<String> {
    if v.is_nan() || v.is_infinite() {
        return None;
    }
    if v == v.trunc() && v.abs() < 1e15 {
        Some(format!("{}", v as i64))
    } else {
        let s = format!("{:.10}", v);
        let s = s.trim_end_matches('0').trim_end_matches('.');
        Some(s.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn basic_arithmetic() {
        assert_eq!(evaluate("2+3"), Some("5".into()));
        assert_eq!(evaluate("10-4"), Some("6".into()));
        assert_eq!(evaluate("3*4"), Some("12".into()));
        assert_eq!(evaluate("15/4"), Some("3.75".into()));
    }

    #[test]
    fn operator_precedence() {
        assert_eq!(evaluate("2+3*4"), Some("14".into()));
        assert_eq!(evaluate("(2+3)*4"), Some("20".into()));
    }

    #[test]
    fn power() {
        assert_eq!(evaluate("2^10"), Some("1024".into()));
    }

    #[test]
    fn unary_minus() {
        assert_eq!(evaluate("-5+3"), Some("-2".into()));
    }

    #[test]
    fn not_math() {
        assert_eq!(evaluate("hello"), None);
        assert_eq!(evaluate("firefox"), None);
        assert_eq!(evaluate(""), None);
    }

    #[test]
    fn division_by_zero() {
        assert_eq!(evaluate("1/0"), None);
    }
}

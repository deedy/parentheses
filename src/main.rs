use std::str::FromStr;

#[derive(Debug, Clone, PartialEq)]
enum Token {
    Number(i64),
    Plus,
    Mul,
}

fn main() {
    // The given expression:
    let expression = "1 + 2 * 3 + 4 * 5 + 6 * 7 + 8 * 9";
    let tokens = tokenize(expression);

    let target_value = 479;
    let mut results = Vec::new();

    // Try placing parentheses around every possible sub-expression
    // We'll consider every pair of indices that correspond to a valid sub-expression.
    for start in 0..tokens.len() {
        for end in start + 1..tokens.len() {
            // We only consider substrings that contain at least one operator
            // (because parenthesizing just a single number or no operators doesn't make sense)
            if is_valid_subexpression(&tokens, start, end) {
                if let Some(value) = evaluate_with_parentheses(&tokens, start, end) {
                    if value == target_value {
                        // Record the actual string representation of this particular parenthesization
                        let parenthesized = insert_parentheses(&tokens, start, end);
                        results.push(parenthesized);
                    }
                }
            }
        }
    }

    // Remove duplicates
    results.sort();
    results.dedup();

    if results.is_empty() {
        println!(
            "No single-pair parenthetical placement found that results in {}.",
            target_value
        );
    } else {
        println!("Found the following ways to achieve {}:", target_value);
        for r in results {
            println!("{}", r);
        }
    }
}

/// Tokenize the input expression into numbers and operators.
fn tokenize(expr: &str) -> Vec<Token> {
    let mut tokens = Vec::new();
    for part in expr.split_whitespace() {
        if let Ok(num) = i64::from_str(part) {
            tokens.push(Token::Number(num));
        } else {
            match part {
                "+" => tokens.push(Token::Plus),
                "*" => tokens.push(Token::Mul),
                _ => {}
            }
        }
    }
    tokens
}

/// Check if the subexpression from `start` to `end` (inclusive) contains at least one operator.
fn is_valid_subexpression(tokens: &[Token], start: usize, end: usize) -> bool {
    let mut has_operator = false;
    let mut has_number = false;
    for i in start..=end {
        match tokens[i] {
            Token::Number(_) => has_number = true,
            Token::Plus | Token::Mul => has_operator = true,
        }
    }
    // Must contain at least one operator and at least one number
    has_operator && has_number
}

/// Evaluate the entire expression normally (with * having precedence over +).
fn evaluate_expression(tokens: &[Token]) -> Option<i64> {
    if tokens.is_empty() {
        return None; // No expression at all
    }

    // We'll parse the expression into terms separated by pluses.
    // Each term is a product of one or more numbers.
    let mut terms: Vec<i64> = Vec::new();
    let mut current_product: Option<i64> = None;

    enum Expectation {
        Number,
        Operator,
    }

    // At the start of the expression, we expect a number.
    let mut expectation = Expectation::Number;

    let mut i = 0;
    while i < tokens.len() {
        match (&tokens[i], &expectation) {
            // Expecting a number and we got one
            (Token::Number(n), Expectation::Number) => {
                // If we currently have no product in progress, start one.
                // If we do have one, that would mean we got two numbers in a row without an operator,
                // which should be invalid in a well-formed expression.
                if current_product.is_some() {
                    return None;
                }
                current_product = Some(*n);
                expectation = Expectation::Operator;
                i += 1;
            }

            // Expecting an operator and got a plus
            (Token::Plus, Expectation::Operator) => {
                // Plus means we commit the current product to terms and reset for the next term.
                if let Some(prod) = current_product.take() {
                    terms.push(prod);
                } else {
                    // We got a plus but no current product, invalid
                    return None;
                }
                expectation = Expectation::Number;
                i += 1;
            }

            // Expecting an operator and got a multiplication
            (Token::Mul, Expectation::Operator) => {
                // Multiplication means we should multiply the current product with the next number.
                // But we must check that the next token is a number.
                if current_product.is_none() {
                    return None;
                }
                if i + 1 >= tokens.len() {
                    return None; // Mul at the end with no following number
                }
                if let Token::Number(m) = tokens[i + 1] {
                    // multiply current_product by m
                    let prod = current_product.unwrap();
                    current_product = Some(prod * m);
                    i += 2; // move past Mul and the Number
                    expectation = Expectation::Operator;
                } else {
                    return None; // Mul not followed by a number
                }
            }

            // If we are expecting a number but got an operator, that's invalid
            (Token::Plus, Expectation::Number) | (Token::Mul, Expectation::Number) => {
                return None;
            }

            // If we are expecting an operator but got a number, that means no operator between them
            (Token::Number(_), Expectation::Operator) => {
                return None;
            }
        }
    }

    // At the end, if we were expecting an operator, that means expression ended with a number
    // which is okay, but we must add the last product to terms.
    // If expectation was Number, that means it ended with an operator, which is invalid.
    match expectation {
        Expectation::Number => {
            // Expression ended expecting a number, means ended on an operator like "1 +"
            return None;
        }
        Expectation::Operator => {
            if let Some(prod) = current_product {
                terms.push(prod);
            } else {
                // No product at the end, shouldn't happen if we got this far
                return None;
            }
        }
    }

    // Now sum all terms
    let sum: i64 = terms.into_iter().sum();
    Some(sum)
}

/// Evaluate a subexpression defined by [start, end], then replace that portion in the original
/// tokens with its single evaluated result, and then evaluate the entire expression.
fn evaluate_with_parentheses(tokens: &[Token], start: usize, end: usize) -> Option<i64> {
    // Extract the subexpression
    let sub_tokens = &tokens[start..=end];
    // Evaluate the subexpression on its own
    let sub_value = evaluate_expression(sub_tokens)?;

    // Now replace this portion in the original token list with sub_value
    let mut new_tokens = Vec::new();
    new_tokens.extend_from_slice(&tokens[0..start]);
    new_tokens.push(Token::Number(sub_value));
    new_tokens.extend_from_slice(&tokens[end + 1..]);

    // Evaluate the resulting expression
    let results = evaluate_expression(&new_tokens);
    results
}

/// Insert parentheses into the original string representation given the token indices.
/// This is just a heuristic reconstruction to show how parentheses are inserted.
fn insert_parentheses(tokens: &[Token], start: usize, end: usize) -> String {
    // We'll map token indices back to their string positions.
    // A simple approach is to re-construct the expression from tokens and insert parentheses
    // around the subexpression of interest by counting tokens.
    let pieces: Vec<String> = tokens
        .iter()
        .map(|t| match t {
            Token::Number(n) => n.to_string(),
            Token::Plus => "+".to_string(),
            Token::Mul => "*".to_string(),
        })
        .collect();

    // We know pieces are spaced out as in `1 + 2 * 3 ...`
    // We'll join them with spaces and then insert parentheses.
    // However, this won't exactly match original spacing if it was different,
    // but will produce a logically equivalent expression.

    let mut with_paren = String::new();
    for (i, p) in pieces.iter().enumerate() {
        if i == start {
            with_paren.push('(');
        }
        if !with_paren.is_empty() && !with_paren.ends_with('(') {
            with_paren.push(' ');
        }
        with_paren.push_str(p);
        if i == end {
            with_paren.push(')');
        }
    }

    with_paren
}

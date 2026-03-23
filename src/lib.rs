use pest::{Parser, iterators::Pair};
use pest_derive::Parser;
use std::fmt;

#[cfg(feature = "sea-query")]
pub mod sea_query;

#[derive(Parser)]
#[grammar = "src/re.pest"]
pub struct RExprParser;

#[derive(Debug, PartialEq, Clone, Copy)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum BinaryOperator {
    // Arithmetic
    Add,
    Subtract,
    Multiply,
    Divide,
    Modulo,
    // Comparison
    Equal,
    NotEqual,
    Less,
    LessEqual,
    Greater,
    GreaterEqual,
    // Logical
    And,
    Or,
}

#[derive(Debug, PartialEq, Clone, Copy)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum UnaryOperator {
    Not,
    Negate,
    BitwiseNot,
}

impl fmt::Display for BinaryOperator {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            BinaryOperator::Add => write!(f, "+"),
            BinaryOperator::Subtract => write!(f, "-"),
            BinaryOperator::Multiply => write!(f, "*"),
            BinaryOperator::Divide => write!(f, "/"),
            BinaryOperator::Modulo => write!(f, "%"),
            BinaryOperator::Equal => write!(f, "=="),
            BinaryOperator::NotEqual => write!(f, "!="),
            BinaryOperator::Less => write!(f, "<"),
            BinaryOperator::LessEqual => write!(f, "<="),
            BinaryOperator::Greater => write!(f, ">"),
            BinaryOperator::GreaterEqual => write!(f, ">="),
            BinaryOperator::And => write!(f, "&&"),
            BinaryOperator::Or => write!(f, "||"),
        }
    }
}

impl fmt::Display for UnaryOperator {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            UnaryOperator::Not => write!(f, "!"),
            UnaryOperator::Negate => write!(f, "-"),
            UnaryOperator::BitwiseNot => write!(f, "~"),
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum Value {
    Integer(i64),
    Float(f64),
    String(String),
    Identifier(String),
    Accessor(Box<Value>, Box<Value>), // For dot and bracket accessors
    FunctionCall(String, Vec<Value>), // For direct function calls
    MethodCall(Box<Value>, String, Vec<Value>), // For method calls
    BinaryOp(BinaryOperator, Box<Value>, Box<Value>),
    UnaryOp(UnaryOperator, Box<Value>),
}

pub fn parse(input: &str) -> Result<Value, pest::error::Error<Rule>> {
    let mut pairs = RExprParser::parse(Rule::program, input)?;
    let program_pair = pairs.next().unwrap();

    // Extract the value from the program (program -> SOI ~ logical_or ~ EOI)
    let value_pair = program_pair
        .into_inner()
        .find(|p| p.as_rule() == Rule::logical_or)
        .unwrap();

    Ok(pair_to_value(value_pair))
}

fn pair_to_value(pair: Pair<Rule>) -> Value {
    match pair.as_rule() {
        Rule::logical_or => {
            let mut inner = pair.into_inner();
            let mut left = pair_to_value(inner.next().unwrap());

            for right_pair in inner {
                if right_pair.as_rule() == Rule::logical_and {
                    let right = pair_to_value(right_pair);
                    left = Value::BinaryOp(BinaryOperator::Or, Box::new(left), Box::new(right));
                }
            }

            left
        }
        Rule::logical_and => {
            let mut inner = pair.into_inner();
            let mut left = pair_to_value(inner.next().unwrap());

            for right_pair in inner {
                if right_pair.as_rule() == Rule::comparison {
                    let right = pair_to_value(right_pair);
                    left = Value::BinaryOp(BinaryOperator::And, Box::new(left), Box::new(right));
                }
            }

            left
        }
        Rule::comparison => {
            let mut inner = pair.into_inner();
            let mut left = pair_to_value(inner.next().unwrap());

            while let Some(op_pair) = inner.next() {
                if op_pair.as_rule() == Rule::cmp_op {
                    let op = match op_pair.as_str() {
                        "==" => BinaryOperator::Equal,
                        "!=" => BinaryOperator::NotEqual,
                        "<" => BinaryOperator::Less,
                        "<=" => BinaryOperator::LessEqual,
                        ">" => BinaryOperator::Greater,
                        ">=" => BinaryOperator::GreaterEqual,
                        _ => panic!("Unknown comparison operator: {}", op_pair.as_str()),
                    };

                    let right = pair_to_value(inner.next().unwrap());
                    left = Value::BinaryOp(op, Box::new(left), Box::new(right));
                }
            }

            left
        }
        Rule::additive => {
            let mut inner = pair.into_inner();
            let mut left = pair_to_value(inner.next().unwrap());

            while let Some(op_pair) = inner.next() {
                if op_pair.as_rule() == Rule::add_op {
                    let op = match op_pair.as_str() {
                        "+" => BinaryOperator::Add,
                        "-" => BinaryOperator::Subtract,
                        _ => panic!("Unknown additive operator: {}", op_pair.as_str()),
                    };

                    let right = pair_to_value(inner.next().unwrap());
                    left = Value::BinaryOp(op, Box::new(left), Box::new(right));
                }
            }

            left
        }
        Rule::multiplicative => {
            let mut inner = pair.into_inner();
            let mut left = pair_to_value(inner.next().unwrap());

            while let Some(op_pair) = inner.next() {
                if op_pair.as_rule() == Rule::mult_op {
                    let op = match op_pair.as_str() {
                        "*" => BinaryOperator::Multiply,
                        "/" => BinaryOperator::Divide,
                        "%" => BinaryOperator::Modulo,
                        _ => panic!("Unknown multiplicative operator: {}", op_pair.as_str()),
                    };

                    let right = pair_to_value(inner.next().unwrap());
                    left = Value::BinaryOp(op, Box::new(left), Box::new(right));
                }
            }

            left
        }
        Rule::unary => {
            let mut inner = pair.into_inner();
            let first = inner.next().unwrap();

            if first.as_rule() == Rule::unary_op {
                // We have a unary operator
                let op = match first.as_str() {
                    "!" => UnaryOperator::Not,
                    "-" => UnaryOperator::Negate,
                    "~" => UnaryOperator::BitwiseNot,
                    _ => panic!("Unknown unary operator: {}", first.as_str()),
                };
                let operand = pair_to_value(inner.next().unwrap());
                Value::UnaryOp(op, Box::new(operand))
            } else {
                // No unary operator, just pass through to primary
                pair_to_value(first)
            }
        }
        Rule::primary => {
            // primary contains either (logical_or) or value
            let inner = pair.into_inner().next().unwrap();
            pair_to_value(inner)
        }
        Rule::value => {
            // value contains exactly one of: method_call, function_call, accessor_chain, float, integer, string
            let inner = pair.into_inner().next().unwrap();
            pair_to_value(inner)
        }
        Rule::method_call => {
            let mut inner = pair.into_inner();
            
            // First element is always the accessor_chain (the base object)
            let accessor_pair = inner.next().unwrap();
            let mut result = pair_to_value(accessor_pair);
            
            // Iterate through all method_suffix pairs and build left-associative chain
            // Each method_suffix is: ":" ~ identifier ~ "(" ~ (arg ~ ("," ~ arg)*)? ~ ")"
            for suffix_pair in inner {
                if suffix_pair.as_rule() == Rule::method_suffix {
                    let mut suffix_inner = suffix_pair.into_inner();
                    
                    // Extract method name (identifier)
                    let method_name_pair = suffix_inner.next().unwrap();
                    let method_name = method_name_pair.as_str().to_string();
                    
                    // Collect all args for this method
                    let args = suffix_inner
                        .filter_map(|p| {
                            if p.as_rule() == Rule::arg {
                                let arg_inner = p.into_inner().next().unwrap();
                                Some(pair_to_value(arg_inner))
                            } else {
                                None
                            }
                        })
                        .collect();
                    
                    // Build left-associative chain: wrap previous result in new MethodCall
                    result = Value::MethodCall(Box::new(result), method_name, args);
                }
            }
            
            result
        }
        Rule::function_call => {
            let mut inner = pair.into_inner();
            let func_name_pair = inner.next().unwrap(); // identifier
            let func_name = func_name_pair.as_str().to_string();

            // Collect all args
            let args = inner
                .filter_map(|p| {
                    if p.as_rule() == Rule::arg {
                        let arg_inner = p.into_inner().next().unwrap();
                        Some(pair_to_value(arg_inner))
                    } else {
                        None
                    }
                })
                .collect();

            Value::FunctionCall(func_name, args)
        }
        Rule::accessor_chain => {
            let mut inner = pair.into_inner();
            let id_pair = inner.next().unwrap(); // identifier
            let id = id_pair.as_str().to_string();

            let mut value = Value::Identifier(id);

            // Apply accessor suffixes in sequence
            for suffix_pair in inner {
                if suffix_pair.as_rule() == Rule::accessor_suffix {
                    let mut suffix_inner = suffix_pair.into_inner();
                    let first_token = suffix_inner.next().unwrap();

                    match first_token.as_rule() {
                        Rule::identifier => {
                            // Dot accessor: .field
                            let field_name = first_token.as_str().to_string();
                            value = Value::Accessor(
                                Box::new(value),
                                Box::new(Value::Identifier(field_name)),
                            );
                        }
                        Rule::index_value => {
                            // Bracket accessor: [index]
                            let index_value = pair_to_value(first_token);
                            value = Value::Accessor(Box::new(value), Box::new(index_value));
                        }
                        _ => {}
                    }
                }
            }

            value
        }
        Rule::identifier => Value::Identifier(pair.as_str().to_string()),
        Rule::integer | Rule::integer_with_sign => {
            let num = pair.as_str().parse::<i64>().unwrap();
            Value::Integer(num)
        }
        Rule::float | Rule::float_with_sign => {
            let num = pair.as_str().parse::<f64>().unwrap();
            Value::Float(num)
        }
        Rule::string => {
            let s = pair.as_str();
            let content = &s[1..s.len() - 1]; // Remove surrounding quotes
            let unescaped = unescape_string(content);
            Value::String(unescaped)
        }
        Rule::index_value => {
            // index_value is a wrapper; unwrap and parse the inner value
            let inner = pair.into_inner().next().unwrap();
            pair_to_value(inner)
        }
        Rule::arg => {
            // arg is a wrapper; unwrap and parse the inner value
            let inner = pair.into_inner().next().unwrap();
            pair_to_value(inner)
        }
        _ => panic!("Unexpected rule: {:?}", pair.as_rule()),
    }
}

fn unescape_string(s: &str) -> String {
    let mut result = String::new();
    let mut chars = s.chars();

    while let Some(ch) = chars.next() {
        if ch == '\\' {
            if let Some(next) = chars.next() {
                match next {
                    'n' => result.push('\n'),
                    't' => result.push('\t'),
                    'r' => result.push('\r'),
                    '\\' => result.push('\\'),
                    '"' => result.push('"'),
                    c => {
                        // For any other escape sequence, keep the backslash
                        result.push('\\');
                        result.push(c);
                    }
                }
            }
        } else {
            result.push(ch);
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse(input: &str) -> Value {
        super::parse(input).expect("Failed to parse")
    }

    fn root_level1_0_level2_key_obj() -> Value {
        Value::Accessor(
            Box::new(Value::Accessor(
                Box::new(Value::Accessor(
                    Box::new(Value::Accessor(
                        Box::new(Value::Accessor(
                            Box::new(Value::Identifier("root".to_string())),
                            Box::new(Value::Identifier("level1".to_string())),
                        )),
                        Box::new(Value::Integer(0)),
                    )),
                    Box::new(Value::Identifier("level2".to_string())),
                )),
                Box::new(Value::String("key".to_string())),
            )),
            Box::new(Value::Identifier("obj".to_string())),
        )
    }

    #[rstest::rstest]
    // Basic integers
    #[case::zero("0", Value::Integer(0))]
    #[case::positive_single_digit("5", Value::Integer(5))]
    #[case::positive_multi_digit("12345", Value::Integer(12345))]
    #[case::negative_single_digit("-5", Value::UnaryOp(UnaryOperator::Negate, Box::new(Value::Integer(5))))]
    #[case::negative_multi_digit("-98765", Value::UnaryOp(UnaryOperator::Negate, Box::new(Value::Integer(98765))))]
    #[case::large_integer("999999999", Value::Integer(999999999))]
    // Basic floats
    #[case::float_one_decimal("1.0", Value::Float(1.0))]
    #[case::float_multi_decimal("1.15", Value::Float(1.15))]
    #[case::float_leading_zero("0.5", Value::Float(0.5))]
    #[case::negative_float("-2.5", Value::UnaryOp(UnaryOperator::Negate, Box::new(Value::Float(2.5))))]
    #[case::scientific_positive_exp("1.5e10", Value::Float(1.5e10))]
    #[case::scientific_negative_exp("1.5e-10", Value::Float(1.5e-10))]
    #[case::scientific_plus_sign("1.5e+10", Value::Float(1.5e+10))]
    #[case::scientific_uppercase_e("1.5E-10", Value::Float(1.5E-10))]
    // Strings
    #[case::empty_string(r#""""#, Value::String("".to_string()))]
    #[case::single_char_string(r#""a""#, Value::String("a".to_string()))]
    #[case::string_with_spaces(r#""hello world""#, Value::String("hello world".to_string()))]
    #[case::string_with_escape_newline(r#""line1\nline2""#, Value::String("line1\nline2".to_string()))]
    #[case::string_with_escape_tab(r#""col1\tcol2""#, Value::String("col1\tcol2".to_string()))]
    #[case::string_with_escape_backslash(r#""path\\to\\file""#, Value::String("path\\to\\file".to_string()))]
    #[case::string_with_escape_quote(r#""say \"hi\"""#, Value::String("say \"hi\"".to_string()))]
    #[case::string_mixed_escapes(r#""a\nb\tc\d\"e""#, Value::String("a\nb\tc\\d\"e".to_string()))]
    fn test_literals(#[case] input: &str, #[case] expected: Value) {
        let result = parse(input);
        assert_eq!(result, expected);
    }

    #[rstest::rstest]
    // Simple accessors
    #[case::single_identifier("my_var", Value::Identifier("my_var".to_string()))]
    #[case::underscore_prefix("_private", Value::Identifier("_private".to_string()))]
    #[case::underscore_only("_", Value::Identifier("_".to_string()))]
    // Dot accessors
    #[case::single_dot("obj.prop", Value::Accessor(Box::new(Value::Identifier("obj".to_string())), Box::new(Value::Identifier("prop".to_string()))))]
    #[case::double_dot("obj.prop.field", Value::Accessor(Box::new(Value::Accessor(Box::new(Value::Identifier("obj".to_string())), Box::new(Value::Identifier("prop".to_string())))), Box::new(Value::Identifier("field".to_string()))))]
    #[case::triple_dot("a.b.c.d", Value::Accessor(Box::new(Value::Accessor(Box::new(Value::Accessor(Box::new(Value::Identifier("a".to_string())), Box::new(Value::Identifier("b".to_string())))), Box::new(Value::Identifier("c".to_string())))), Box::new(Value::Identifier("d".to_string()))))]
    #[case::dot_with_numbers("var1.field2", Value::Accessor(Box::new(Value::Identifier("var1".to_string())), Box::new(Value::Identifier("field2".to_string()))))]
    // Bracket accessors - integers
    #[case::bracket_zero("arr[0]", Value::Accessor(Box::new(Value::Identifier("arr".to_string())), Box::new(Value::Integer(0))))]
    #[case::bracket_positive_int("arr[42]", Value::Accessor(Box::new(Value::Identifier("arr".to_string())), Box::new(Value::Integer(42))))]
    #[case::bracket_negative_int("arr[-5]", Value::Accessor(Box::new(Value::Identifier("arr".to_string())), Box::new(Value::Integer(-5))))]
    #[case::bracket_large_int("arr[999999]", Value::Accessor(Box::new(Value::Identifier("arr".to_string())), Box::new(Value::Integer(999999))))]
    // Bracket accessors - strings
    #[case::bracket_string_key(r#"obj["key"]"#, Value::Accessor(Box::new(Value::Identifier("obj".to_string())), Box::new(Value::String("key".to_string()))))]
    #[case::bracket_empty_string(r#"obj[""]"#, Value::Accessor(Box::new(Value::Identifier("obj".to_string())), Box::new(Value::String("".to_string()))))]
    #[case::bracket_string_with_space(r#"obj["my key"]"#, Value::Accessor(Box::new(Value::Identifier("obj".to_string())), Box::new(Value::String("my key".to_string()))))]
    #[case::bracket_string_with_special(r#"obj["key_123"]"#, Value::Accessor(Box::new(Value::Identifier("obj".to_string())), Box::new(Value::String("key_123".to_string()))))]
    // Mixed and complex chains
    #[case::dot_then_bracket("obj.arr[0]", Value::Accessor(Box::new(Value::Accessor(Box::new(Value::Identifier("obj".to_string())), Box::new(Value::Identifier("arr".to_string())))), Box::new(Value::Integer(0))))]
    #[case::bracket_then_dot("arr[0].prop", Value::Accessor(Box::new(Value::Accessor(Box::new(Value::Identifier("arr".to_string())), Box::new(Value::Integer(0)))), Box::new(Value::Identifier("prop".to_string()))))]
    #[case::multiple_brackets("data[0][1][2]", Value::Accessor(Box::new(Value::Accessor(Box::new(Value::Accessor(Box::new(Value::Identifier("data".to_string())), Box::new(Value::Integer(0)))), Box::new(Value::Integer(1)))), Box::new(Value::Integer(2))))]
    #[case::multiple_brackets_string_keys(r#"obj["a"]["b"]["c"]"#, Value::Accessor(Box::new(Value::Accessor(Box::new(Value::Accessor(Box::new(Value::Identifier("obj".to_string())), Box::new(Value::String("a".to_string())))), Box::new(Value::String("b".to_string())))), Box::new(Value::String("c".to_string()))))]
    #[case::identifier_with_underscores("my_obj._private[0]._field", Value::Accessor(Box::new(Value::Accessor(Box::new(Value::Accessor(Box::new(Value::Identifier("my_obj".to_string())), Box::new(Value::Identifier("_private".to_string())))), Box::new(Value::Integer(0)))), Box::new(Value::Identifier("_field".to_string()))))]
    fn test_accessors(#[case] input: &str, #[case] expected: Value) {
        let result = parse(input);
        assert_eq!(result, expected);
    }

    #[rstest::rstest]
    fn test_accessor_alternating_dots_brackets() {
        let expected = Value::Accessor(
            Box::new(Value::Accessor(
                Box::new(Value::Accessor(
                    Box::new(Value::Accessor(
                        Box::new(Value::Accessor(
                            Box::new(Value::Identifier("obj".to_string())),
                            Box::new(Value::Identifier("arr".to_string())),
                        )),
                        Box::new(Value::Integer(0)),
                    )),
                    Box::new(Value::Identifier("item".to_string())),
                )),
                Box::new(Value::String("key".to_string())),
            )),
            Box::new(Value::Identifier("value".to_string())),
        );
        let result = parse(r#"obj.arr[0].item["key"].value"#);
        assert_eq!(result, expected);
    }

    #[rstest::rstest]
    fn test_accessor_complex_chain() {
        let chain_to_1 = Value::Accessor(
            Box::new(Value::Accessor(
                Box::new(Value::Accessor(
                    Box::new(Value::Accessor(
                        Box::new(Value::Accessor(
                            Box::new(Value::Accessor(
                                Box::new(Value::Identifier("root".to_string())),
                                Box::new(Value::Identifier("users".to_string())),
                            )),
                            Box::new(Value::Integer(0)),
                        )),
                        Box::new(Value::Identifier("profile".to_string())),
                    )),
                    Box::new(Value::String("email".to_string())),
                )),
                Box::new(Value::Identifier("address".to_string())),
            )),
            Box::new(Value::Integer(1)),
        );
        let expected = Value::Accessor(
            Box::new(chain_to_1),
            Box::new(Value::Identifier("street".to_string())),
        );
        let result = parse(r#"root.users[0].profile["email"].address[1].street"#);
        assert_eq!(result, expected);
    }

    #[rstest::rstest]
    // No arguments
    #[case::func_empty_args("func()", Value::FunctionCall("func".to_string(), vec![]))]
    #[case::func_no_args_underscore("_func()", Value::FunctionCall("_func".to_string(), vec![]))]
    // Single argument - literals
    #[case::func_int_arg("add(5)", Value::FunctionCall("add".to_string(), vec![Value::Integer(5)]))]
    #[case::func_negative_int("sub(-3)", Value::FunctionCall("sub".to_string(), vec![Value::Integer(-3)]))]
    #[case::func_float_arg("multiply(2.5)", Value::FunctionCall("multiply".to_string(), vec![Value::Float(2.5)]))]
    #[case::func_negative_float("divide(-1.5)", Value::FunctionCall("divide".to_string(), vec![Value::Float(-1.5)]))]
    #[case::func_string_arg(r#"greet("Alice")"#, Value::FunctionCall("greet".to_string(), vec![Value::String("Alice".to_string())]))]
    // Multiple arguments - same type
    #[case::func_two_ints("add(1, 2)", Value::FunctionCall("add".to_string(), vec![Value::Integer(1), Value::Integer(2)]))]
    #[case::func_three_ints("sum(1, 2, 3)", Value::FunctionCall("sum".to_string(), vec![Value::Integer(1), Value::Integer(2), Value::Integer(3)]))]
    #[case::func_two_floats("calc(1.5, 2.5)", Value::FunctionCall("calc".to_string(), vec![Value::Float(1.5), Value::Float(2.5)]))]
    #[case::func_two_strings(r#"concat("hello", "world")"#, Value::FunctionCall("concat".to_string(), vec![Value::String("hello".to_string()), Value::String("world".to_string())]))]
    // Multiple arguments - mixed types
    #[case::func_int_float("mix(1, 2.5)", Value::FunctionCall("mix".to_string(), vec![Value::Integer(1), Value::Float(2.5)]))]
    #[case::func_int_string(r#"describe(42, "answer")"#, Value::FunctionCall("describe".to_string(), vec![Value::Integer(42), Value::String("answer".to_string())]))]
    #[case::func_float_string(r#"format(1.15, "pi")"#, Value::FunctionCall("format".to_string(), vec![Value::Float(1.15), Value::String("pi".to_string())]))]
    #[case::func_int_float_string(r#"complex(5, 2.5, "test")"#, Value::FunctionCall("complex".to_string(), vec![Value::Integer(5), Value::Float(2.5), Value::String("test".to_string())]))]
    // Many arguments
    #[case::func_four_mixed(r#"process(1, 2.5, "text", -3)"#, Value::FunctionCall("process".to_string(), vec![Value::Integer(1), Value::Float(2.5), Value::String("text".to_string()), Value::Integer(-3)]))]
    #[case::func_five_args(r#"calculate(0, 1.1, 2.2, "x", -5)"#, Value::FunctionCall("calculate".to_string(), vec![Value::Integer(0), Value::Float(1.1), Value::Float(2.2), Value::String("x".to_string()), Value::Integer(-5)]))]
    // Zero and negative arguments
    #[case::func_zero_int("count(0)", Value::FunctionCall("count".to_string(), vec![Value::Integer(0)]))]
    #[case::func_zero_float("measure(0.0)", Value::FunctionCall("measure".to_string(), vec![Value::Float(0.0)]))]
    #[case::func_all_zeros("zeros(0, 0, 0)", Value::FunctionCall("zeros".to_string(), vec![Value::Integer(0), Value::Integer(0), Value::Integer(0)]))]
    // Large numbers as arguments
    #[case::func_large_int("big(999999999)", Value::FunctionCall("big".to_string(), vec![Value::Integer(999999999)]))]
    #[case::func_scientific("sci(1.23e45)", Value::FunctionCall("sci".to_string(), vec![Value::Float(1.23e45)]))]
    fn test_direct_function_calls(#[case] input: &str, #[case] expected: Value) {
        let result = parse(input);
        assert_eq!(result, expected);
    }

    #[rstest::rstest]
    // Simple method calls - no args
    #[case::method_simple("obj:method()", Value::MethodCall(Box::new(Value::Identifier("obj".to_string())), "method".to_string(), vec![]))]
    #[case::method_underscore("_obj:_method()", Value::MethodCall(Box::new(Value::Identifier("_obj".to_string())), "_method".to_string(), vec![]))]
    // Method calls with arguments
    #[case::method_single_int("obj:process(5)", Value::MethodCall(Box::new(Value::Identifier("obj".to_string())), "process".to_string(), vec![Value::Integer(5)]))]
    #[case::method_single_string("obj:set(\"value\")", Value::MethodCall(Box::new(Value::Identifier("obj".to_string())), "set".to_string(), vec![Value::String("value".to_string())]))]
    #[case::method_two_args("obj:init(1, 2)", Value::MethodCall(Box::new(Value::Identifier("obj".to_string())), "init".to_string(), vec![Value::Integer(1), Value::Integer(2)]))]
    #[case::method_mixed_args("obj:configure(42, \"settings\", 1.15)", Value::MethodCall(Box::new(Value::Identifier("obj".to_string())), "configure".to_string(), vec![Value::Integer(42), Value::String("settings".to_string()), Value::Float(1.15)]))]
    // Method on accessors - single level
    #[case::method_on_dot("obj.prop:method()", Value::MethodCall(Box::new(Value::Accessor(Box::new(Value::Identifier("obj".to_string())), Box::new(Value::Identifier("prop".to_string())))), "method".to_string(), vec![]))]
    #[case::method_on_bracket_int("arr[0]:process()", Value::MethodCall(Box::new(Value::Accessor(Box::new(Value::Identifier("arr".to_string())), Box::new(Value::Integer(0)))), "process".to_string(), vec![]))]
    #[case::method_on_bracket_string("dict[\"key\"]:apply()", Value::MethodCall(Box::new(Value::Accessor(Box::new(Value::Identifier("dict".to_string())), Box::new(Value::String("key".to_string())))), "apply".to_string(), vec![]))]
    // Method on complex chains
    #[case::method_on_chain_dot_bracket("obj.arr[0]:method()", Value::MethodCall(Box::new(Value::Accessor(Box::new(Value::Accessor(Box::new(Value::Identifier("obj".to_string())), Box::new(Value::Identifier("arr".to_string())))), Box::new(Value::Integer(0)))), "method".to_string(), vec![]))]
    #[case::method_on_chain_bracket_dot("arr[0].obj:method()", Value::MethodCall(Box::new(Value::Accessor(Box::new(Value::Accessor(Box::new(Value::Identifier("arr".to_string())), Box::new(Value::Integer(0)))), Box::new(Value::Identifier("obj".to_string())))), "method".to_string(), vec![]))]
    // Method calls with various argument combinations
    #[case::method_many_args("obj:process(1, 2.5, \"text\", -3, 0)", Value::MethodCall(Box::new(Value::Identifier("obj".to_string())), "process".to_string(), vec![Value::Integer(1), Value::Float(2.5), Value::String("text".to_string()), Value::Integer(-3), Value::Integer(0)]))]
    #[case::method_zero_arg("obj:reset(0)", Value::MethodCall(Box::new(Value::Identifier("obj".to_string())), "reset".to_string(), vec![Value::Integer(0)]))]
    #[case::method_negative_args("obj:apply(-1, -2.5)", Value::MethodCall(Box::new(Value::Identifier("obj".to_string())), "apply".to_string(), vec![Value::Integer(-1), Value::Float(-2.5)]))]
    // Chain with multiple access then method
    #[case::method_underscore_chain("_obj._prop[0]:_method()", Value::MethodCall(Box::new(Value::Accessor(Box::new(Value::Accessor(Box::new(Value::Identifier("_obj".to_string())), Box::new(Value::Identifier("_prop".to_string())))), Box::new(Value::Integer(0)))), "_method".to_string(), vec![]))]
    fn test_method_calls(#[case] input: &str, #[case] expected: Value) {
        let result = parse(input);
        assert_eq!(result, expected);
    }

    #[test]
    fn test_method_on_deep_chain() {
        let expected = Value::MethodCall(
            Box::new(root_level1_0_level2_key_obj()),
            "execute".to_string(),
            vec![],
        );
        let result = parse("root.level1[0].level2[\"key\"].obj:execute()");
        assert_eq!(result, expected);
    }

    #[rstest::rstest]
    // Simple two-method chains (left-associative: apply m1 first, then m2)
    #[case::chain_two_simple("obj:m1():m2()", Value::MethodCall(Box::new(Value::MethodCall(Box::new(Value::Identifier("obj".to_string())), "m1".to_string(), vec![])), "m2".to_string(), vec![]))]
    #[case::chain_two_with_first_args("obj:m1(5):m2()", Value::MethodCall(Box::new(Value::MethodCall(Box::new(Value::Identifier("obj".to_string())), "m1".to_string(), vec![Value::Integer(5)])), "m2".to_string(), vec![]))]
    #[case::chain_two_with_second_args("obj:m1():m2(10)", Value::MethodCall(Box::new(Value::MethodCall(Box::new(Value::Identifier("obj".to_string())), "m1".to_string(), vec![])), "m2".to_string(), vec![Value::Integer(10)]))]
    #[case::chain_two_with_both_args("obj:m1(5):m2(10)", Value::MethodCall(Box::new(Value::MethodCall(Box::new(Value::Identifier("obj".to_string())), "m1".to_string(), vec![Value::Integer(5)])), "m2".to_string(), vec![Value::Integer(10)]))]
    #[case::chain_two_with_string_args(r#"obj:m1("a"):m2("b")"#, Value::MethodCall(Box::new(Value::MethodCall(Box::new(Value::Identifier("obj".to_string())), "m1".to_string(), vec![Value::String("a".to_string())])), "m2".to_string(), vec![Value::String("b".to_string())]))]
    
    // Three-method chains
    #[case::chain_three_simple("obj:m1():m2():m3()", Value::MethodCall(Box::new(Value::MethodCall(Box::new(Value::MethodCall(Box::new(Value::Identifier("obj".to_string())), "m1".to_string(), vec![])), "m2".to_string(), vec![])), "m3".to_string(), vec![]))]
    #[case::chain_three_with_args("obj:m1(1):m2(2):m3(3)", Value::MethodCall(Box::new(Value::MethodCall(Box::new(Value::MethodCall(Box::new(Value::Identifier("obj".to_string())), "m1".to_string(), vec![Value::Integer(1)])), "m2".to_string(), vec![Value::Integer(2)])), "m3".to_string(), vec![Value::Integer(3)]))]
    
    // Four-method chains (extended length)
    #[case::chain_four("obj:a():b():c():d()", Value::MethodCall(Box::new(Value::MethodCall(Box::new(Value::MethodCall(Box::new(Value::MethodCall(Box::new(Value::Identifier("obj".to_string())), "a".to_string(), vec![])), "b".to_string(), vec![])), "c".to_string(), vec![])), "d".to_string(), vec![]))]
    
    // Chains on accessor chains (complex receivers)
    #[case::chain_on_dot_access("obj.prop:m1():m2()", Value::MethodCall(Box::new(Value::MethodCall(Box::new(Value::Accessor(Box::new(Value::Identifier("obj".to_string())), Box::new(Value::Identifier("prop".to_string())))), "m1".to_string(), vec![])), "m2".to_string(), vec![]))]
    #[case::chain_on_bracket_access("arr[0]:m1():m2()", Value::MethodCall(Box::new(Value::MethodCall(Box::new(Value::Accessor(Box::new(Value::Identifier("arr".to_string())), Box::new(Value::Integer(0)))), "m1".to_string(), vec![])), "m2".to_string(), vec![]))]
    #[case::chain_on_mixed_access("obj.arr[0]:m1():m2()", Value::MethodCall(Box::new(Value::MethodCall(Box::new(Value::Accessor(Box::new(Value::Accessor(Box::new(Value::Identifier("obj".to_string())), Box::new(Value::Identifier("arr".to_string())))), Box::new(Value::Integer(0)))), "m1".to_string(), vec![])), "m2".to_string(), vec![]))]
    
    // Chains with mixed argument types
    #[case::chain_mixed_args("obj:m1(1, \"x\"):m2(2.5, \"y\")", Value::MethodCall(Box::new(Value::MethodCall(Box::new(Value::Identifier("obj".to_string())), "m1".to_string(), vec![Value::Integer(1), Value::String("x".to_string())])), "m2".to_string(), vec![Value::Float(2.5), Value::String("y".to_string())]))]
    
    // Chains with underscores in names
    #[case::chain_underscore_names("_obj:_m1():_m2()", Value::MethodCall(Box::new(Value::MethodCall(Box::new(Value::Identifier("_obj".to_string())), "_m1".to_string(), vec![])), "_m2".to_string(), vec![]))]
    fn test_method_chaining(#[case] input: &str, #[case] expected: Value) {
        let result = parse(input);
        assert_eq!(result, expected);
    }

    #[rstest::rstest]
    #[case("my_func(my_var)")]
    #[case("my_var[3.14]")]
    #[case("my_var:my_func(obj.field)")]
    #[case("func(arr[0])")]
    #[case("func(nested.prop)")]
    #[case("obj:method(1, nested.x)")]
    #[case("obj[1.5]")]
    #[case("obj[my_func()]")]
    #[should_panic]
    fn test_invalid_syntax(#[case] input: &str) {
        parse(input);
    }

    // Test cases based on README.md examples to verify documentation accuracy
    #[rstest::rstest]
    // Accessor examples from Syntax Reference section
    #[case::readme_accessor_simple("user")]
    #[case::readme_accessor_dot("user.profile")]
    #[case::readme_accessor_chained_dot("user.profile.email")]
    #[case::readme_accessor_bracket("user[\"profile\"]")]
    #[case::readme_accessor_chained_bracket(r#"user["profile"]["email"]"#)]
    #[case::readme_accessor_mixed(r#"user.profile["email"]"#)]
    // Function call examples from Syntax Reference section
    #[case::readme_func_sum("sum(10, 20)")]
    #[case::readme_func_concat(r#"concat("hello", "world")"#)]
    #[case::readme_func_max("max(42.5, 100, -5)")]
    #[case::readme_func_floor("floor(3.14)")]
    // Literal examples from Syntax Reference section
    #[case::readme_literal_int("42")]
    #[case::readme_literal_float("3.14")]
    #[case::readme_literal_scientific("-2.5e-3")]
    #[case::readme_literal_string(r#""hello world""#)]
    #[case::readme_literal_escaped_string(r#""escaped \"quotes\"""#)]
    // Method call examples from Syntax Reference section
    #[case::readme_method_simple("user.profile:transform()")]
    #[case::readme_method_bracket("data[\"items\"]:count()")]
    #[case::readme_method_with_args(r#"collection["data"]["key"]:upper("prefix", "suffix")"#)]
    // Use case examples from README
    #[case::readme_usecase_sql_sum("revenue:sum()")]
    #[case::readme_usecase_sql_count(r#"orders["status"]:count()"#)]
    #[case::readme_usecase_dsl_cache("cache.ttl:set(3600)")]
    fn test_readme_examples(#[case] input: &str) {
        // Simply verify that the examples from README parse without error
        let _ = parse(input); // parse() from the test helper already unwraps and panics on error
    }

    // Test the specific example from the Quick Start section
    #[test]
    fn test_readme_quick_start_example() {
        let expr = parse("user.profile.email");
        let expected = Value::Accessor(
            Box::new(Value::Accessor(
                Box::new(Value::Identifier("user".to_string())),
                Box::new(Value::Identifier("profile".to_string())),
            )),
            Box::new(Value::Identifier("email".to_string())),
        );
        assert_eq!(expr, expected);
    }

    // Test the specific example from the API Documentation section
    #[test]
    fn test_readme_api_doc_example() {
        let expr = parse("user.name");
        match expr {
            Value::Accessor(parent, field) => {
                // Verify the structure matches the documentation
                assert!(matches!(*parent, Value::Identifier(_)));
                assert!(matches!(*field, Value::Identifier(_)));
            }
            _ => panic!("Expected Accessor variant"),
        }
    }

    // Test the SQL integration example from README
    #[test]
    fn test_readme_sql_integration_sum_example() {
        let expr = parse("sales:sum()");
        match expr {
            Value::MethodCall(obj, method, args) => {
                assert!(matches!(*obj, Value::Identifier(_)));
                assert_eq!(method, "sum");
                assert!(args.is_empty());
            }
            _ => panic!("Expected MethodCall variant"),
        }
    }

    // Test the SQL integration example from README
    #[test]
    fn test_readme_sql_integration_accessor_example() {
        let expr = parse("user.created_at");
        assert!(matches!(expr, Value::Accessor(_, _)));
    }

    #[rstest::rstest]
    // Simple arithmetic operators
    #[case::add_simple("1 + 2", Value::BinaryOp(BinaryOperator::Add, Box::new(Value::Integer(1)), Box::new(Value::Integer(2))))]
    #[case::subtract_simple("5 - 3", Value::BinaryOp(BinaryOperator::Subtract, Box::new(Value::Integer(5)), Box::new(Value::Integer(3))))]
    #[case::multiply_simple("3 * 4", Value::BinaryOp(BinaryOperator::Multiply, Box::new(Value::Integer(3)), Box::new(Value::Integer(4))))]
    #[case::divide_simple("10 / 2", Value::BinaryOp(BinaryOperator::Divide, Box::new(Value::Integer(10)), Box::new(Value::Integer(2))))]
    #[case::modulo_simple("10 % 3", Value::BinaryOp(BinaryOperator::Modulo, Box::new(Value::Integer(10)), Box::new(Value::Integer(3))))]
    // Comparison operators
    #[case::equal_simple("1 == 1", Value::BinaryOp(BinaryOperator::Equal, Box::new(Value::Integer(1)), Box::new(Value::Integer(1))))]
    #[case::not_equal_simple("1 != 2", Value::BinaryOp(BinaryOperator::NotEqual, Box::new(Value::Integer(1)), Box::new(Value::Integer(2))))]
    #[case::less_simple("1 < 2", Value::BinaryOp(BinaryOperator::Less, Box::new(Value::Integer(1)), Box::new(Value::Integer(2))))]
    #[case::less_equal_simple("1 <= 2", Value::BinaryOp(BinaryOperator::LessEqual, Box::new(Value::Integer(1)), Box::new(Value::Integer(2))))]
    #[case::greater_simple("2 > 1", Value::BinaryOp(BinaryOperator::Greater, Box::new(Value::Integer(2)), Box::new(Value::Integer(1))))]
    #[case::greater_equal_simple("2 >= 1", Value::BinaryOp(BinaryOperator::GreaterEqual, Box::new(Value::Integer(2)), Box::new(Value::Integer(1))))]
    // Logical operators
    #[case::and_simple("1 && 2", Value::BinaryOp(BinaryOperator::And, Box::new(Value::Integer(1)), Box::new(Value::Integer(2))))]
    #[case::or_simple("1 || 2", Value::BinaryOp(BinaryOperator::Or, Box::new(Value::Integer(1)), Box::new(Value::Integer(2))))]
    // Unary operators
    #[case::not_unary("!a", Value::UnaryOp(UnaryOperator::Not, Box::new(Value::Identifier("a".to_string()))))]
    #[case::negate_unary("-5", Value::UnaryOp(UnaryOperator::Negate, Box::new(Value::Integer(5))))]
    #[case::bitnot_unary("~x", Value::UnaryOp(UnaryOperator::BitwiseNot, Box::new(Value::Identifier("x".to_string()))))]
    fn test_single_operators(#[case] input: &str, #[case] expected: Value) {
        let result = parse(input);
        assert_eq!(result, expected);
    }

    #[rstest::rstest]
    // Precedence: multiplicative before additive
    #[case::multiply_before_add("1 + 2 * 3", 
        Value::BinaryOp(BinaryOperator::Add, 
            Box::new(Value::Integer(1)),
            Box::new(Value::BinaryOp(BinaryOperator::Multiply, 
                Box::new(Value::Integer(2)), 
                Box::new(Value::Integer(3))))))]
    #[case::divide_before_subtract("10 - 6 / 2", 
        Value::BinaryOp(BinaryOperator::Subtract, 
            Box::new(Value::Integer(10)),
            Box::new(Value::BinaryOp(BinaryOperator::Divide, 
                Box::new(Value::Integer(6)), 
                Box::new(Value::Integer(2))))))]
    // Precedence: comparison before logical
    #[case::comparison_before_and("1 < 2 && 3 < 4", 
        Value::BinaryOp(BinaryOperator::And,
            Box::new(Value::BinaryOp(BinaryOperator::Less, Box::new(Value::Integer(1)), Box::new(Value::Integer(2)))),
            Box::new(Value::BinaryOp(BinaryOperator::Less, Box::new(Value::Integer(3)), Box::new(Value::Integer(4))))))]
    // Precedence: && before ||
    #[case::and_before_or("true || false && false", 
        Value::BinaryOp(BinaryOperator::Or,
            Box::new(Value::Identifier("true".to_string())),
            Box::new(Value::BinaryOp(BinaryOperator::And, 
                Box::new(Value::Identifier("false".to_string())), 
                Box::new(Value::Identifier("false".to_string()))))))]
    fn test_operator_precedence(#[case] input: &str, #[case] expected: Value) {
        let result = parse(input);
        assert_eq!(result, expected);
    }

    #[rstest::rstest]
    // Parentheses override precedence
    #[case::parens_override_mult_add("(1 + 2) * 3", 
        Value::BinaryOp(BinaryOperator::Multiply, 
            Box::new(Value::BinaryOp(BinaryOperator::Add, 
                Box::new(Value::Integer(1)), 
                Box::new(Value::Integer(2)))),
            Box::new(Value::Integer(3))))]
    #[case::parens_override_or("(a || b) && c", 
        Value::BinaryOp(BinaryOperator::And,
            Box::new(Value::BinaryOp(BinaryOperator::Or, 
                Box::new(Value::Identifier("a".to_string())), 
                Box::new(Value::Identifier("b".to_string())))),
            Box::new(Value::Identifier("c".to_string()))))]
    // Nested parentheses
    #[case::nested_parens("((1 + 2) * 3) - 4", 
        Value::BinaryOp(BinaryOperator::Subtract,
            Box::new(Value::BinaryOp(BinaryOperator::Multiply, 
                Box::new(Value::BinaryOp(BinaryOperator::Add, 
                    Box::new(Value::Integer(1)), 
                    Box::new(Value::Integer(2)))),
                Box::new(Value::Integer(3)))),
            Box::new(Value::Integer(4))))]
    fn test_parentheses(#[case] input: &str, #[case] expected: Value) {
        let result = parse(input);
        assert_eq!(result, expected);
    }

    #[rstest::rstest]
    // Associativity: left-associative
    #[case::left_assoc_subtract("1 - 2 - 3", 
        Value::BinaryOp(BinaryOperator::Subtract,
            Box::new(Value::BinaryOp(BinaryOperator::Subtract, 
                Box::new(Value::Integer(1)), 
                Box::new(Value::Integer(2)))),
            Box::new(Value::Integer(3))))]
    #[case::left_assoc_divide("20 / 4 / 2", 
        Value::BinaryOp(BinaryOperator::Divide,
            Box::new(Value::BinaryOp(BinaryOperator::Divide, 
                Box::new(Value::Integer(20)), 
                Box::new(Value::Integer(4)))),
            Box::new(Value::Integer(2))))]
    fn test_associativity(#[case] input: &str, #[case] expected: Value) {
        let result = parse(input);
        assert_eq!(result, expected);
    }

    #[rstest::rstest]
    // Chained unary operators
    #[case::double_negation("!!a", 
        Value::UnaryOp(UnaryOperator::Not, 
            Box::new(Value::UnaryOp(UnaryOperator::Not, 
                Box::new(Value::Identifier("a".to_string()))))))]
    #[case::double_negation_value("-(-5)", 
        Value::UnaryOp(UnaryOperator::Negate, 
            Box::new(Value::UnaryOp(UnaryOperator::Negate, 
                Box::new(Value::Integer(5))))))]
    #[case::triple_negate("---5", 
        Value::UnaryOp(UnaryOperator::Negate,
            Box::new(Value::UnaryOp(UnaryOperator::Negate,
                Box::new(Value::UnaryOp(UnaryOperator::Negate,
                    Box::new(Value::Integer(5))))))))]
    fn test_chained_unary(#[case] input: &str, #[case] expected: Value) {
        let result = parse(input);
        assert_eq!(result, expected);
    }

    #[rstest::rstest]
    // Mixed operators with complex values
    #[case::operators_with_accessor("user.age > 18", 
        Value::BinaryOp(BinaryOperator::Greater,
            Box::new(Value::Accessor(
                Box::new(Value::Identifier("user".to_string())),
                Box::new(Value::Identifier("age".to_string())))),
            Box::new(Value::Integer(18))))]
    #[case::operators_with_function_call("getValue() + 5", 
        Value::BinaryOp(BinaryOperator::Add,
            Box::new(Value::FunctionCall("getValue".to_string(), vec![])),
            Box::new(Value::Integer(5))))]
    #[case::complex_logic("a.b > 5 && c() == \"x\"",
        Value::BinaryOp(BinaryOperator::And,
            Box::new(Value::BinaryOp(BinaryOperator::Greater,
                Box::new(Value::Accessor(
                    Box::new(Value::Identifier("a".to_string())),
                    Box::new(Value::Identifier("b".to_string())))),
                Box::new(Value::Integer(5)))),
            Box::new(Value::BinaryOp(BinaryOperator::Equal,
                Box::new(Value::FunctionCall("c".to_string(), vec![])),
                Box::new(Value::String("x".to_string()))))))]
    fn test_mixed_operators_with_values(#[case] input: &str, #[case] expected: Value) {
        let result = parse(input);
        assert_eq!(result, expected);
    }

    #[rstest::rstest]
    // Operators with strings and floats
    #[case::float_add("1.5 + 2.5", 
        Value::BinaryOp(BinaryOperator::Add,
            Box::new(Value::Float(1.5)),
            Box::new(Value::Float(2.5))))]
    #[case::string_compare(r#""hello" == "hello""#, 
        Value::BinaryOp(BinaryOperator::Equal,
            Box::new(Value::String("hello".to_string())),
            Box::new(Value::String("hello".to_string()))))]
    #[case::mixed_numeric("1 + 2.5", 
        Value::BinaryOp(BinaryOperator::Add,
            Box::new(Value::Integer(1)),
            Box::new(Value::Float(2.5))))]
    fn test_operators_with_various_types(#[case] input: &str, #[case] expected: Value) {
        let result = parse(input);
        assert_eq!(result, expected);
    }
}

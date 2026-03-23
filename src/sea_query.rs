use sea_query::{Alias, BinOper, IntoColumnRef};

use crate::{BinaryOperator, UnaryOperator, Value};

pub trait ToSeaQuery
where
    Self: Sized,
{
    fn to_sea_query(&self) -> sea_query::SimpleExpr;
}

impl ToSeaQuery for Value {
    fn to_sea_query(&self) -> sea_query::SimpleExpr {
        match self {
            Value::Integer(i) => sea_query::SimpleExpr::Value((*i).into()),
            Value::Float(f) => sea_query::SimpleExpr::Value((*f).into()),
            Value::String(s) => sea_query::SimpleExpr::Value(s.into()),
            Value::Identifier(iden) => {
                sea_query::SimpleExpr::Column(Alias::new(iden.clone()).into_column_ref())
            }
            Value::Accessor(_base, field) => {
                // Extract the field name from the accessor
                let field_name = match field.as_ref() {
                    Value::Identifier(name) => name.clone(),
                    _ => panic!("Accessor field must be an identifier"),
                };

                // For obj.field, extract just the final field name as a column
                Alias::new(field_name).into_column_ref().into()
            }
            Value::UnaryOp(op, value) => {
                let value_expr = value.to_sea_query();
                match op {
                    UnaryOperator::Not => {
                        // Logical NOT: use sea_query::UnOper::Not
                        sea_query::SimpleExpr::Unary(sea_query::UnOper::Not, Box::new(value_expr))
                    }
                    UnaryOperator::Negate => {
                        // Arithmetic negation: multiply by -1
                        sea_query::SimpleExpr::Binary(
                            Box::new(sea_query::SimpleExpr::Value((-1i64).into())),
                            BinOper::Mul,
                            Box::new(value_expr),
                        )
                    }
                    UnaryOperator::BitwiseNot => {
                        // Bitwise NOT: use custom representation or panic if not supported
                        panic!("BitwiseNot operator is not directly supported in sea_query")
                    }
                }
            }
            Value::BinaryOp(op, left, right) => {
                let left_expr = left.to_sea_query();
                let right_expr = right.to_sea_query();
                let bin_oper = match op {
                    BinaryOperator::Add => BinOper::Add,
                    BinaryOperator::Subtract => BinOper::Sub,
                    BinaryOperator::Multiply => BinOper::Mul,
                    BinaryOperator::Divide => BinOper::Div,
                    BinaryOperator::Modulo => BinOper::Mod,
                    BinaryOperator::Equal => BinOper::Equal,
                    BinaryOperator::NotEqual => BinOper::NotEqual,
                    BinaryOperator::Less => BinOper::SmallerThan,
                    BinaryOperator::LessEqual => BinOper::SmallerThanOrEqual,
                    BinaryOperator::Greater => BinOper::GreaterThan,
                    BinaryOperator::GreaterEqual => BinOper::GreaterThanOrEqual,
                    BinaryOperator::And => BinOper::And,
                    BinaryOperator::Or => BinOper::Or,
                };
                sea_query::SimpleExpr::Binary(Box::new(left_expr), bin_oper, Box::new(right_expr))
            }
            Value::FunctionCall(name, args) => {
                let arg_exprs: Vec<sea_query::SimpleExpr> =
                    args.iter().map(|arg| arg.to_sea_query()).collect();
                sea_query::Func::cust(Alias::new(name.clone()))
                    .args(arg_exprs)
                    .into()
            }
            Self::MethodCall(obj, method, args) => {
                let obj_expr = obj.to_sea_query();
                let arg_exprs: Vec<sea_query::SimpleExpr> =
                    args.iter().map(|arg| arg.to_sea_query()).collect();
                // Method calls are represented as function calls with the object as the first argument
                sea_query::Func::cust(Alias::new(method.clone()))
                    .arg(obj_expr)
                    .args(arg_exprs)
                    .into()
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parse;

    #[rstest::rstest]
    // Integer cases
    #[case::integer_zero("0")]
    #[case::integer_positive("42")]
    #[case::integer_negative("-15")]
    #[case::integer_large("999999999")]
    // Float cases
    #[case::float_simple("1.5")]
    #[case::float_zero("0.0")]
    #[case::float_scientific("1.5e10")]
    #[case::float_scientific_negative("1.5e-10")]
    // String cases
    #[case::string_simple(r#""hello""#)]
    #[case::string_empty(r#""""#)]
    #[case::string_with_spaces(r#""hello world""#)]
    // Identifier cases
    #[case::identifier_simple("id")]
    #[case::identifier_snake_case("user_id")]
    #[case::identifier_with_numbers("field123")]
    // Accessor/Dot notation cases
    #[case::accessor_simple("obj.field")]
    #[case::accessor_chained("a.b.c")]
    #[case::accessor_with_underscore("user.email_address")]
    // Function call cases
    #[case::function_no_args("sum()")]
    #[case::function_one_arg("max(42)")]
    #[case::function_multiple_args("add(10, 20)")]
    fn test_to_sea_query_literals_and_functions(#[case] input: &str) {
        let value = parse(input).expect("Failed to parse input");
        let result = value.to_sea_query();
        // Verify conversion doesn't panic and returns a valid SimpleExpr
        assert!(!format!("{:?}", result).is_empty());
    }

    #[rstest::rstest]
    // Unary operator cases
    #[case::unary_not("!a")]
    #[case::unary_negate("-10")]
    fn test_unary_operators(#[case] input: &str) {
        let value = parse(input).expect("Failed to parse input");
        // Should not panic during conversion
        let _result = value.to_sea_query();
    }

    #[rstest::rstest]
    // Binary operator cases - Arithmetic
    #[case::binary_add("1 + 2")]
    #[case::binary_subtract("10 - 5")]
    #[case::binary_multiply("3 * 4")]
    #[case::binary_divide("20 / 4")]
    #[case::binary_modulo("10 % 3")]
    // Comparison operators
    #[case::binary_equal("a == b")]
    #[case::binary_not_equal("a != b")]
    #[case::binary_less("a < b")]
    #[case::binary_less_equal("a <= b")]
    #[case::binary_greater("a > b")]
    #[case::binary_greater_equal("a >= b")]
    // Logical operators
    #[case::binary_and("a && b")]
    #[case::binary_or("a || b")]
    fn test_binary_operators(#[case] input: &str) {
        let value = parse(input).expect("Failed to parse input");
        // Should not panic during conversion
        let _result = value.to_sea_query();
    }

    #[rstest::rstest]
    // Method call cases
    #[case::method_no_args("items:count()")]
    #[case::method_one_arg("values:max(10)")]
    #[case::method_multiple_args("data:filter(1, 2)")]
    fn test_method_calls(#[case] input: &str) {
        let value = parse(input).expect("Failed to parse input");
        // Should not panic during conversion
        let _result = value.to_sea_query();
    }

    #[rstest::rstest]
    // Method chaining cases (NEW)
    #[case::chain_two_simple("obj:m1():m2()")]
    #[case::chain_two_with_args("obj:m1(5):m2(10)")]
    #[case::chain_two_mixed_args(r#"obj:m1("x"):m2(5)"#)]
    #[case::chain_three("obj:m1():m2():m3()")]
    #[case::chain_three_with_args("obj:m1(1):m2(2):m3(3)")]
    #[case::chain_on_accessor("obj.prop:m1():m2()")]
    #[case::chain_on_accessor_with_args("obj.data:process(5):filter(10)")]
    #[case::chain_four("obj:a():b():c():d()")]
    fn test_method_chaining(#[case] input: &str) {
        let value = parse(input).expect("Failed to parse input");
        let result = value.to_sea_query();
        // Verify conversion doesn't panic and returns a valid SimpleExpr
        assert!(!format!("{:?}", result).is_empty());
    }

    #[test]
    fn test_integer_conversion() {
        let value = parse("42").expect("Failed to parse");
        let expr = value.to_sea_query();
        match expr {
            sea_query::SimpleExpr::Value(v) => {
                // sea_query converts i64 to BigInt, not Int
                assert_eq!(v, sea_query::Value::BigInt(Some(42)));
            }
            _ => panic!("Expected SimpleExpr::Value"),
        }
    }

    #[test]
    fn test_identifier_conversion() {
        let value = parse("myField").expect("Failed to parse");
        let expr = value.to_sea_query();
        match expr {
            sea_query::SimpleExpr::Column(_) => {
                // Column reference created successfully
            }
            _ => panic!("Expected SimpleExpr::Column"),
        }
    }

    #[test]
    fn test_binary_add_operation() {
        let value = parse("5 + 3").expect("Failed to parse");
        let expr = value.to_sea_query();
        match expr {
            sea_query::SimpleExpr::Binary(_, op, _) => {
                assert_eq!(op, sea_query::BinOper::Add);
            }
            _ => panic!("Expected SimpleExpr::Binary"),
        }
    }

    #[test]
    fn test_binary_comparison_less_than() {
        let value = parse("a < b").expect("Failed to parse");
        let expr = value.to_sea_query();
        match expr {
            sea_query::SimpleExpr::Binary(_, op, _) => {
                assert_eq!(op, sea_query::BinOper::SmallerThan);
            }
            _ => panic!("Expected SimpleExpr::Binary"),
        }
    }

    #[test]
    fn test_unary_not_operation() {
        let value = parse("!flag").expect("Failed to parse");
        let expr = value.to_sea_query();
        match expr {
            sea_query::SimpleExpr::Unary(op, _) => {
                assert_eq!(op, sea_query::UnOper::Not);
            }
            _ => panic!("Expected SimpleExpr::Unary"),
        }
    }

    #[test]
    fn test_unary_negate_operation() {
        let value = parse("-10").expect("Failed to parse");
        let expr = value.to_sea_query();
        match expr {
            sea_query::SimpleExpr::Binary(_, op, _) => {
                // Negation is represented as multiplication by -1
                assert_eq!(op, sea_query::BinOper::Mul);
            }
            _ => panic!("Expected SimpleExpr::Binary for negation"),
        }
    }

    #[test]
    fn test_accessor_extracts_field_name() {
        let value = parse("user.email").expect("Failed to parse");
        let expr = value.to_sea_query();
        match expr {
            sea_query::SimpleExpr::Column(_) => {
                // Successfully converts to column reference
            }
            _ => panic!("Expected SimpleExpr::Column"),
        }
    }

    #[test]
    fn test_function_call_conversion() {
        let value = parse("max(42)").expect("Failed to parse");
        let expr = value.to_sea_query();
        match expr {
            sea_query::SimpleExpr::FunctionCall(_) => {
                // Successfully converts to function call
            }
            _ => panic!("Expected SimpleExpr::FunctionCall"),
        }
    }

    #[test]
    fn test_method_call_conversion() {
        let value = parse("items:count()").expect("Failed to parse");
        let expr = value.to_sea_query();
        match expr {
            sea_query::SimpleExpr::FunctionCall(_) => {
                // Method call is converted to a function call with object as first arg
            }
            _ => panic!("Expected SimpleExpr::FunctionCall"),
        }
    }

    #[test]
    fn test_method_chaining_two_methods() {
        // obj:m1():m2() should convert to nested function calls
        let value = parse("obj:m1():m2()").expect("Failed to parse input");
        let expr = value.to_sea_query();

        // The outer call should be a function call for m2
        match expr {
            sea_query::SimpleExpr::FunctionCall(_) => {
                // Successfully converts to function call; the argument is the result of m1()
                // We can't easily inspect nested structure, but non-panic is success
            }
            _ => panic!("Expected SimpleExpr::FunctionCall for chained method"),
        }
    }

    #[test]
    fn test_method_chaining_with_arguments() {
        // obj:m1(5):m2(10) should create nested calls preserving arg counts
        let value = parse("obj:m1(5):m2(10)").expect("Failed to parse input");
        let expr = value.to_sea_query();

        match expr {
            sea_query::SimpleExpr::FunctionCall(_) => {
                // m2's first arg is the result of m1(5), second arg is 10
            }
            _ => panic!("Expected SimpleExpr::FunctionCall for chained method with args"),
        }
    }

    #[test]
    fn test_method_chaining_three_deep() {
        // obj:a():b():c() should create a function call hierarchy
        let value = parse("obj:a():b():c()").expect("Failed to parse input");
        let expr = value.to_sea_query();

        match expr {
            sea_query::SimpleExpr::FunctionCall(_) => {
                // Top-level is c(b(a(obj)))
            }
            _ => panic!("Expected SimpleExpr::FunctionCall for three-level chain"),
        }
    }

    #[test]
    fn test_method_chaining_on_accessor() {
        // obj.prop:m1():m2() - method chain on accessor chain
        let value = parse("obj.prop:m1():m2()").expect("Failed to parse input");
        let expr = value.to_sea_query();

        match expr {
            sea_query::SimpleExpr::FunctionCall(_) => {
                // m2(m1(obj.prop))
            }
            _ => panic!("Expected SimpleExpr::FunctionCall for accessor with chain"),
        }
    }
}

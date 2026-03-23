# r-expr

[![GitHub](https://img.shields.io/badge/github-r--expr-blue?logo=github)](https://github.com/karlrobeck/r-expr)
[![License: MIT](https://img.shields.io/badge/license-MIT-yellow.svg)](LICENSE)

A lightweight Rust parser library that parses string expressions into an extensible enum representation. Designed for building SQL query builders, custom DSLs, expression evaluators, and configuration systems.

## Overview

`r-expr` makes it easy to parse user-defined or configuration-driven expressions into an AST (Abstract Syntax Tree) representation. Instead of writing a parser from scratch, you get a battle-tested grammar that supports:

- **Accessor chains** — Navigate nested data with dot and bracket notation
- **Function calls** — Direct invocation with arguments
- **Method calls** — Chain operations on accessors
- **Literals** — Integers, floats, and strings

The resulting `Value` enum can be traversed, transformed, or compiled to any target system—SQL, custom functions, configuration, or domain-specific languages.

Key features:
- **Operators** — Arithmetic, comparison, logical, and unary operations with proper precedence
- **Parentheses** — Explicit grouping to override precedence
- **Accessor chains** — Navigate nested data with dot and bracket notation
- **Function & method calls** — Direct and chained invocation
- **Literals** — Integers, floats, and strings

## Quick Start

### Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
r-expr = { git = "https://github.com/karlrobeck/r-expr" }
```

### Optional Features

The library supports optional integrations with feature flags:

**`sea-query`** — Compile expressions to [sea-query](https://docs.rs/sea-query/) `SimpleExpr` for database queries:

```toml
[dependencies]
r-expr = { git = "https://github.com/karlrobeck/r-expr", features = ["sea-query"] }
```

**`serde`** — Serialize/deserialize `Value` with serde:

```toml
[dependencies]
r-expr = { git = "https://github.com/karlrobeck/r-expr", features = ["serde"] }
```

Multiple features can be combined:

```toml
[dependencies]
r-expr = { git = "https://github.com/karlrobeck/r-expr", features = ["sea-query", "serde"] }
```

### Basic Example

```rust
use r_expr::{parse, Value};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Parse an expression
    let expr = parse("user.profile.email")?;
    
    println!("{:?}", expr);
    // Output: Accessor(
    //   Accessor(
    //     Identifier("user"),
    //     Identifier("profile")
    //   ),
    //   Identifier("email")
    // )
    
    Ok(())
}
```

## Syntax Reference

### Accessor Chains

Navigate nested structures with dot notation or bracket notation:

```rust
user                    // Identifier
user.profile            // Dot accessor
user.profile.email      // Chained dot accessors
user["profile"]         // Bracket accessor
user["profile"]["email"] // Chained bracket accessors
user.profile["email"]   // Mix dot and bracket accessors
user.age + 5            // With arithmetic operator
user.status == "active" // With comparison operator
```

Parse example:

```rust
let expr = parse("user.profile.email")?;
// Value::Accessor(
//   Value::Accessor(
//     Value::Identifier("user"),
//     Value::Identifier("profile")
//   ),
//   Value::Identifier("email")
// )

let expr = parse("user.age > 18")?;
// Value::BinaryOp(
//   Greater,
//   Value::Accessor(
//     Value::Identifier("user"),
//     Value::Identifier("age")
//   ),
//   Value::Integer(18)
// )
```

### Function Calls

Invoke functions with literal arguments:

```rust
sum(10, 20)
concat("hello", "world")
max(42.5, 100, -5)
floor(3.14)
```

Parse example:

```rust
let expr = parse("sum(10, 20)")?;
// Value::FunctionCall(
//   "sum".to_string(),
//   vec![
//     Value::Integer(10),
//     Value::Integer(20)
//   ]
// )
```

### Method Calls

Chain methods on accessor expressions:

```rust
user.profile:transform()
data["items"]:count()
collection["data"]["key"]:upper("prefix", "suffix")
```

Parse example:

```rust
let expr = parse("user.profile:transform()")?;
// Value::MethodCall(
//   Value::Accessor(
//     Value::Identifier("user"),
//     Value::Identifier("profile")
//   ),
//   "transform".to_string(),
//   vec![] // no arguments
// )
```

### Operators

Full support for arithmetic, comparison, logical, and unary operators with standard math precedence:

**Arithmetic Operators** (Left-associative)
```rust
1 + 2                   // Addition
5 - 3                   // Subtraction
3 * 4                   // Multiplication
10 / 2                  // Division
10 % 3                  // Modulo
```

**Comparison Operators**
```rust
1 == 1                  // Equal
1 != 2                  // Not equal
1 < 2                   // Less than
1 <= 2                  // Less or equal
2 > 1                   // Greater than
2 >= 1                  // Greater or equal
```

**Logical Operators** (Left-associative)
```rust
true && false           // Logical AND
true || false           // Logical OR
!flag                   // Logical NOT
```

**Unary Operators** (Right-associative)
```rust
-5                      // Negation
!flag                   // Logical NOT
~bits                   // Bitwise NOT
```

**Operator Precedence** (highest to lowest)
| Level | Operator | Associativity |
|-------|----------|---------------|
| 1 | Unary: `!`, `-`, `~` | Right |
| 2 | Multiplicative: `*`, `/`, `%` | Left |
| 3 | Additive: `+`, `-` | Left |
| 4 | Comparison: `==`, `!=`, `<`, `<=`, `>`, `>=` | Left |
| 5 | Logical AND: `&&` | Left |
| 6 | Logical OR: `\|\|` | Left |

**Parentheses for Grouping**
```rust
(1 + 2) * 3             // → 9 (not 7)
(a || b) && c           // Group logical operations
((x > 5) && (y < 10))   // Nested grouping
```

### Literals

Supported literal types:

```rust
42                      // Integer
3.14                    // Float
-2.5e-3                 // Scientific notation
"hello world"           // String (double-quoted)
"escaped \"quotes\""    // Escaped characters
```

Parse examples:

```rust
parse("42")?;           // Value::Integer(42)
parse("3.14")?;         // Value::Float(3.14)
parse("\"text\"")?;     // Value::String("text".to_string())

// With operators
parse("1 + 2 * 3")?;    // Value::BinaryOp(
                        //   Add,
                        //   Integer(1),
                        //   BinaryOp(Multiply, Integer(2), Integer(3))
                        // )
```

## API Documentation

### `parse(input: &str) -> Result<Value, pest::error::Error<Rule>>`

Parses a single expression string into a `Value` AST.

**Parameters:**
- `input` — The expression string to parse

**Returns:**
- `Ok(Value)` — The parsed expression as an AST
- `Err(pest::error::Error)` — Parse error with location and details

**Example:**

```rust
use r_expr::parse;

let result = parse("user.name")?;
match result {
    Value::Accessor(parent, field) => {
        println!("Accessing field on: {:?}", parent);
    },
    _ => {}
}
```

### `Value` Enum

The core AST representation:

```rust
pub enum Value {
    /// Integer literal (e.g., 42, -100)
    Integer(i64),
    
    /// Floating-point literal (e.g., 3.14, -2.5e-3)
    Float(f64),
    
    /// String literal (e.g., "hello")
    String(String),
    
    /// Variable identifier (e.g., user, profile)
    Identifier(String),
    
    /// Accessor chain: object.field or object[index]
    /// First param: parent value, Second param: field/key
    Accessor(Box<Value>, Box<Value>),
    
    /// Function call with arguments (e.g., sum(1, 2, 3))
    /// First param: function name, Second param: arguments
    FunctionCall(String, Vec<Value>),
    
    /// Method call on an accessor (e.g., data:transform())
    /// First param: object, Second param: method name, Third param: arguments
    MethodCall(Box<Value>, String, Vec<Value>),
    
    /// Binary operation (e.g., 1 + 2, a && b)
    /// First param: operator, Second param: left operand, Third param: right operand
    BinaryOp(BinaryOperator, Box<Value>, Box<Value>),
    
    /// Unary operation (e.g., !flag, -5)
    /// First param: operator, Second param: operand
    UnaryOp(UnaryOperator, Box<Value>),
}

pub enum BinaryOperator {
    // Arithmetic
    Add, Subtract, Multiply, Divide, Modulo,
    // Comparison
    Equal, NotEqual, Less, LessEqual, Greater, GreaterEqual,
    // Logical
    And, Or,
}

pub enum UnaryOperator {
    Not, Negate, BitwiseNot,
}
```

## Example: Sea-Query Integration (Feature: `sea-query`)

The library provides a `ToSeaQuery` trait to compile `r-expr` expressions into [sea-query](https://docs.rs/sea-query/) `SimpleExpr` for database-agnostic query building. Enable the `sea-query` feature to use this integration:

```toml
[dependencies]
r-expr = { git = "https://github.com/karlrobeck/r-expr", features = ["sea-query"] }
sea-query = "0.32"
```

**Example Usage:**

```rust
use r_expr::{parse, ToSeaQuery};
use sea_query::Query;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Parse an expression and convert to sea-query
    let expr = parse("user.age > 18 && status == \"active\"")?;
    let sea_expr = expr.to_sea_query();
    
    // Use in a SQL query
    let (sql, values) = Query::select()
        .from(...)
        .and_where(sea_expr)
        .build(...);  // Specify backend (MySQL, Postgres, Sqlite)
    
    println!("SQL: {}", sql);
    Ok(())
}
```

**Supported Value Conversions:**

| r-expr Value | sea-query SimpleExpr |
|--------------|---------------------|
| `Integer(42)` | `Value(42i64)` |
| `Float(3.14)` | `Value(3.14f64)` |
| `String("text")` | `Value("text")` |
| `Identifier("field")` | `Column(field)` |
| `Accessor(obj, field)` | `Column(field)` — extracts final field name |
| `FunctionCall("max", args)` | `FunctionCall` with converted args |
| `MethodCall(obj, method, args)` | `FunctionCall` with object as first arg |
| `BinaryOp(+, /, ==, etc.)` | `Binary(left, BinOper, right)` with proper operator mapping |
| `UnaryOp(!, -)` | `Unary(UnOper, expr)` — logical NOT and arithmetic negation |

**Example: Building a WHERE Clause**

```rust
use r_expr::{parse, ToSeaQuery};
use sea_query::{Query, SqliteQueryBuilder};

let expr = parse("price > 100 && category == \"electronics\"")?;
let filter = expr.to_sea_query();

let (sql, _) = Query::select()
    .from(Table::Products)
    .and_where(filter)
    .build(SqliteQueryBuilder);

println!("{}", sql);
// Output: SELECT * FROM products WHERE price > 100 AND category = 'electronics'
```

**Binary Operators Supported:**
- Arithmetic: `+`, `-`, `*`, `/`, `%`
- Comparison: `==`, `!=`, `<`, `<=`, `>`, `>=`
- Logical: `&&`, `||`

**Unary Operators Supported:**
- Logical NOT: `!`
- Negation: `-`  (represented as multiplication by -1)

**Note:** Bitwise NOT (`~`) is not directly supported in sea-query 0.32.7 and will panic if encountered.

With sea-query, you can build complex, database-independent queries in Rust with full operator support, automatic SQL generation, and type safety.

## Use Cases

### 1. SQL Query Builders

Build dynamic SQL queries with filters, comparisons, and computed columns:

```
revenue:sum()                    → SELECT SUM(revenue)
orders["status"]:count()         → SELECT COUNT(orders['status'])
user.age > 18 && status == "active" → WHERE (user.age > 18) AND (status = 'active')
(price * 1.1) > 100              → WHERE (price * 1.1) > 100
-discount as discount            → Unary negation for columns
profit = revenue - expenses      → Computed columns
```

### 2. Filtering & Query Languages

Complex filter expressions in REST APIs or data pipelines:

```
items["quantity"] > 0 && price < 100
tags:includes("featured") || rating >= 4.5
!archived && status == "published"
```

### 3. Expression Evaluators

Parse and evaluate mathematical or logical expressions:

```
price.value * 1.2                → Scale derived fields
(discount + tax) / 100           → Apply transformations
!feature.enabled                 → Logical negation
items:length() > 5 && total >= 50 → Complex conditions
```

### 4. Custom DSLs for Rules Engines

Build business rules with comparison and logical operators:

```
user.credit_score >= 700 && income > 50000 → Loan eligibility
order.total * 0.1 as shipping_fee          → Dynamic calculations
age >= 18 && (citizenship == "USA" || visa == "valid") → Complex logic
```

### 5. Configuration & Metrics

Parse configuration expressions with operators:

```
cache.ttl = 3600                 → Cache configuration
alerts:trigger(error_count > 10) → Conditional alerts
metrics.cpu > 80 || memory > 90  → System monitoring
```

## Performance Considerations

- **Parsing Speed**: `r-expr` uses the `pest` parser generator, which produces efficient parsing code. Single-pass parsing with minimal memory allocation.
- **AST Memory**: The `Value` enum uses `Box` for recursive variants to keep the stack footprint small.
- **No Optimization**: The library produces a straightforward AST without optimization passes. Post-parse transformations should be applied as needed.
- **Argument Limitations**: Function and method arguments accept only literals (integers, floats, strings) to keep the grammar simple and fast. For complex nested expressions, decompose into multiple parse calls.

**Benchmark Characteristics** (typical on modern hardware):
- Simple identifiers: < 1 μs
- Chain of 5 accessors: ~ 5 μs
- Function call with 3 args: ~ 8 μs
- Deeply nested expressions: linear in expression size

## Operator Examples

### Arithmetic Operations

```rust
use r_expr::{parse, Value, BinaryOperator};

let expr = parse("1 + 2 * 3")?;  // Respects precedence: 1 + (2 * 3)
match expr {
    Value::BinaryOp(BinaryOperator::Add, left, right) => {
        // left = Integer(1)
        // right = BinaryOp(Multiply, Integer(2), Integer(3))
    },
    _ => {}
}
```

### Logical Conditions

```rust
let expr = parse("age >= 18 && (status == \"active\" || admin)")?;
// Builds nested AND/OR structure with proper grouping
```

### Complex Filters

```rust
let expr = parse("user.role == \"admin\" && !archived && score > 100")?;
// Multi-condition filter suitable for database WHERE clauses
```

### Computed Fields

```rust
let expr = parse("(price * quantity) - discount")?;
// Arithmetic with accessors for calculations
```

## Contributing

We welcome contributions! Areas of interest:

- **Performance Optimizations** — Profile and optimize bottlenecks, add benchmarks
- **Documentation** — Expand examples, add integration guides for other systems
- **Testing** — Edge case coverage, fuzz testing
- **Backend Compilers** — Add example compilers for other target systems (GraphQL, JSON Path, etc.)

**Getting Started:**

1. Clone: `git clone https://github.com/karlrobeck/r-expr.git`
2. Test: `cargo test`
3. Create a branch: `git checkout -b feature/your-feature`
4. Make changes and add tests
5. Submit a pull request

## Comparison with Alternatives

| Feature | r-expr | pest (raw) | regex | tree-sitter |
|---------|--------|----------|-------|-------------|
| Easy to use | ✓ | ✗ (manual) | ✗ | ✗ |
| Structured AST | ✓ | ✗ | ✗ | ✓ |
| Lightweight | ✓ | ✓ | ✓ | ✗ |
| Customizable grammar | ✗ | ✓ | ~ | ✓ |
| Method chaining | ✓ | ~ | ✗ | ~ |
| Zero-copy | ~ | ✓ | ✓ | ✓ |

**Why r-expr?** If you need a battle-tested, opinionated expression parser that covers common data access patterns (accessors, function calls, method chaining) with minimal boilerplate, `r-expr` is your answer. It's purpose-built for SQL builders, DSLs, and configuration systems.

## License

Licensed under the MIT License. See [LICENSE](LICENSE) file for details.

---

**Questions or Issues?** Please open an issue on [GitHub](https://github.com/karlrobeck/r-expr) or check the [discussions](https://github.com/karlrobeck/r-expr/discussions).

**Version**: 0.1.0 (Beta)

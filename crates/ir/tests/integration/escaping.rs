//! A record built and read inside one function, which `docs/specs/codegen.md` keeps in locals.

use std::collections::HashMap;

use hegel::TestCase;
use hegel::generators as gs;
use lumen_ir::{Arithmetic, Body, Instruction};

use crate::common;

/// A module declaring `held`, which is the function every example here is about, and `Point`.
fn held(body: &str) -> Body {
    let source = format!(
        "fn held(count: Int) -> Int {{\n{body}}}\n\ntype Point = {{\n    across: Int\n    down: Int\n}}\n"
    );
    declaring(&source)
}

/// The body of `held`, in a module written out in full because it declares more than `held`.
fn declaring(source: &str) -> Body {
    common::body_of(&common::lowered(source), "held").clone()
}

/// Whether the body builds an instance of anything at all.
fn builds(body: &Body) -> bool {
    body.instructions
        .iter()
        .any(|instruction| matches!(instruction, Instruction::New(_)))
}

/// Whether the body reads the field `name` off an instance.
fn reads(body: &Body, name: &str) -> bool {
    body.instructions.iter().any(|instruction| {
        matches!(instruction, Instruction::GetField(reference) if reference.name == name)
    })
}

#[test]
fn a_record_built_and_read_in_one_function_is_never_built() {
    let lowered = held("    point := Point { across: count, down: count }\n    point.across\n");

    assert!(!builds(&lowered), "{:?}", lowered.instructions);
}

#[test]
fn each_field_of_a_record_kept_in_locals_is_read_from_a_local() {
    let lowered =
        held("    point := Point { across: count, down: count }\n    point.across + point.down\n");

    assert!(!reads(&lowered, "across"));
    assert!(!reads(&lowered, "down"));
}

#[test]
fn a_record_built_once_a_turn_and_only_read_is_never_built() {
    let lowered = held(
        "    var total = 0\n    var left = count\n    for left > 0 {\n        \
         point := Point { across: left, down: left }\n        \
         total += point.across + point.down\n        left += -1\n    }\n    total\n",
    );

    assert!(!builds(&lowered), "{:?}", lowered.instructions);
}

#[test]
fn a_record_passed_to_a_call_is_built() {
    let source = "fn held(count: Int) -> Int {\n    \
                  point := Point { across: count, down: count }\n    across_of(point)\n}\n\n\
                  fn across_of(point: Point) -> Int {\n    point.across\n}\n\n\
                  type Point = {\n    across: Int\n    down: Int\n}\n";

    let lowered = declaring(source);

    assert!(builds(&lowered));
}

#[test]
fn a_record_given_back_is_built() {
    let source = "fn held(count: Int) -> Point {\n    \
                  point := Point { across: count, down: count }\n    point\n}\n\n\
                  type Point = {\n    across: Int\n    down: Int\n}\n";

    let lowered = declaring(source);

    assert!(builds(&lowered));
}

#[test]
fn a_record_bound_to_a_second_name_is_built() {
    let lowered = held(
        "    point := Point { across: count, down: count }\n    \
         other := point\n    other.across\n",
    );

    assert!(builds(&lowered));
}

#[test]
fn a_record_updated_is_built() {
    let lowered = held(
        "    point := Point { across: count, down: count }\n    \
         wider := point { across: 1 }\n    wider.across\n",
    );

    assert!(builds(&lowered));
}

#[test]
fn a_record_a_var_holds_is_built() {
    let lowered = held("    var point = Point { across: count, down: count }\n    point.across\n");

    assert!(builds(&lowered));
}

#[test]
fn one_mention_that_lets_the_value_go_builds_it_however_the_others_read_it() {
    let source = "fn held(count: Int) -> Int {\n    \
                  point := Point { across: count, down: count }\n    \
                  _ = point.across\n    across_of(point)\n}\n\n\
                  fn across_of(point: Point) -> Int {\n    point.across\n}\n\n\
                  type Point = {\n    across: Int\n    down: Int\n}\n";

    let lowered = declaring(source);

    assert!(builds(&lowered));
}

#[test]
fn the_fields_of_a_record_kept_in_locals_are_worked_out_in_the_order_the_type_declares_them() {
    let lowered = held("    point := Point { down: 2, across: 1 }\n    point.across\n");

    let written: Vec<i64> = lowered
        .instructions
        .iter()
        .filter_map(|instruction| match instruction {
            Instruction::Long(value) => Some(*value),
            _ => None,
        })
        .collect();

    assert_eq!(written, vec![1, 2]);
}

#[test]
fn a_variant_of_an_algebraic_data_type_is_built_however_little_it_is_let_go_of() {
    let source = "fn held(count: Int) -> Int {\n    \
                  asked := Authorized { identifier: count }\n    count\n}\n\n\
                  type Authorization =\n    | Authorized { identifier: Int }\n    | Denied\n";

    let lowered = declaring(source);

    assert!(builds(&lowered), "{:?}", lowered.instructions);
}

/// A value a generated body works with, which is a whole number or a record holding them.
///
/// There is no interpreter in the compiler and there is none needed here: these are the handful of
/// instructions the generated bodies below are made of, run so that what a record kept in locals
/// computes can be set beside what the same record built on the heap computes, with no JDK in it.
#[derive(Clone, Debug, PartialEq, Eq)]
enum Value {
    Number(i64),
    /// A record, its fields in the order the class declares them.
    Instance(Vec<i64>),
    /// What `new` leaves, which a constructor has not taken yet.
    Fresh,
}

impl Value {
    fn number(&self) -> i64 {
        let Self::Number(value) = self else {
            panic!("a generated body works out a whole number, not {self:?}")
        };
        *value
    }
}

/// What the function `held` of `source` gives back when it is handed `count`.
fn computed(source: &str, count: i64) -> i64 {
    let lowered = common::lowered(source);
    let body = common::body_of(&lowered, "held");
    let mut stack: Vec<Value> = Vec::new();
    let mut locals: HashMap<u16, Value> = HashMap::from([(0, Value::Number(count))]);
    for instruction in &body.instructions {
        match instruction {
            Instruction::Long(value) => stack.push(Value::Number(*value)),
            Instruction::Load { slot, .. } => stack.push(locals[slot].clone()),
            Instruction::Store { slot, .. } => {
                let held = stack.pop().expect("a store takes what was left for it");
                locals.insert(*slot, held);
            }
            Instruction::Arithmetic(operator) => worked_out(&mut stack, *operator),
            Instruction::New(_) => stack.push(Value::Fresh),
            Instruction::Copy => stack.push(stack.last().expect("a copy has a value").clone()),
            Instruction::Construct(reference) => {
                let carried = common::holds(common::class_of(&lowered, &reference.class)).len();
                constructed(&mut stack, carried);
            }
            Instruction::GetField(reference) => {
                let names = common::holds(common::class_of(&lowered, &reference.class));
                let at = names
                    .iter()
                    .position(|name| *name == reference.name)
                    .expect("a field is read off the class that declares it");
                read(&mut stack, at);
            }
            Instruction::Return(_) => break,
            other => panic!("a generated body holds no {other:?}"),
        }
    }
    stack
        .pop()
        .expect("the function gives a number back")
        .number()
}

/// Joins the two numbers on the stack the way `operator` joins them.
fn worked_out(stack: &mut Vec<Value>, operator: Arithmetic) {
    let right = stack
        .pop()
        .expect("an operator has a right operand")
        .number();
    let left = stack
        .pop()
        .expect("an operator has a left operand")
        .number();
    let worked = match operator {
        Arithmetic::Add => left + right,
        Arithmetic::Subtract => left - right,
        Arithmetic::Multiply => left * right,
        other => panic!("a generated body works out no {other:?}"),
    };
    stack.push(Value::Number(worked));
}

/// Takes `carried` values and the copy `new` left, and makes the one below them the record.
fn constructed(stack: &mut Vec<Value>, carried: usize) {
    let at = stack.len() - carried;
    let fields: Vec<i64> = stack.split_off(at).iter().map(Value::number).collect();
    assert_eq!(
        stack.pop(),
        Some(Value::Fresh),
        "a constructor takes an instance"
    );
    let held = stack
        .last_mut()
        .expect("the instance the constructor made whole");
    *held = Value::Instance(fields);
}

/// Reads the field at `at` off the record on the stack.
fn read(stack: &mut Vec<Value>, at: usize) {
    let Some(Value::Instance(fields)) = stack.pop() else {
        panic!("a field is read off a record")
    };
    stack.push(Value::Number(fields[at]));
}

/// The fields a generated record declares, of which it declares one or more.
const FIELDS: [&str; 3] = ["across", "down", "along"];

/// What a generated field is given, each written out of the one parameter and whole numbers.
const VALUES: [&str; 5] = ["count", "7", "count * 2", "count + -1", "3 - count"];

/// The most a generated `count` is, kept small so that no sum of the values above overflows.
const LARGEST: i64 = 1000;

/// A record type of one field or more, and the values a literal of it gives them.
struct Record {
    declared: Vec<&'static str>,
    given: Vec<&'static str>,
}

fn a_record(tc: &TestCase) -> Record {
    let how_many: usize = tc.draw(gs::integers().min_value(1).max_value(FIELDS.len()));
    let declared: Vec<&'static str> = FIELDS[..how_many].to_vec();
    let given = declared
        .iter()
        .map(|_| tc.draw(gs::sampled_from(&VALUES)))
        .collect();
    Record { declared, given }
}

/// A module whose `held` builds the record and reads every field of it back.
///
/// `through` is the name the fields are read through: the binding the literal is written for keeps
/// the record in locals, and a second name the binding is bound to is a mention that lets it go.
fn module(record: &Record, through: &str) -> String {
    let literal: Vec<String> = record
        .declared
        .iter()
        .zip(&record.given)
        .map(|(field, value)| format!("{field}: {value}"))
        .collect();
    let read: Vec<String> = record
        .declared
        .iter()
        .map(|field| format!("{through}.{field}"))
        .collect();
    let declared: Vec<String> = record
        .declared
        .iter()
        .map(|field| format!("    {field}: Int\n"))
        .collect();
    let second = if through == "point" {
        String::new()
    } else {
        format!("    {through} := point\n")
    };
    format!(
        "fn held(count: Int) -> Int {{\n    point := Point {{ {} }}\n{second}    {}\n}}\n\n\
         type Point = {{\n{}}}\n",
        literal.join(", "),
        read.join(" + "),
        declared.concat()
    )
}

#[hegel::test]
fn a_record_kept_in_locals_computes_what_the_same_record_built_on_the_heap_computes(tc: TestCase) {
    let record = a_record(&tc);
    let count: i64 = tc.draw(gs::integers().min_value(-LARGEST).max_value(LARGEST));
    let kept = module(&record, "point");
    let whole = module(&record, "other");

    assert!(!builds(&declaring(&kept)), "{kept}");
    assert!(builds(&declaring(&whole)), "{whole}");
    assert_eq!(computed(&kept, count), computed(&whole, count), "{kept}");
}

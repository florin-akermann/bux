//! What a call inside a supplied module runs, which `docs/specs/io.md` states.

use lumen_ir::{Body, ClassName, Descriptor, Guard, Instruction, Label, Lowered, MethodDescriptor};

use crate::common::{body_of, class_of, lowered, method_of, written};

/// A `main` that writes `"text"` out under the name `written`.
fn writing(written: &str) -> Vec<Instruction> {
    let source = format!("import io\n\nfn main() -> () {{\n    io.{written}(\"text\")\n}}\n");
    body_of(&lowered(&source), "main").instructions.clone()
}

/// The stream `java.lang.System.out` names, which writing out stands on.
fn standard_output() -> Instruction {
    Instruction::GetStatic(lumen_ir::FieldRef {
        class: ClassName::new("java/lang/System"),
        name: "out".to_owned(),
        of: Descriptor::reference("java/io/PrintStream"),
    })
}

/// The method of `java.io.PrintStream` that takes one string and gives nothing back.
fn taking_a_string(named: &str) -> Instruction {
    Instruction::InvokeVirtual(lumen_ir::MethodRef {
        class: ClassName::new("java/io/PrintStream"),
        name: named.to_owned(),
        descriptor: MethodDescriptor::new(vec![Descriptor::reference("java/lang/String")], None),
    })
}

#[test]
fn writing_a_line_out_reaches_the_stream_through_the_class_that_holds_it() {
    let instructions = writing("println");

    assert_eq!(instructions.first(), Some(&standard_output()));
}

#[test]
fn what_is_written_out_is_pushed_above_the_stream_that_writes_it() {
    let instructions = writing("println");

    assert_eq!(
        instructions,
        vec![
            standard_output(),
            Instruction::Text("text".to_owned()),
            taking_a_string("println"),
            Instruction::Return(None),
        ]
    );
}

#[test]
fn each_name_the_module_declares_runs_the_method_of_the_stream_it_is_named_for() {
    assert!(writing("print").contains(&taking_a_string("print")));
    assert!(writing("println").contains(&taking_a_string("println")));
}

#[test]
fn writing_out_leaves_nothing_on_the_stack_so_nothing_is_dropped_after_it() {
    let dropped = writing("println")
        .iter()
        .filter(|instruction| matches!(instruction, Instruction::Drop(_)))
        .count();

    assert_eq!(dropped, 0);
}

#[test]
fn a_module_a_program_imports_is_no_class_of_its_own() {
    let importing = "import io\n\nfn main() -> () {\n    io.println(\"text\")\n}\n";
    let quiet = "fn main() -> () {\n}\n";

    assert_eq!(written(&lowered(importing)), written(&lowered(quiet)));
}

/// A module whose `read` gives back the file at `path`, as the classes it becomes.
fn reading(path: &str) -> Lowered {
    let source = format!(
        "import files\n\nfn read(given: String) -> Result<String, String> {{\n    files.read({path})\n}}\n"
    );
    lowered(&source)
}

/// The class the compiler writes the guarded read in.
const READER: &str = "lumen/Files";

/// The body of the read itself, which is a method of the compiler's own class.
fn read_body(lowered: &Lowered) -> &Body {
    let reader = class_of(lowered, &ClassName::new(READER));
    &method_of(reader, "read").body
}

/// The one span of `body` whose failure is caught.
fn guarded(body: &Body) -> Guard {
    let [one] = body.guards.as_slice() else {
        panic!("a read guards one span")
    };
    one.clone()
}

/// Where `label` is written in `body`, counted in instructions.
fn at(body: &Body, label: Label) -> usize {
    body.instructions
        .iter()
        .position(|instruction| instruction == &Instruction::Label(label))
        .unwrap_or_else(|| panic!("{label:?} is written in the body"))
}

#[test]
fn a_read_is_a_call_of_the_class_the_compiler_writes_the_guarded_read_in() {
    let written = reading("given");

    let calls: Vec<String> = body_of(&written, "read")
        .instructions
        .iter()
        .filter_map(crate::common::called)
        .map(|called| format!("{}.{}", called.class, called.name))
        .collect();
    assert_eq!(calls, vec!["lumen/Files.read".to_owned()]);
}

#[test]
fn the_guarded_read_is_written_only_with_a_module_that_reads_a_file() {
    let quiet = lowered("fn answer() -> Int {\n    7\n}\n");

    assert!(written(&reading("given")).contains(&READER));
    assert!(!written(&quiet).contains(&READER));
}

#[test]
fn a_read_is_a_method_of_its_own_so_that_a_guard_begins_with_an_empty_stack() {
    let nested = lowered(
        "import files\n\nfn read() -> Option<Result<String, String>> {\n    Some(files.read(\"a.txt\"))\n}\n",
    );

    assert!(body_of(&nested, "read").guards.is_empty());
    assert_eq!(read_body(&nested).guards.len(), 1);
}

#[test]
fn everything_the_read_can_throw_is_caught_because_every_failure_is_one_err() {
    let written = reading("given");

    assert_eq!(
        guarded(read_body(&written)).catching,
        ClassName::new("java/lang/Throwable")
    );
}

#[test]
fn the_path_is_asked_of_a_file_before_the_file_is_read_whole() {
    let written = reading("given");
    let body = read_body(&written);
    let read = guarded(body);
    let inside = &body.instructions[at(body, read.from)..at(body, read.to)];

    let names: Vec<String> = inside
        .iter()
        .filter_map(crate::common::called)
        .map(|called| format!("{}.{}", called.class, called.name))
        .collect();
    assert_eq!(
        names,
        vec![
            "java/io/File.<init>".to_owned(),
            "java/io/File.toPath".to_owned(),
            "java/nio/file/Files.readString".to_owned(),
        ]
    );
}

#[test]
fn the_handler_asks_the_throwable_what_it_says_of_itself_and_nothing_else_about_it() {
    let written = reading("given");
    let body = read_body(&written);
    let read = guarded(body);

    let caught = &body.instructions[at(body, read.handler)..];
    let asked: Vec<&str> = caught
        .iter()
        .filter_map(crate::common::called)
        .filter(|called| called.class == ClassName::new("java/lang/Throwable"))
        .map(|called| called.name.as_str())
        .collect();
    assert_eq!(asked, vec!["toString"]);
}

#[test]
fn both_answers_a_read_can_have_are_built_and_both_are_a_result() {
    let written = reading("given");

    let built: Vec<String> = read_body(&written)
        .instructions
        .iter()
        .filter_map(|instruction| match instruction {
            Instruction::New(class) => Some(class.written().to_owned()),
            _ => None,
        })
        .collect();
    assert!(built.contains(&"lumen/Result$Ok".to_owned()));
    assert!(built.contains(&"lumen/Result$Err".to_owned()));
}

#[test]
fn the_path_a_read_is_given_is_any_expression_that_gives_back_text() {
    let joined = reading("given + \".txt\"");

    assert!(
        body_of(&joined, "read")
            .instructions
            .contains(&Instruction::Concat)
    );
    assert_eq!(read_body(&joined).guards.len(), 1, "the read is unchanged");
}

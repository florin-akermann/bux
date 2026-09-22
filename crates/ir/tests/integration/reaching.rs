//! What an `extern` declaration becomes, which `docs/specs/interop.md` states.
//!
//! Each one is a static method of the module that declares it, holding the member it names and
//! the mapping the declaration asks for. `docs/specs/io.md` states what `io`, `files`, and
//! `process` reach through theirs, and the last of these hold those library modules to it.

use lumen_ir::{Asked, Body, ClassName, Comparison, Descriptor, FieldRef, Guard, Instruction};
use lumen_ir::{Label, Lowered, MethodDescriptor, MethodRef};
use lumen_types::Imported;

use crate::common::{LibraryModule, Written, body_of, called, class_of, library};
use crate::common::{lowered_as, method_of};

/// A module declaring the two ways of writing a line out, over the types those name.
const WRITING: &str = concat!(
    "fn printed(text: String) -> () {\n    put(out(), text)\n}\n\n",
    "extern method put(stream: PrintStream, text: String) -> () = \"print\"\n\n",
    "extern field out() -> PrintStream = \"java.lang.System.out\"\n\n",
    "extern type PrintStream = \"java.io.PrintStream\"\n",
);

/// A module declaring a constructor and the static member that takes what it builds.
const BUILDING: &str = concat!(
    "extern static read_whole(path: Path) -> String = \"java.nio.file.Files.readString\"\n\n",
    "extern new named(path: String) -> File\n\n",
    "extern type File = \"java.io.File\"\n\n",
    "extern type Path = \"java.nio.file.Path\"\n",
);

/// A module whose one declaration gives back a `Result`, which is the guarded shape.
const GUARDED: &str = concat!(
    "extern method as_a_path(file: File) -> Result<Path, String> = \"toPath\"\n\n",
    "extern type File = \"java.io.File\"\n\n",
    "extern type Path = \"java.nio.file.Path\"\n",
);

/// A module whose declarations say the members they name give back an `int`.
const WIDENED: &str = concat!(
    "extern method int length(text: String) -> Int = \"length\"\n\n",
    "extern method int counted(text: String) -> Result<Int, String> = \"hashCode\"\n",
);

/// A module whose declarations narrow an argument, which is the range-reading shape.
///
/// The first gives back a number, which is never `null`, and the second gives back a reference,
/// which may be. `docs/specs/interop.md` states that the two reasons for a `None` compose.
const NARROWED: &str = concat!(
    "extern method char at(text: String, int index: Int) -> Option<Int> = \"charAt\"\n\n",
    "extern method cut(text: String, int from: Int) -> Option<String> = \"substring\"\n",
);

/// A module whose one declaration gives back an `Option`, which is the null-reading shape.
const OPTIONAL: &str = concat!(
    "extern method as_a_path(file: File) -> Option<Path> = \"toPath\"\n\n",
    "extern type File = \"java.io.File\"\n\n",
    "extern type Path = \"java.nio.file.Path\"\n",
);

#[test]
fn a_field_is_read_off_the_class_it_names_and_handed_straight_back() {
    let stream = Descriptor::reference("java/io/PrintStream");

    assert_eq!(
        body_of(&lowered(WRITING), "out").instructions,
        vec![
            Instruction::Label(Label(0)),
            Instruction::GetStatic(FieldRef {
                class: ClassName::new("java/lang/System"),
                name: "out".to_owned(),
                of: stream.clone(),
            }),
            Instruction::Label(Label(1)),
            Instruction::Return(Some(stream)),
        ]
    );
}

#[test]
fn a_method_loads_the_receiver_it_is_given_and_calls_the_member_on_it() {
    let stream = Descriptor::reference("java/io/PrintStream");
    let text = Descriptor::reference("java/lang/String");

    assert_eq!(
        body_of(&lowered(WRITING), "put").instructions,
        vec![
            Instruction::Label(Label(0)),
            Instruction::Load {
                slot: 0,
                of: stream,
            },
            Instruction::Load {
                slot: 1,
                of: text.clone(),
            },
            Instruction::InvokeVirtual(MethodRef {
                class: ClassName::new("java/io/PrintStream"),
                name: "print".to_owned(),
                descriptor: MethodDescriptor::new(vec![text], None),
            }),
            Instruction::Label(Label(1)),
            Instruction::Return(None),
        ]
    );
}

#[test]
fn a_constructor_builds_the_class_the_declaration_gives_back_and_hands_it_on() {
    let file = ClassName::new("java/io/File");

    assert_eq!(
        body_of(&lowered(BUILDING), "named").instructions,
        vec![
            Instruction::Label(Label(0)),
            Instruction::New(file.clone()),
            Instruction::Copy,
            Instruction::Load {
                slot: 0,
                of: Descriptor::reference("java/lang/String"),
            },
            Instruction::Construct(MethodRef {
                class: file.clone(),
                name: "<init>".to_owned(),
                descriptor: MethodDescriptor::new(
                    vec![Descriptor::reference("java/lang/String")],
                    None,
                ),
            }),
            Instruction::Label(Label(1)),
            Instruction::Return(Some(Descriptor::Reference(file))),
        ]
    );
}

#[test]
fn a_static_member_is_called_with_what_the_declaration_takes_and_nothing_else() {
    let calls: Vec<String> = reached_by(body_of(&lowered(BUILDING), "read_whole"));

    assert_eq!(calls, vec!["java/nio/file/Files.readString".to_owned()]);
}

#[test]
fn a_declaration_that_gives_nothing_back_leaves_nothing_on_the_stack_to_drop() {
    let dropped = body_of(&lowered(WRITING), "put")
        .instructions
        .iter()
        .filter(|instruction| matches!(instruction, Instruction::Drop(_)))
        .count();

    assert_eq!(dropped, 0);
}

#[test]
fn a_result_guards_the_member_and_catches_everything_a_jvm_can_throw() {
    let written = lowered(GUARDED);
    let body = body_of(&written, "as_a_path");

    assert_eq!(
        guarded(body).catching,
        ClassName::new("java/lang/Throwable")
    );
}

#[test]
fn only_the_member_is_guarded_because_the_mapping_around_it_throws_nothing() {
    let written = lowered(GUARDED);
    let body = body_of(&written, "as_a_path");
    let guard = guarded(body);
    let inside = &body.instructions[at(body, guard.from)..at(body, guard.to)];

    assert_eq!(
        reached_in(inside),
        vec!["java/io/File.toPath".to_owned()],
        "a guard covers the member the declaration names and nothing more"
    );
}

#[test]
fn the_handler_asks_the_throwable_what_it_says_of_itself_and_nothing_else_about_it() {
    let written = lowered(GUARDED);
    let body = body_of(&written, "as_a_path");
    let caught = &body.instructions[at(body, guarded(body).handler)..];

    let asked: Vec<&str> = caught
        .iter()
        .filter_map(called)
        .filter(|call| call.class == ClassName::new("java/lang/Throwable"))
        .map(|call| call.name.as_str())
        .collect();
    assert_eq!(asked, vec!["toString"]);
}

#[test]
fn both_answers_a_guarded_declaration_can_have_are_built_and_both_are_a_result() {
    let written = lowered(GUARDED);
    let body = body_of(&written, "as_a_path");
    let (taken, thrown) = body.instructions.split_at(at(body, guarded(body).handler));

    assert_eq!(built_in(taken), vec!["lumen/Result$Ok".to_owned()]);
    assert_eq!(built_in(thrown), vec!["lumen/Result$Err".to_owned()]);
}

#[test]
fn an_option_reads_what_came_back_for_null_and_builds_one_variant_down_each_path() {
    let written = lowered(OPTIONAL);
    let body = body_of(&written, "as_a_path");

    assert!(
        body.instructions
            .iter()
            .any(|instruction| matches!(instruction, Instruction::JumpIfNull(_))),
        "an `Option` is read off whether the member gave back null"
    );
    assert_eq!(
        built_in(&body.instructions),
        vec![
            "lumen/Option$Some".to_owned(),
            "lumen/Option$None".to_owned()
        ]
    );
    assert!(body.guards.is_empty(), "an `Option` catches nothing");
}

#[test]
fn a_declaration_that_wraps_nothing_gives_back_what_the_member_gave_and_guards_none() {
    let written = lowered(BUILDING);
    let body = body_of(&written, "read_whole");

    assert!(body.guards.is_empty());
    assert_eq!(built_in(&body.instructions), Vec::<String>::new());
}

#[test]
fn a_width_reaches_the_member_for_an_int_and_widens_that_to_the_int_it_gives_back() {
    let text = Descriptor::reference("java/lang/String");

    assert_eq!(
        body_of(&lowered(WIDENED), "length").instructions,
        vec![
            Instruction::Label(Label(0)),
            Instruction::Load { slot: 0, of: text },
            Instruction::InvokeVirtual(MethodRef {
                class: ClassName::new("java/lang/String"),
                name: "length".to_owned(),
                descriptor: MethodDescriptor::new(Vec::new(), Some(Descriptor::Integer)),
            }),
            Instruction::Label(Label(1)),
            Instruction::Widen,
            Instruction::Return(Some(Descriptor::Long)),
        ]
    );
}

#[test]
fn a_declaration_writing_no_width_reaches_its_member_for_a_long_and_widens_nothing() {
    let written = lowered(BUILDING);
    let body = body_of(&written, "read_whole");

    assert!(
        !body.instructions.contains(&Instruction::Widen),
        "a member gives back what the signature declares, so nothing widens it"
    );
}

#[test]
fn a_guarded_width_is_widened_outside_the_guard_because_a_widening_throws_nothing() {
    let written = lowered(WIDENED);
    let body = body_of(&written, "counted");
    let guard = guarded(body);
    let inside = &body.instructions[at(body, guard.from)..at(body, guard.to)];

    assert!(
        !inside.contains(&Instruction::Widen),
        "a guard covers the member the declaration names and nothing more"
    );
    assert!(body.instructions.contains(&Instruction::Widen));
}

#[test]
fn a_widened_answer_is_held_in_a_slot_wide_enough_for_the_whole_number_it_became() {
    assert_eq!(
        body_of(&lowered(WIDENED), "counted").locals,
        2,
        "the slot the `Ok` is built around holds a widened whole number"
    );
}

#[test]
fn every_extern_is_a_method_of_the_module_that_declares_it_and_of_no_class_of_its_own() {
    let written = lowered(WRITING);

    for named in ["printed", "put", "out"] {
        method_of(class_of(&written, &ClassName::new("demo")), named);
    }
    let own: Vec<&str> = crate::common::written(&written)
        .into_iter()
        .filter(|class| !class.starts_with("lumen/"))
        .collect();
    assert_eq!(own, vec!["demo"], "a declaration is a method, not a class");
}

#[test]
fn a_guard_is_an_extern_s_own_so_nothing_a_module_writes_around_one_guards_a_span() {
    let nesting = concat!(
        "fn held(file: File) -> Option<Result<Path, String>> {\n    Some(as_a_path(file))\n}\n\n",
        "extern method as_a_path(file: File) -> Result<Path, String> = \"toPath\"\n\n",
        "extern type File = \"java.io.File\"\n\n",
        "extern type Path = \"java.nio.file.Path\"\n",
    );
    let written = lowered(nesting);

    assert!(body_of(&written, "held").guards.is_empty());
    assert_eq!(body_of(&written, "as_a_path").guards.len(), 1);
}

#[test]
fn a_call_into_a_library_module_is_a_static_call_of_that_module_s_class() {
    let source = "import io\n\nfn go(text: String) -> () {\n    io.println(text)\n}\n";
    let written = lowered_as(&Written {
        source,
        named: "demo",
        imported: &library("io").offered(),
        asked: &Asked::default(),
    });

    assert_eq!(
        reached_by(body_of(&written, "go")),
        vec!["io.println".to_owned()]
    );
}

#[test]
fn writing_a_line_out_reaches_the_stream_through_the_class_that_holds_it() {
    let written = library("io").lowered();
    let stream = ClassName::new("java/io/PrintStream");

    assert!(
        body_of_module(&written, library("io"), "out")
            .instructions
            .iter()
            .any(|instruction| matches!(
                instruction,
                Instruction::GetStatic(field) if field.class == ClassName::new("java/lang/System")
            ))
    );
    for named in ["put", "put_line"] {
        let body = body_of_module(&written, library("io"), named);
        assert!(
            body.instructions
                .iter()
                .filter_map(called)
                .any(|call| call.class == stream),
            "io.{named} writes on the stream it is given"
        );
    }
}

#[test]
fn the_path_is_asked_of_a_file_before_the_file_is_read_whole() {
    let written = library("files").lowered();

    let reached: Vec<String> = ["named", "as_a_path", "read_whole"]
        .into_iter()
        .flat_map(|named| reached_by(body_of_module(&written, library("files"), named)))
        .filter(|call| call.starts_with("java/io/") || call.starts_with("java/nio/"))
        .collect();
    assert_eq!(
        reached,
        vec![
            "java/io/File.<init>".to_owned(),
            "java/io/File.toPath".to_owned(),
            "java/nio/file/Files.readString".to_owned(),
        ]
    );
}

/// Every member of `process`, in the order the module declares them.
const REACHED_BY_A_RUN: [&str; 12] = [
    "of_command",
    "reading_from",
    "inherited",
    "started",
    "output_of",
    "errors_of",
    "ended",
    "over",
    "delimited",
    "token",
    "closed",
    "nothing_at_all",
];

/// The JVM classes `docs/specs/io.md` names for `process`, apart from the mapping around them.
const STARTS_A_PROGRAM: [&str; 4] = [
    "java/lang/ProcessBuilder",
    "java/lang/Process",
    "java/util/Scanner",
    "java/io/InputStream",
];

/// The static field a run reads, which is the standard input the program it starts reads.
const READS_WHAT_WE_READ: &str = "java/lang/ProcessBuilder$Redirect";

#[test]
fn a_run_reaches_the_jvm_classes_that_start_a_program_and_read_what_it_wrote() {
    let written = library("process").lowered();

    let reached: Vec<String> = REACHED_BY_A_RUN
        .into_iter()
        .flat_map(|named| reached_by(body_of_module(&written, library("process"), named)))
        .filter(|call| STARTS_A_PROGRAM.iter().any(|class| call.starts_with(class)))
        .collect();

    assert_eq!(
        reached,
        vec![
            "java/lang/ProcessBuilder.<init>".to_owned(),
            "java/lang/ProcessBuilder.redirectInput".to_owned(),
            "java/lang/ProcessBuilder.start".to_owned(),
            "java/lang/Process.getInputStream".to_owned(),
            "java/lang/Process.getErrorStream".to_owned(),
            "java/lang/Process.waitFor".to_owned(),
            "java/util/Scanner.<init>".to_owned(),
            "java/util/Scanner.useDelimiter".to_owned(),
            "java/util/Scanner.next".to_owned(),
            "java/util/Scanner.close".to_owned(),
            "java/io/InputStream.nullInputStream".to_owned(),
        ]
    );
}

#[test]
fn the_command_a_run_is_given_crosses_as_the_java_util_list_a_jvm_holds_it_as() {
    let written = library("process").lowered();
    let list = Descriptor::reference("java/util/List");

    let built = method_of(
        class_of(&written, &library("process").class()),
        "of_command",
    );

    assert_eq!(built.descriptor.parameters, [list]);
}

#[test]
fn the_only_method_of_a_library_module_that_guards_a_span_gives_back_a_result() {
    assert!(guarding("io").is_empty(), "nothing `io` reaches throws");
    assert_eq!(
        guarding("files"),
        ["read_whole", "as_a_path"],
        "the two declarations `files` writes with a `Result` are the two guarded spans"
    );
    assert_eq!(
        guarding("process"),
        ["started", "ended", "token"],
        "the three declarations `process` writes with a `Result` are its guarded spans"
    );
}

#[test]
fn a_run_points_the_standard_input_of_the_program_at_the_one_this_program_reads() {
    let written = library("process").lowered();

    let read = body_of_module(&written, library("process"), "inherited");

    assert!(
        read.instructions.iter().any(|instruction| matches!(
            instruction,
            Instruction::GetStatic(field) if field.class == ClassName::new(READS_WHAT_WE_READ)
        )),
        "`inherited` reads the redirect off the class that holds it"
    );
}

/// Every method of the library module `named` that guards a span, in the order it writes them.
fn guarding(named: &'static str) -> Vec<String> {
    let written = library(named).lowered();
    class_of(&written, &library(named).class())
        .methods
        .iter()
        .filter(|method| !method.body.guards.is_empty())
        .map(|method| method.name.clone())
        .collect()
}

/// The classes `source` becomes as a module named `demo`, reaching nothing it does not declare.
fn lowered(source: &str) -> Lowered {
    lowered_as(&Written {
        source,
        named: "demo",
        imported: &Imported::default(),
        asked: &Asked::default(),
    })
}

/// The body of the method `name` of the class `module` is.
fn body_of_module<'a>(lowered: &'a Lowered, module: LibraryModule, name: &str) -> &'a Body {
    &method_of(class_of(lowered, &module.class()), name).body
}

/// The one span of `body` whose failure is caught.
fn guarded(body: &Body) -> Guard {
    let [one] = body.guards.as_slice() else {
        panic!("a guarded declaration guards one span")
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

/// Every method `body` calls, each written as the class and the name it reaches.
fn reached_by(body: &Body) -> Vec<String> {
    reached_in(&body.instructions)
}

/// Every method `instructions` call, each written as the class and the name it reaches.
fn reached_in(instructions: &[Instruction]) -> Vec<String> {
    instructions
        .iter()
        .filter_map(called)
        .map(|call| format!("{}.{}", call.class, call.name))
        .collect()
}

/// Every class `instructions` build, in the order they are built.
fn built_in(instructions: &[Instruction]) -> Vec<String> {
    instructions
        .iter()
        .filter_map(|instruction| match instruction {
            Instruction::New(class) => Some(class.written().to_owned()),
            _ => None,
        })
        .collect()
}

#[test]
fn a_narrowed_argument_is_read_against_the_int_range_before_the_member_is_reached() {
    let written = lowered(NARROWED);
    let body = body_of(&written, "at");
    let opened = at(body, Label(0));

    assert_eq!(
        &body.instructions[..opened],
        [
            read_against(i64::from(i32::MIN), Comparison::GreaterOrEqual),
            read_against(i64::from(i32::MAX), Comparison::LessOrEqual),
        ]
        .concat()
    );
}

#[test]
fn a_narrowed_argument_reaches_the_member_as_an_int_and_a_char_widens_to_the_int_declared() {
    let written = lowered(NARROWED);
    let body = body_of(&written, "at");

    assert!(
        body.instructions
            .contains(&Instruction::InvokeVirtual(MethodRef {
                class: ClassName::new("java/lang/String"),
                name: "charAt".to_owned(),
                descriptor: MethodDescriptor::new(
                    vec![Descriptor::Integer],
                    Some(Descriptor::Character)
                ),
            }))
    );
    assert!(body.instructions.contains(&Instruction::Narrow));
    assert!(body.instructions.contains(&Instruction::Widen));
}

#[test]
fn an_argument_outside_the_range_is_a_none_that_reaches_the_member_not_at_all() {
    let written = lowered(NARROWED);
    let body = body_of(&written, "at");
    let unfit = at(body, Label(4));

    assert_eq!(
        built_in(&body.instructions[unfit..]),
        vec!["lumen/Option$None".to_owned()]
    );
    assert_eq!(
        reached_in(&body.instructions[unfit..]),
        vec!["lumen/Option$None.<init>".to_owned()],
        "the `None` is built, and the member the declaration names is not reached"
    );
}

#[test]
fn a_number_the_member_gave_back_is_a_some_because_a_number_is_never_null() {
    let written = lowered(NARROWED);
    let body = body_of(&written, "at");
    let opened = at(body, Label(0));

    assert!(
        !body.instructions[opened..]
            .iter()
            .any(|instruction| matches!(instruction, Instruction::JumpIfNull(_))),
        "a number is never `null`, so the one `None` there is is the unfit argument's"
    );
}

#[test]
fn a_reference_the_member_gave_back_reads_a_null_as_well_as_the_range() {
    let written = lowered(NARROWED);
    let body = body_of(&written, "cut");

    assert!(
        body.instructions
            .iter()
            .any(|instruction| matches!(instruction, Instruction::JumpIfNull(_))),
        "the two reasons for a `None` compose into the one answer"
    );
    assert_eq!(
        built_in(&body.instructions),
        vec![
            "lumen/Option$Some".to_owned(),
            "lumen/Option$None".to_owned(),
            "lumen/Option$None".to_owned(),
        ]
    );
}

#[test]
fn a_declaration_that_narrows_nothing_reads_no_range_and_narrows_no_argument() {
    let written = lowered(WIDENED);
    let body = body_of(&written, "length");

    assert!(!body.instructions.contains(&Instruction::Narrow));
    assert_eq!(at(body, Label(0)), 0, "nothing stands before the member");
}

/// The whole number in the second slot read against `bound`, which is what `fitting` writes.
fn read_against(bound: i64, how: Comparison) -> Vec<Instruction> {
    vec![
        Instruction::Load {
            slot: 1,
            of: Descriptor::Long,
        },
        Instruction::Long(bound),
        Instruction::CompareLongs(how),
        Instruction::JumpIfFalse(Label(4)),
    ]
}

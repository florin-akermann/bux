//! A class-file reader, which is how a test reads back what the writer wrote.
//!
//! It reads only what the tests ask about, and it is deliberately not the writer run backwards:
//! a test that shares the writer's mistakes proves nothing.

/// One class file, as much of it as a test asks about.
#[derive(Debug)]
pub struct ClassFile {
    pub magic: u32,
    pub minor: u16,
    pub major: u16,
    pub pool: Vec<Constant>,
    pub access: u16,
    pub this: u16,
    pub extends: u16,
    pub fields: Vec<Member>,
    pub methods: Vec<Member>,
}

/// One constant pool entry, as far as a test looks into it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Constant {
    Utf8(String),
    Integer(i32),
    Long(i64),
    Text(u16),
    Class(u16),
    NameAndType {
        name: u16,
        descriptor: u16,
    },
    Field {
        class: u16,
        member: u16,
    },
    Method {
        class: u16,
        member: u16,
    },
    /// The index a whole number leaves unusable behind it.
    Unusable,
}

/// One field or one method.
#[derive(Debug)]
pub struct Member {
    pub access: u16,
    pub name: String,
    pub descriptor: String,
    pub code: Option<Code>,
}

/// The `Code` attribute of one method.
#[derive(Debug)]
pub struct Code {
    pub max_stack: u16,
    pub max_locals: u16,
    pub instructions: Vec<u8>,
    pub frames: Option<Vec<u8>>,
}

/// Reading the `Code` of a method that must have one.
pub trait TakeCode {
    fn take_code(&self) -> &Code;
}

impl TakeCode for Option<Code> {
    fn take_code(&self) -> &Code {
        self.as_ref().expect("this method has a body")
    }
}

impl ClassFile {
    /// The class the pool names at `at`.
    pub fn class(&self, held: u16) -> &str {
        match &self.pool[held as usize - 1] {
            Constant::Class(written) => self.utf8(*written),
            other => panic!("{held} holds {other:?}, not a class"),
        }
    }

    /// The method of this class written as `name`.
    pub fn method(&self, name: &str) -> &Member {
        self.methods
            .iter()
            .find(|method| method.name == name)
            .unwrap_or_else(|| panic!("this class writes a method named {name}"))
    }

    /// The text the pool holds at `at`, which a test reads a name through.
    pub fn utf8(&self, held: u16) -> &str {
        match &self.pool[held as usize - 1] {
            Constant::Utf8(text) => text,
            other => panic!("{held} holds {other:?}, not text"),
        }
    }
}

/// Reads `bytes` as the class file they are.
pub fn read(bytes: &[u8]) -> ClassFile {
    let mut reading = Reading { bytes, at: 0 };
    let magic = reading.u4();
    let minor = reading.u2();
    let major = reading.u2();
    let pool = reading.pool();
    let access = reading.u2();
    let this = reading.u2();
    let extends = reading.u2();
    let interfaces = reading.u2();
    assert_eq!(interfaces, 0, "version 0.1 writes no interfaces");
    let fields = reading.members(&pool);
    let methods = reading.members(&pool);
    ClassFile {
        magic,
        minor,
        major,
        pool,
        access,
        this,
        extends,
        fields,
        methods,
    }
}

struct Reading<'a> {
    bytes: &'a [u8],
    at: usize,
}

impl Reading<'_> {
    fn pool(&mut self) -> Vec<Constant> {
        let count = self.u2();
        let mut held = Vec::new();
        while held.len() + 1 < count as usize {
            let entry = self.constant();
            let wide = matches!(entry, Constant::Long(_));
            held.push(entry);
            if wide {
                held.push(Constant::Unusable);
            }
        }
        held
    }

    fn constant(&mut self) -> Constant {
        match self.u1() {
            1 => {
                let many = self.u2() as usize;
                let text = self.bytes(many);
                Constant::Utf8(String::from_utf8(text).expect("a name is text"))
            }
            3 => Constant::Integer(self.u4().cast_signed()),
            5 => Constant::Long(self.u8().cast_signed()),
            7 => Constant::Class(self.u2()),
            8 => Constant::Text(self.u2()),
            9 => Constant::Field {
                class: self.u2(),
                member: self.u2(),
            },
            10 => Constant::Method {
                class: self.u2(),
                member: self.u2(),
            },
            12 => Constant::NameAndType {
                name: self.u2(),
                descriptor: self.u2(),
            },
            tag => panic!("version 0.1 writes no constant of tag {tag}"),
        }
    }

    fn members(&mut self, pool: &[Constant]) -> Vec<Member> {
        let count = self.u2();
        (0..count).map(|_| self.member(pool)).collect()
    }

    fn member(&mut self, pool: &[Constant]) -> Member {
        let access = self.u2();
        let name = named(pool, self.u2());
        let descriptor = named(pool, self.u2());
        let attributes = self.u2();
        let mut code = None;
        for _ in 0..attributes {
            let of = named(pool, self.u2());
            let length = self.u4() as usize;
            let held = self.bytes(length);
            if of == "Code" {
                code = Some(read_code(&held, pool));
            }
        }
        Member {
            access,
            name,
            descriptor,
            code,
        }
    }

    fn u1(&mut self) -> u8 {
        self.at += 1;
        self.bytes[self.at - 1]
    }

    fn u2(&mut self) -> u16 {
        self.at += 2;
        u16::from_be_bytes([self.bytes[self.at - 2], self.bytes[self.at - 1]])
    }

    fn u4(&mut self) -> u32 {
        let mut held = [0; 4];
        held.copy_from_slice(&self.bytes[self.at..self.at + 4]);
        self.at += 4;
        u32::from_be_bytes(held)
    }

    fn u8(&mut self) -> u64 {
        let mut held = [0; 8];
        held.copy_from_slice(&self.bytes[self.at..self.at + 8]);
        self.at += 8;
        u64::from_be_bytes(held)
    }

    fn bytes(&mut self, many: usize) -> Vec<u8> {
        self.at += many;
        self.bytes[self.at - many..self.at].to_vec()
    }
}

fn read_code(bytes: &[u8], pool: &[Constant]) -> Code {
    let mut reading = Reading { bytes, at: 0 };
    let max_stack = reading.u2();
    let max_locals = reading.u2();
    let length = reading.u4() as usize;
    let instructions = reading.bytes(length);
    let handlers = reading.u2();
    assert_eq!(handlers, 0, "version 0.1 catches nothing");
    let attributes = reading.u2();
    let mut frames = None;
    for _ in 0..attributes {
        let of = named(pool, reading.u2());
        let held = reading.u4() as usize;
        let written = reading.bytes(held);
        if of == "StackMapTable" {
            frames = Some(written);
        }
    }
    Code {
        max_stack,
        max_locals,
        instructions,
        frames,
    }
}

fn named(pool: &[Constant], held: u16) -> String {
    match &pool[held as usize - 1] {
        Constant::Utf8(text) => text.clone(),
        other => panic!("{held} holds {other:?}, not text"),
    }
}

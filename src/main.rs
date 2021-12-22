use std::{
    collections::HashSet,
    fs::{self, File},
    io::{Read, Write},
    path::Path,
    slice::Iter,
};

use clap::{App, Arg};

#[derive(Debug)]
enum Token {
    Keyword(String),
    Verb(String),
    Whitespace(String),
    Special(String),
}

fn make_norse(word: String) -> String {
    let mut out = "".to_string();
    for ch in word.to_lowercase().chars() {
        out.push(match ch {
            'a' => '\u{16a8}', // ᚨ
            'b' => '\u{16d3}', // ᛒ
            'c' => '\u{16cd}', // -
            'd' => '\u{16d1}', // ᛞ
            'e' => '\u{16c2}', // ᛖ
            'f' => '\u{16a0}', // ᚠ
            'g' => '\u{16b5}', // ᚷ
            'h' => '\u{16bb}', // ᚺ
            'i' => '\u{16c1}', // ᛁ
            'j' => '\u{16c3}', // ᛃ
            'k' => '\u{16b4}', // ᚲ
            'l' => '\u{16da}', // ᛚ
            'm' => '\u{16d7}', // ᛗ
            'n' => '\u{16bf}', // ᚾ
            'o' => '\u{16df}', // ᛟ
            'p' => '\u{16c8}', // ᛈ
            'q' => '\u{16e9}', // -
            'r' => '\u{16b1}', // ᚱ
            's' => '\u{16ca}', // ᛊ
            't' => '\u{16cf}', // ᛏ
            'u' => '\u{16a2}', // ᚢ
            'v' => '\u{16a1}', // -
            'w' => '\u{16b9}', // ᚹ
            'x' => '\u{16ea}', // -
            'y' => '\u{16a3}', // -
            'z' => '\u{16ce}', // ᛉ
            x => x,
        });
    }

    out
}

fn tokenize_program(program: String) -> Vec<Token> {
    let reserved: HashSet<&str> = HashSet::from([
        "class",
        "interface",
        "enum",
        "public",
        "private",
        "protected",
        "abstract",
        "static",
        "this",
        "extends",
        "Override",
        "super",
        "new",
        "import",
        "assert",
        "package",
        "throws",
        "throw",
        "try",
        "catch",
        "if",
        "else",
        "for",
        "while",
        "return",
        "instanceof",
        "final",
        "void",
        "int",
        "long",
        "char",
        "float",
        "double",
        "boolean",
        "true",
        "false",
        "break",
    ]);
    let word_end = HashSet::from([
        '.', ',', '(', ')', '<', '@', '>', '[', ']', '{', '}', '/', '+', '-', '*', '%', '&', '=',
        '?', ':', ';',
    ]);

    let mut tokens = vec![];

    let mut is_whitespace = false;
    let mut current = "".to_string();

    for ch in program.chars() {
        if is_whitespace && matches!(ch, ' ' | '\t' | '\n') {
            current.push(ch);
        } else if is_whitespace {
            tokens.push(Token::Whitespace(current));
            if word_end.contains(&ch) {
                current = "".to_string();
                tokens.push(Token::Special(ch.to_string()));
            } else {
                current = ch.to_string();
            }
            is_whitespace = false;
        } else if matches!(ch, ' ' | '\t' | '\n') {
            if !current.is_empty() {
                if reserved.contains(&current.as_str()) {
                    tokens.push(Token::Keyword(current));
                } else {
                    tokens.push(Token::Verb(current));
                }
            }
            current = ch.to_string();
            is_whitespace = true;
        } else if word_end.contains(&ch) {
            if !current.is_empty() {
                tokens.push(if reserved.contains(&current.as_str()) {
                    Token::Keyword(current)
                } else {
                    Token::Verb(current)
                });
            }
            tokens.push(Token::Special(ch.to_string()));
            current = "".to_string();
        } else {
            current.push(ch);
        }
    }

    if !current.is_empty() {
        tokens.push(
            if matches!(current.chars().nth(0).unwrap(), ' ' | '\t' | '\n') {
                Token::Whitespace(current)
            } else {
                Token::Verb(current)
                // TODO not exhaustive lol
            },
        );
    }

    tokens
}

macro_rules! token_content {
    ($a:expr) => {{
        let tmp = $a;
        match tmp {
            Token::Special(c) => c,
            Token::Verb(c) => c,
            Token::Whitespace(c) => c,
            Token::Keyword(c) => c,
        }
    }};
}

fn skip_till_str(iterator: &mut Iter<Token>, string: String) -> String {
    let mut out = "".to_string();
    while let Some(tok) = iterator.next() {
        if *token_content!(tok) == string {
            out += token_content!(tok);
            break;
        }
    }
    out
}

fn skip_till_end(iterator: &mut Iter<Token>) -> String {
    let mut out = "".to_string();

    let tok = iterator.next().unwrap();
    out += token_content!(tok);
    if out != "." {
        return out;
    }
    let mut was_whitespace = false;
    while let Some(tok) = iterator.next() {
        out += token_content!(tok).as_str();
        match tok {
            Token::Whitespace(_) => {
                was_whitespace = true;
            }
            Token::Verb(_) | Token::Keyword(_) => {
                if was_whitespace {
                    break;
                }
            }
            Token::Special(content) => {
                if content == "." {
                    was_whitespace = false;
                } else {
                    break;
                }
            }
        }
    }
    out
}

fn translate_program(program: String, verb_keep: HashSet<&str>) -> (String, Option<String>) {
    let mut verb_keep = verb_keep.clone();
    let mut new_program = String::new();

    let prog = tokenize_program(program);
    let mut program = prog.iter();
    let mut classname = None;

    while let Some(token) = program.next() {
        match token {
            Token::Whitespace(whitespace) => {
                new_program += whitespace.as_str();
            }
            Token::Keyword(keyword) => {
                new_program += keyword.as_str();
                match keyword.as_str() {
                    "import" => {
                        let mut verb = None;
                        while let Some(tok) = program.next() {
                            match tok {
                                Token::Special(x) => {
                                    new_program += x.as_str();
                                    if x == ";" {
                                        break;
                                    }
                                }
                                Token::Verb(x) => {
                                    verb = Some(x);
                                    new_program += x.as_str();
                                }
                                Token::Whitespace(x) | Token::Keyword(x) => {
                                    new_program += x.as_str()
                                }
                            }
                        }
                        if let Some(v) = verb {
                            verb_keep.insert(v);
                        }
                    }
                    "package" => {
                        new_program += skip_till_str(&mut program, ";".to_string()).as_str();
                    }
                    "class" | "interface" | "enum" if classname.is_none() => {
                        let mut cn = None;
                        while let Some(tok) = program.next() {
                            if token_content!(tok) == "{" && !matches!(tok, Token::Keyword(_)) {
                                new_program += "{";
                                break;
                            }
                            if let Token::Verb(c) = tok {
                                let tmp = make_norse(c.to_string());
                                new_program += tmp.as_str();
                                cn = Some(tmp);
                            } else {
                                new_program += token_content!(tok);
                            }
                        }
                        classname = cn;
                    }
                    _ => {}
                }
            }
            Token::Verb(verb) => {
                if verb_keep.contains(verb.as_str()) {
                    new_program += verb;
                } else {
                    new_program += &make_norse(verb.to_owned());
                }
                let tmp = skip_till_end(&mut program);
                new_program += tmp.as_str();
            }
            Token::Special(special) => {
                new_program += special.as_str();
            }
        }
    }

    (new_program, classname)
}

fn main() {
    let args = App::new("java to runic java")
        .version("0.1.0")
        .author("hyde <hiddy.tiddey@gmail.com>")
        .about("translator from java to runic java (in rust and not in java lol) i think it's pretty cool and useful ofc. BTW i am mixing elder futhark with younger futhark; fight me")
        .arg(
            Arg::with_name("filename")
                .takes_value(true)
                .required(false)
                .value_name("FILE")
                .help("file to translate")
                .index(1),
        )
        .arg(
            Arg::with_name("output")
                .short("o")
                .takes_value(true)
                .value_name("DIRECTORY")
                .help("sets the directory where to place the new java file. If not set, prints to stdout")
                .required(false)
        )
        .arg(
            Arg::with_name("keep")
                .short("k")
                .takes_value(true)
                .value_name("KEEP")
                .help("adds keywords to keep when translating would break stuff (e.g. System)")
                .multiple(true)

        )
        .get_matches();

    let mut buffer = String::new();
    if let Some(filename) = args.value_of("filename") {
        buffer = fs::read_to_string(filename).expect("cant");
    } else {
        std::io::stdin()
            .read_to_string(&mut buffer)
            .unwrap_or_else(|err| panic!("{}", err));
    }
    let mut verb_keep = HashSet::from([
        "String",
        "System",
        "Double",
        "Float",
        "Integer",
        "Boolean",
        "Exception",
        "Math",
    ]);
    let keeps = args.values_of("keep");
    if let Some(keeps) = keeps {
        for keep in keeps {
            verb_keep.insert(keep);
        }
    }
    let program = translate_program(buffer, verb_keep);
    if let Some(dir) = args.value_of("output") {
        let mut f = File::create(Path::new(dir).join(format!("{}.java", program.1.unwrap())))
            .expect("unable to open file");
        f.write_all(program.0.as_bytes()).expect("unable to write");
    } else {
        println!("{}", program.0);
    }
}

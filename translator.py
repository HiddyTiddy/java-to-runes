from dataclasses import dataclass
import sys


dictonary = {
    "a": "\u16a8",
    "b": "\u16d3",
    "c": "\u16cd",
    "d": "\u16d1",
    "e": "\u16c2",
    "f": "\u16a0",
    "g": "\u16b5",
    "h": "\u16bb",
    "i": "\u16c1",
    "j": "\u16c3",
    "k": "\u16b4",
    "l": "\u16da",
    "m": "\u16d7",
    "n": "\u16bf",
    "o": "\u16df",
    "p": "\u16c8",
    "q": "\u16e9",
    "r": "\u16b1",
    "s": "\u16ca",
    "t": "\u16cf",
    "u": "\u16a2",
    "v": "\u16a1",
    "w": "\u16b9",
    "x": "\u16ea",
    "y": "\u16a3",
    "z": "\u16ce",
}


@dataclass
class Token:
    content: str
    is_keyword: bool = False
    is_verb: bool = False
    is_whitespace: bool = False


def make_norse(word):
    out = word.lower()
    for ch in dictonary:
        out = out.replace(ch, dictonary[ch])
    return out


def tokenize_program(program):
    reserved = {
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

    }
    word_end = {
        ".",
        ",",
        '(',
        ")",
        "<",
        "@",
        ">",
        "[",
        "]",
        "{",
        "}",
        "/",
        "+",
        "-",
        "*",
        "%",
        "&",
        "=",
        "?",
        ":",
        ";",
    }
    whitespace = (" ", "\t", "\n")
    tokens = []
    current = ""
    is_whitespace = False

    for ch in program:
        if is_whitespace and ch in whitespace:
            current += ch
        elif is_whitespace:
            tokens.append(Token(content=current, is_whitespace=True))
            if ch in word_end:
                current = ""
                tokens.append(Token(content=ch))
            else:
                current = ch
            is_whitespace = False
        elif ch in whitespace:
            if current in reserved:
                tokens.append(Token(content=current, is_keyword=True))
            else:
                tokens.append(Token(content=current, is_verb=True))
            current = ch
            is_whitespace = True
        elif ch in word_end:
            if current in reserved:
                tokens.append(Token(content=current, is_keyword=True))
            else:
                tokens.append(Token(content=current, is_verb=True))
            tokens.append(Token(content=ch))
            current = ""
        else:
            current += ch
    if current != "":
        if current[0] in whitespace:
            tokens.append(Token(content=current, is_whitespace=True))
        else:
            tokens.append(Token(content=current, is_verb=True))
    return tokens


def skip_till_str(iterator, end):
    out = ""
    tok = next(iterator)
    while tok.content != end:
        out += tok.content
        tok = next(iterator)
    out += tok.content
    return out


def skip_till_end(iterator):
    out = ""
    tok = next(iterator)
    if tok.content != ".":
        return tok.content
    was_whitespace = False
    while True:
        out += tok.content
        if tok.is_whitespace:
            was_whitespace = True
        elif tok.is_verb:
            if was_whitespace:
                return out
        elif tok.is_keyword:
            if was_whitespace:
                return out
        else:
            if tok.content == ".":
                was_whitespace = False
            else:
                return out
        tok = next(iterator)


def read_stdin():
    out = ""
    for line in sys.stdin:
        out += line
    return out


def main(program, verb_keep, write_dir=None):
    new_program = ""
    prog = iter(tokenize_program(program))
    classname = None
    for token in prog:
        if token.is_whitespace:
            new_program += token.content
        elif token.is_verb:
            if token.content in verb_keep:
                new_program += token.content
                new_program += skip_till_end(prog)
            else:
                new_program += make_norse(token.content)
                new_program += skip_till_end(prog)
        elif token.is_keyword:
            new_program += token.content
            if token.content == "import":
                tok = next(prog)
                verb = tok.content
                while tok.content != ";":
                    new_program += tok.content
                    verb = tok.content
                    tok = next(prog)
                verb_keep.add(verb)
                new_program += tok.content
            if token.content == "package":
                new_program += skip_till_str(prog, ";")
            if token.content == "class" and classname is None:
                tok = next(prog)
                cn = None
                while tok.content != "{" and not tok.is_keyword:
                    if tok.is_verb:
                        new_program += make_norse(tok.content)
                        cn = make_norse(tok.content)
                    else:
                        new_program += tok.content

                    tok = next(prog)
                classname = cn
                new_program += tok.content


        else:
            new_program += token.content

    # if classname is not None:
    #     new_program = new_program.replace(classname, newname)
    #     new_program = new_program.replace(make_norse(classname), newname)
    new_program = new_program.replace("language", "ᛚᚨᚿᚵᚢᚨᚵᛂ")

    if write_dir is not None:
        with open(f"{write_dir}/{classname}.java", "w") as f:
            f.write(new_program)
    else:
        print(new_program)


if __name__ == "__main__":
    prog = read_stdin()
    verb_keep = {"String", "System", "Double", "Float", "Integer", "Boolean", "Exception", "Math"}
    # TODO do argument parsing
    # TODO RIIR
    for keep in sys.argv[1:]:
        verb_keep.add(keep)
    main(prog, verb_keep)

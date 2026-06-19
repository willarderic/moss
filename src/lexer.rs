use std::iter;
use std::iter::Peekable;
use std::vec;

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum Token {
    UNKNOWN,
    IDENTIFIER(String),
    NUMBER(u64),
    LPAREN,
    RPAREN,
    LBRACKET,
    RBRACKET,
    LBRACE,
    RBRACE,
    SEMICOLON,
    COLON,
    // operators
    PLUS,
    DASH,
    ASTERISK,
    SLASH,
    ASSIGN,
    BANG,
    TILDE,
    LT,
    GT,
    LEQ,
    GEQ,
    EQ,
    NEQ,
    QUESTION,
    AMPERSAND,
    // Keywords
    FN,
    RETURN,
    VAR,
    FOR,
    IF,
    ELSE,
    TRUE,
    FALSE,
    EOF,
}

impl std::fmt::Display for Token {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::UNKNOWN => write!(f, "(UNKNOWN, )"),
            Self::NUMBER(num) => write!(f, "(NUMBER, {})", num),
            Self::IDENTIFIER(ident) => write!(f, "(IDENTIFIER, {})", ident),
            Self::LPAREN => write!(f, "(LPAREN, ()"),
            Self::RPAREN => write!(f, "(RPAREN, ))"),
            Self::LBRACKET => write!(f, "(LBRACKET, [)"),
            Self::RBRACKET => write!(f, "(RBRACKET, ])"),
            Self::RBRACE => write!(f, "(RBRACE, }})"),
            Self::LBRACE => write!(f, "(LBRACE, {{)"),
            Self::SEMICOLON => write!(f, "(SEMICOLON, ;)"),
            Self::COLON => write!(f, "(COLON, :)"),
            Self::PLUS => write!(f, "(PLUS, +)"),
            Self::DASH => write!(f, "(DASH, -)"),
            Self::ASTERISK => write!(f, "(ASTERISK, *)"),
            Self::SLASH => write!(f, "(SLASH, /)"),
            Self::ASSIGN => write!(f, "(ASSIGN, =)"),
            Self::BANG => write!(f, "(BANG, !)"),
            Self::TILDE => write!(f, "(TILDE, ~)"),
            Self::LT => write!(f, "(LT, <)"),
            Self::GT => write!(f, "(GT, >)"),
            Self::LEQ => write!(f, "(LEQ, <=)"),
            Self::GEQ => write!(f, "(GEQ, >=)"),
            Self::EQ => write!(f, "(EQ, ==)"),
            Self::NEQ => write!(f, "(NEQ, !=)"),
            Self::QUESTION => write!(f, "(QUESTION, ?)"),
            Self::AMPERSAND => write!(f, "(AMPERSAND, &)"),
            Self::FN => write!(f, "(FN, fn)"),
            Self::IF => write!(f, "(IF, if)"),
            Self::RETURN => write!(f, "(RETURN, return)"),
            Self::VAR => write!(f, "(VAR, var)"),
            Self::FOR => write!(f, "(FOR, for)"),
            Self::ELSE => write!(f, "(ELSE, else)"),
            Self::TRUE => write!(f, "(TRUE, true)"),
            Self::FALSE => write!(f, "(FALSE, false)"),
            Self::EOF => write!(f, "(EOF, )"),
        }
    }
}

impl Token {
    pub fn get_literal(&self) -> String {
        match self {
            Self::IDENTIFIER(ident) => ident.to_owned(),
            Self::NUMBER(num) => num.to_string(),
            Self::LPAREN => '('.to_string(),
            Self::RPAREN => ')'.to_string(),
            Self::LBRACKET => '['.to_string(),
            Self::RBRACKET => ']'.to_string(),
            Self::LBRACE => '{'.to_string(),
            Self::RBRACE => '}'.to_string(),
            Self::SEMICOLON => ';'.to_string(),
            Self::COLON => ':'.to_string(),
            Self::PLUS => '+'.to_string(),
            Self::DASH => '-'.to_string(),
            Self::ASTERISK => '*'.to_string(),
            Self::SLASH => '/'.to_string(),
            Self::ASSIGN => '='.to_string(),
            Self::BANG => '!'.to_string(),
            Self::TILDE => '~'.to_string(),
            Self::LT => '<'.to_string(),
            Self::GT => '>'.to_string(),
            Self::LEQ => "<=".to_string(),
            Self::GEQ => ">=".to_string(),
            Self::EQ => "==".to_string(),
            Self::NEQ => "!=".to_string(),
            Self::QUESTION => '?'.to_string(),
            Self::AMPERSAND => '&'.to_string(),
            Self::FN => "fn".to_string(),
            Self::IF => "if".to_string(),
            Self::RETURN => "return".to_string(),
            Self::VAR => "var".to_string(),
            Self::FOR => "for".to_string(),
            Self::ELSE => "else".to_string(),
            Self::TRUE => "true".to_string(),
            Self::FALSE => "false".to_string(),
            _ => '\0'.to_string(),
        }
    }
}

fn token_from(lexeme: &str) -> Token {
    match lexeme {
        "<=" => Token::LEQ,
        ">=" => Token::GEQ,
        "!=" => Token::NEQ,
        "==" => Token::EQ,
        "<" => Token::LT,
        ">" => Token::GT,
        "!" => Token::BANG,
        "~" => Token::TILDE,
        ":" => Token::COLON,
        "+" => Token::PLUS,
        "-" => Token::DASH,
        "/" => Token::SLASH,
        "*" => Token::ASTERISK,
        "{" => Token::LBRACE,
        "}" => Token::RBRACE,
        ")" => Token::RPAREN,
        "(" => Token::LPAREN,
        "[" => Token::LBRACKET,
        "]" => Token::RBRACKET,
        "?" => Token::QUESTION,
        "&" => Token::AMPERSAND,
        ";" => Token::SEMICOLON,
        "=" => Token::ASSIGN,
        _ => Token::UNKNOWN,
    }
}

fn is_punctuator(c: char) -> bool {
    match c {
        '(' => true,
        ')' => true,
        '[' => true,
        ']' => true,
        '{' => true,
        '}' => true,
        '+' => true,
        '-' => true,
        '/' => true,
        '*' => true,
        '=' => true,
        '<' => true,
        '>' => true,
        '?' => true,
        '&' => true,
        '!' => true,
        '~' => true,
        ':' => true,
        ';' => true,
        _ => false,
    }
}

fn is_digit(c: char) -> bool {
    '0' <= c && c <= '9'
}

fn is_alpha(c: char) -> bool {
    ('a' <= c && c <= 'z') || ('A' <= c && c <= 'Z')
}

fn is_alphanumeric(c: char) -> bool {
    is_digit(c) || is_alpha(c) || c == '_'
}

fn punctuator_eq<I: Iterator<Item = char>>(c: char, chars: &mut Peekable<I>) -> Token {
    let peeked: char = match chars.peek() {
        Some(x) => x.clone(),
        None => '\0',
    };

    if peeked == '=' {
        let peeked = chars.next().unwrap();
        let lexeme: String = format!("{c}{peeked}");
        return token_from(lexeme.as_str());
    } else {
        let lexeme: String = c.to_string();
        return token_from(lexeme.as_str());
    }
}

fn handle_punctuator<I: Iterator<Item = char>>(c: char, chars: &mut Peekable<I>) -> Token {
    match c {
        '>' | '<' | '!' | '=' => punctuator_eq(c, chars),
        _ => {
            let lexeme: String = c.to_string();
            token_from(lexeme.as_str())
        }
    }
}

fn handle_number<I: Iterator<Item = char>>(c: char, chars: &mut Peekable<I>) -> Token {
    let mut number: String = String::from(c);
    while is_digit(peek(chars)) {
        let n = chars.next().unwrap();
        if !is_digit(n) {
            break;
        }
        number.push(n);
    }
    Token::NUMBER(number.parse::<u64>().unwrap())
}

fn handle_keyword(word: &str) -> Option<Token> {
    match word {
        "if" => Some(Token::IF),
        "return" => Some(Token::RETURN),
        "fn" => Some(Token::FN),
        "var" => Some(Token::VAR),
        "for" => Some(Token::FOR),
        "else" => Some(Token::ELSE),
        "true" => Some(Token::TRUE),
        "false" => Some(Token::FALSE),
        _ => None,
    }
}

fn peek<I: Iterator<Item = char>>(chars: &mut Peekable<I>) -> char {
    match chars.peek() {
        Some(x) => *x,
        None => '\0',
    }
}

pub fn lex(input: String) -> Vec<Token> {
    let mut tokens = Vec::new();

    let mut chars = input.chars().peekable();
    while let Some(c) = chars.next() {
        if is_punctuator(c) {
            tokens.push(handle_punctuator(c, &mut chars));
            continue;
        }

        if is_digit(c) {
            tokens.push(handle_number(c, &mut chars));
            continue;
        }

        if is_alpha(c) {
            let mut word = String::from(c);
            while is_alphanumeric(peek(&mut chars)) {
                let c = chars.next().unwrap();
                word.push(c);
            }
            tokens.push(match handle_keyword(word.as_str()) {
                Some(tok) => tok,
                None => Token::IDENTIFIER(word),
            })
        }
    }
    tokens.push(Token::EOF);

    tokens
}

//! APEX Parser Module
//!
//! Tokenization and parsing of APEX documents.

pub mod lexer;
pub mod parser;

pub use lexer::{Lexer, Token, ParseMode, ParseFix};
pub use parser::{parse_str, parse_str_with_mode, ParserConfig};

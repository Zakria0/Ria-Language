use std::env;
use std::fs;
use std::process;
use std::process::Command;

#[derive(Debug, Clone, PartialEq)]
pub enum TokenType {
    Return,
    Number,
    Semi,
}

#[derive(Debug, Clone)]
pub struct Token {
    pub token_type: TokenType,
    pub value: Option<String>,
}

pub fn tokenize(input: &str) -> Vec<Token> {
    let mut tokens = Vec::new();
    let mut chars = input.chars().peekable();
    
    while let Some(&c) = chars.peek() {
        match c {
            'a'..='z' | 'A'..='Z' => {
                let mut identifier = String::new();
                while let Some(&ch) = chars.peek() {
                    if ch.is_alphabetic() {
                        identifier.push(chars.next().unwrap());
                    } else {
                        break;
                    }
                }
                
                let token_type = match identifier.as_str() {
                    "kharrej" => TokenType::Return,
                    _ => continue,
                };
                
                tokens.push(Token {
                    token_type,
                    value: None,
                });
            }
            '0'..='9' => {
                let mut number = String::new();
                while let Some(&ch) = chars.peek() {
                    if ch.is_digit(10) {
                        number.push(chars.next().unwrap());
                    } else {
                        break;
                    }
                }
                
                tokens.push(Token {
                    token_type: TokenType::Number,
                    value: Some(number),
                });
            }
            ';' => {
                tokens.push(Token {
                    token_type: TokenType::Semi,
                    value: None,
                });
                chars.next();
            }
            ' ' | '\t' | '\n' | '\r' => {
                chars.next();
            }
            _ => {
                chars.next();
            }
        }
    }
    
    tokens
}

pub fn tokens_to_asm(tokens: Vec<Token>) -> String {
    let mut asm_code = String::new();
    
    asm_code.push_str("global _start\n");
    asm_code.push_str("section .text\n");
    asm_code.push_str("_start:\n");
    
    let mut i = 0;
    while i < tokens.len() {
        if let TokenType::Return = tokens[i].token_type {
            if i + 2 < tokens.len() {
                if let TokenType::Number = tokens[i + 1].token_type {
                    if let TokenType::Semi = tokens[i + 2].token_type {
                        if let Some(value) = &tokens[i + 1].value {
                            if let Ok(exit_code) = value.parse::<i32>() {
                                asm_code.push_str("    mov rax, 60     ; sys_exit\n");
                                asm_code.push_str(&format!("    mov rdi, {}    ; exit code\n", exit_code));
                                asm_code.push_str("    syscall\n");
                                
                                i += 2;
                            }
                        }
                    }
                }
            }
        }
        i += 1;
    }
    
    asm_code
}

fn compile_to_executable(asm_code: &str, output_name: &str) -> Result<(), Box<dyn std::error::Error>> {
    let asm_file = "output.asm";
    let obj_file = "output.o";
    
    fs::write(asm_file, asm_code)?;
    println!("Generated assembly written to {}", asm_file);
    
    let nasm_status = Command::new("nasm")
        .args(&["-f", "elf64", asm_file, "-o", obj_file])
        .status()?;
    
    if !nasm_status.success() {
        return Err("nasm assembly failed".into());
    }
    println!("Assembled to object file: {}", obj_file);
    
    let ld_status = Command::new("ld")
        .args(&[obj_file, "-o", output_name])
        .status()?;
    
    if !ld_status.success() {
        return Err("linking failed".into());
    }
    println!("Linked to executable: {}", output_name);
    
    Ok(())
}

fn main() {
    let args: Vec<String> = env::args().collect();


    if args.len() != 2 {
        eprintln!("Incorrect usage. Correct usage is...");
        eprintln!("zakaria <input.ria>");
        process::exit(1);
    }

    let file_path = &args[1];

    let content = match fs::read_to_string(file_path) {
        Ok(content) => content,
        Err(e) => {
            eprintln!("Error reading file {}: {}", file_path, e);
            process::exit(1);
        }
    };

    println!("File content:\n{}", content);

    let tokens = tokenize(&content);

    for token in &tokens {
        println!("Token: {:?}, Value: {:?}", token.token_type, token.value);
    }

    let asm_code = tokens_to_asm(tokens);
    println!("\nGenerated Assembly:\n{}", asm_code);

    let output_name = "output";
    match compile_to_executable(&asm_code, output_name) {
        Ok(()) => {
            println!("\nCompilation successful! Executable '{}' created.", output_name);
            
            println!("\nRunning the executable:");
            let run_status = Command::new(format!("./{}", output_name))
                .status()
                .expect("Failed to run executable");
            
            println!("Program exited with: {}", run_status);
        }
        Err(e) => {
            eprintln!("Compilation failed: {}", e);
            process::exit(1);
        }
    }
}

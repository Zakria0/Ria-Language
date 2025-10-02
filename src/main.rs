use std::env;
use std::fs;
use std::path::Path;
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

pub fn tokens_to_asm(tokens: Vec<Token>) -> Result<String, String> {
    let mut asm_code = String::new();
    
    asm_code.push_str("global _start\n");
    asm_code.push_str("section .text\n");
    asm_code.push_str("_start:\n");
    
    let mut i = 0;
    let mut found_return = false;
    
    while i < tokens.len() {
        if let TokenType::Return = tokens[i].token_type {
            found_return = true;
            
            if i + 2 >= tokens.len() {
                return Err("Incomplete return statement: expected 'kharrej <number>;'".into());
            }
            
            match (&tokens[i + 1].token_type, &tokens[i + 2].token_type) {
                (TokenType::Number, TokenType::Semi) => {
                    if let Some(value) = &tokens[i + 1].value {
                        match value.parse::<i32>() {
                            Ok(exit_code) if (0..=255).contains(&exit_code) => {
                                asm_code.push_str("    mov rax, 60     ; sys_exit\n");
                                asm_code.push_str(&format!("    mov rdi, {}    ; exit code\n", exit_code));
                                asm_code.push_str("    syscall\n");
                                
                                i += 2;
                            }
                            Ok(exit_code) => {
                                return Err(format!("Exit code must be between 0 and 255, got {}", exit_code));
                            }
                            Err(_) => {
                                return Err(format!("Invalid number: '{}'", value));
                            }
                        }
                    }
                }
                (TokenType::Number, _) => {
                    return Err("Expected semicolon after number".into());
                }
                (_, _) => {
                    return Err("Expected number after 'kharrej'".into());
                }
            }
        }
        i += 1;
    }
    
    if !found_return {
        return Err("No 'kharrej' statement found".into());
    }
    
    Ok(asm_code)
}

fn compile_to_executable(asm_code: &str, output_name: &str) -> Result<(), Box<dyn std::error::Error>> {
    let asm_file = format!("{}.asm", output_name);
    let obj_file = format!("{}.o", output_name);
    
    fs::write(&asm_file, asm_code)?;
    println!("Generated assembly written to {}", asm_file);
    
    println!("Assembling with nasm...");
    let nasm_output = Command::new("nasm")
        .args(&["-f", "elf64", &asm_file, "-o", &obj_file])
        .output()?;
    
    if !nasm_output.status.success() {
        let error_msg = String::from_utf8_lossy(&nasm_output.stderr);
        return Err(format!("nasm assembly failed: {}", error_msg).into());
    }
    println!("Assembled to object file: {}", obj_file);
    
    println!("Linking with ld...");
    let ld_output = Command::new("ld")
        .args(&[&obj_file, "-o", output_name])
        .output()?;
    
    if !ld_output.status.success() {
        let error_msg = String::from_utf8_lossy(&ld_output.stderr);
        return Err(format!("linking failed: {}", error_msg).into());
    }
    println!("Linked to executable: {}", output_name);
    
    // let _ = fs::remove_file(asm_file);
    // let _ = fs::remove_file(obj_file);
    
    Ok(())
}

fn get_output_name(input_path: &str) -> String {
    let path = Path::new(input_path);
    
    if let Some(stem) = path.file_stem() {
        if let Some(stem_str) = stem.to_str() {
            return stem_str.to_string();
        }
    }
    
    "output".to_string()
}

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() != 2 {
        eprintln!("Incorrect usage. Correct usage is:");
        eprintln!("   zakaria <input.ria>");
        process::exit(1);
    }

    let file_path = &args[1];

    if !Path::new(file_path).exists() {
        eprintln!("File not found: {}", file_path);
        process::exit(1);
    }

    if !file_path.ends_with(".ria") {
        eprintln!("Warning: Expected .ria file extension");
    }

    let content = match fs::read_to_string(file_path) {
        Ok(content) => content,
        Err(e) => {
            eprintln!("Error reading file '{}': {}", file_path, e);
            process::exit(1);
        }
    };

    println!("File content:\n{}", content);

    let tokens = tokenize(&content);

    println!("\nTokens found:");
    for (i, token) in tokens.iter().enumerate() {
        println!("  {}. Token: {:?}, Value: {:?}", i + 1, token.token_type, token.value);
    }

    if tokens.is_empty() {
        eprintln!("No tokens found in file");
        process::exit(1);
    }

    let asm_code = match tokens_to_asm(tokens) {
        Ok(asm) => asm,
        Err(e) => {
            eprintln!("Error generating assembly: {}", e);
            process::exit(1);
        }
    };
    
    println!("\nGenerated Assembly:\n{}", asm_code);

    let output_name = get_output_name(file_path);
    println!("\nOutput executable will be: {}", output_name);

    match compile_to_executable(&asm_code, &output_name) {
        Ok(()) => {
            println!("\nCompilation successful! Executable '{}' created.", output_name);
            
            println!("\nRunning the executable...");
            let run_status = Command::new(format!("./{}", output_name))
                .status()
                .expect("Failed to run executable");
            
            println!("Program exited with: {}", run_status);
            
            println!("\nTo check the exit code manually, run:");
            println!("   ./{}", output_name);
            println!("   echo $?");
        }
        Err(e) => {
            eprintln!("   Compilation failed: {}", e);
            eprintln!("   Make sure 'nasm' and 'ld' are installed:");
            eprintln!("   Ubuntu/Debian: sudo apt install nasm");
            eprintln!("   Fedora: sudo dnf install nasm");
            eprintln!("   Arch: sudo pacman -S nasm");
            process::exit(1);
        }
    }
}
//! A set of capabilities provided to the standard VM execution
//! binary, this provides just the basic stuff for outputting

use std::io::{self, Write};

use lepton_vm::{capabilities::CapabilityFn, values::Value, virtual_machine::VirtualMachine};

/// Basic set of capabilities that we give to our VM
/// essentially just a basic print of a value
pub fn all<'a>() -> Vec<CapabilityFn<'a>> {
    let all_caps: Vec<CapabilityFn<'a>> = vec![
        cap_print as CapabilityFn<'a>,
        cap_print_char as CapabilityFn<'a>,
        cap_debug_breakpoint as CapabilityFn<'a>
    ];

    all_caps
}

/// Capability 0: pops a value from the top of the stack and
/// prints it without a newline
fn cap_print<'a>(
    virtual_machine: &mut VirtualMachine<'a>,
) -> Result<(), Box<dyn std::error::Error>> {
    let value = virtual_machine
        .pop_maybe()
        .ok_or("stack underflow in cap_print, no values on stack")?;
    print!("{}", format_value(&value));
    Ok(())
}

fn format_value(value: &Value) -> String {
    match value {
        Value::Unit => "()".to_string(),
        Value::Int(i) => i.to_string(),
        Value::Float(f) => f.to_string(),
        Value::Bool(b) => b.to_string(),
        Value::Tag(t) => format!("<tag:{}>", u64::from(*t)),
        Value::Object(idx) => format!("<object:{idx}>"),
        Value::Array(idx) => format!("<array:{idx}>"),
        Value::UInt(u) => u.to_string(),
    }
}

/// Capability 1: pops an UInt from the stack and prints it
/// as a unicode character.
///
/// # Errors
///
/// Returns an error if the UInt is not a valid unicode codepoint
/// or if the value is not an UInt.
fn cap_print_char<'a>(
    virtual_machine: &mut VirtualMachine<'a>,
) -> Result<(), Box<dyn std::error::Error>> {
    let value = virtual_machine
        .pop_maybe()
        .ok_or("stack underflow in cap_print_char")?;

    match value {
        Value::UInt(i) => {
            // i64 -> u32 -> char, both conversions can fail
            let codepoint = u32::try_from(i)
                .map_err(|_| format!("uint {i} is out of range for a unicode codepoint"))?;

            let ch = char::from_u32(codepoint)
                .ok_or_else(|| format!("uint {i} is not a valid unicode codepoint"))?;

            print!("{ch}");
            Ok(())
        }
        other => Err(format!("cap_print_char expects UInt, got {}", format_value(&other)).into()),
    }
}

/// Capability 2: debug breakpoint
/// prints the stack and pauses until more user input
fn cap_debug_breakpoint<'a>(
    virtual_machine: &mut VirtualMachine<'a>,
) -> Result<(), Box<dyn std::error::Error>> {
    let locals_top = virtual_machine
        .call_stack
        .last()
        .map_or(0, |f| f.locals_base + f.local_count);


    // print out the stack in reverse order
    for elem in virtual_machine.stack.iter().skip(locals_top)  {
        println!("{}", format_value(elem))
    }

    println!("<top>");

    let mut input = String::new();
    print!("continue? (any input) ");

    // wait for user input to continue
    io::stdout().flush().unwrap();
    io::stdin().read_line(&mut input).expect("Failed to read input");

    
    Ok(())
}
//! A set of capabilities provided to the standard VM execution
//! binary, this provides just the basic stuff for outputting

use lepton_vm::{capabilities::CapabilityFn, values::Value, virtual_machine::VirtualMachine};

/// Basic set of capabilities that we give to our VM
/// essentially just a basic print of a value
pub fn all() -> Vec<CapabilityFn> {
    vec![cap_print, cap_print_char]
}

/// Capability 0: pops a value from the top of the stack and
/// prints it without a newline
fn cap_print(virtual_machine: &mut VirtualMachine) -> Result<(), Box<dyn std::error::Error>> {
    let value = virtual_machine
        .stack
        .pop()
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

/// Capability 1: pops an integer from the stack and prints it
/// as a unicode character.
///
/// # Errors
///
/// Returns an error if the integer is not a valid unicode codepoint
/// or if the value is not an integer.
fn cap_print_char(virtual_machine: &mut VirtualMachine) -> Result<(), Box<dyn std::error::Error>> {
    let value = virtual_machine
        .stack
        .pop()
        .ok_or("stack underflow in cap_print_char")?;

    match value {
        Value::Int(i) => {
            // i64 -> u32 -> char, both conversions can fail
            let codepoint = u32::try_from(i)
                .map_err(|_| format!("integer {i} is out of range for a unicode codepoint"))?;

            let ch = char::from_u32(codepoint)
                .ok_or_else(|| format!("integer {i} is not a valid unicode codepoint"))?;

            print!("{ch}");
            Ok(())
        }
        other => Err(format!("cap_print_char expects Int, got {}", format_value(&other)).into()),
    }
}

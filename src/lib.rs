#[cfg(feature = "wasm")]
use wasm_bindgen::prelude::*;
mod tree;
mod roll;
use tree::{ASTInput, AST};

pub fn consume_input_to_output(input: String, help_message: &String, skip_dropped: bool, short_output: bool, colour: bool) -> Result<String, Box<dyn std::error::Error>> {
    let mut ast_input = ASTInput::from_string(input, help_message);
    let mut ast = AST::consume_input(&mut ast_input)?;
    let mut rng = rand::rng();
    let running_total: i64;
    // And then we build the output
    let mut output: String = String::new();
    if short_output {
        running_total = ast.compute(&mut rng)?;
    }
    else {
        running_total = ast.compute_with_output(&mut output, &mut rng, skip_dropped, colour)?;
    }
    // Specify what the result of our rolls were
    output += &(" => ".to_owned() + &running_total.to_string());
    if short_output {
        return Ok(running_total.to_string());
    }
    else {
        return Ok(output);
    }
}

#[cfg_attr(feature = "wasm", wasm_bindgen)]
// No error output in WASM
pub fn consume_input_to_output_without_error(input: String, help_message: String, skip_dropped: bool, short_output: bool) -> String {
    let output = consume_input_to_output(input.to_lowercase(), &help_message, skip_dropped, short_output, false);
    match output {
        Ok(x) => {
            return x;
        }
        Err(y) => {
            return y.to_string();
        }
    }
}

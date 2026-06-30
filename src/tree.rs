use rand::{rngs::ThreadRng};
use crate::roll::*;

macro_rules! handle_operator {
    ($operator:ident, $input:ident, $input_idx:ident) => {
        if let ASTInput::PartialAST(_first) = &$input[$input_idx - 1] && let ASTInput::PartialAST(_second) = &$input[$input_idx + 1] {
            let ASTInput::PartialAST(second) = $input.remove($input_idx + 1) else {unreachable!()};
            let ASTInput::PartialAST(first) = $input.remove($input_idx - 1) else {unreachable!()};
            $input[$input_idx - 1] = ASTInput::PartialAST(AST::new(ASType::$operator(Box::new(first), Box::new(second)), false));
        }
    };
}

#[derive(Clone, PartialEq, Debug)]
pub enum ASTInput {
    Character(char),
    PartialAST(AST)
}

impl ASTInput {
    pub fn from_string(input: String, help_message: &String) -> Result<Vec<ASTInput>, Box<dyn std::error::Error>> {
        let mut result: Vec<ASTInput> = input.chars().filter(|x| !x.is_whitespace()).map(|x| ASTInput::Character(x)).collect();
        ASTInput::from_partial_input(&mut result, help_message)?;
        Ok(result)
    }

    fn from_partial_input(input: &mut Vec<ASTInput>, help_message: &String) -> Result<(), Box<dyn std::error::Error>>{
        let mut result_idx = 0;
        while result_idx < input.len() {
            let result_char = &input[result_idx];
            result_idx += 1;
            match result_char {
                ASTInput::Character(character) => {
                    if *character == '(' {
                        // Handle brackets
                        let mut num_brackets: usize = 1;
                        let mut bracket_idx = result_idx;
                        while num_brackets > 0 {
                            if input[bracket_idx] == ASTInput::Character(')') {
                                num_brackets -= 1;
                            }
                            else if input[bracket_idx] == ASTInput::Character('(') {
                                num_brackets += 1;
                            }
                            bracket_idx += 1;
                            if bracket_idx > input.len() {
                                return Err("Mismatched brackets".into());
                            }
                        }
                        // Remove the brackets
                        input.remove(bracket_idx - 1);
                        input.remove(result_idx - 1);
                        let mut bracketed_input = input.drain((result_idx - 1)..(bracket_idx - 2)).collect();
                        ASTInput::from_partial_input(&mut bracketed_input, help_message)?;
                        let handled_bracket = ASType::consume_input(&mut bracketed_input)?;
                        input.insert(result_idx - 1, ASTInput::PartialAST(AST::new(handled_bracket, true)));
                    }
                    else if character.is_digit(10) || *character == 'd' {
                        let idx_before_roll = result_idx - 1;
                        result_idx -= 1;
                        let roll = consume_input_to_roll(input, &mut result_idx, help_message)?;
                        input[idx_before_roll] = ASTInput::PartialAST(AST::new(ASType::RollOrConstant(roll), false));
                        if idx_before_roll + 1 < input.len() {
                            input.drain((idx_before_roll + 1)..(result_idx - 1));
                        }
                        result_idx = idx_before_roll;
                    }
                }
                ASTInput::PartialAST(_x) => {
                }
            }
        }
        Ok(())
    }
}

// BEDMAS
const OPERATOR_CHARS_ORDERED: &[char] = &['^', '/', '*', '+', '-'];

#[derive(Clone, PartialEq, Debug)]
pub struct AST {
    ast_type: ASType,
    is_bracketed: bool
}

impl AST {
    fn new(ast_type: ASType, is_bracketed: bool) -> Self {
        Self { ast_type, is_bracketed }
    }

    pub fn compute_with_output(&mut self, output: &mut String, rng: &mut ThreadRng, skip_dropped: bool, colour: bool) -> Result<i64, Box<dyn std::error::Error>> {
        if self.is_bracketed {
            *output += "(";
        }
        let result = self.ast_type.compute_with_output(output, rng, skip_dropped, colour)?;
        if self.is_bracketed {
            *output += ")";
        }
        Ok(result)
    }

    pub fn compute(&mut self, rng: &mut ThreadRng) -> Result<i64, Box<dyn std::error::Error>> {
        self.ast_type.compute(rng)
    }

    pub fn consume_input(input: &mut Vec<ASTInput>) -> Result<Self, Box<dyn std::error::Error>> {
        let ast_type = ASType::consume_input(input)?;
        Ok(Self::new(ast_type, false))
    }
}

#[derive(Clone, PartialEq, Debug)]
pub enum ASType {
    Add(Box<AST>, Box<AST>),
    Subtract(Box<AST>, Box<AST>),
    Multiply(Box<AST>, Box<AST>),
    Divide(Box<AST>, Box<AST>),
    Power(Box<AST>, Box<AST>),
    RollOrConstant(RollOrConstant),
    HiddenConstant(i64)
}

impl ASType {
    pub fn compute_with_output(&mut self, output: &mut String, rng: &mut ThreadRng, skip_dropped: bool, colour: bool) -> Result<i64, Box<dyn std::error::Error>> {
        let mut result: i64 = 0;
        match self {
            Self::Add(x, y) => {
                let x_result = x.compute_with_output(output, rng, skip_dropped, colour)?;
                *output += " + ";
                result += x_result + y.compute_with_output(output, rng, skip_dropped, colour)?;
            }
            Self::Subtract(x, y) => {
                let x_result = x.compute_with_output(output, rng, skip_dropped, colour)?;
                *output += " - ";
                result += x_result - y.compute_with_output(output, rng, skip_dropped, colour)?;
            }
            Self::Multiply(x, y) => {
                let x_result = x.compute_with_output(output, rng, skip_dropped, colour)?;
                *output += " * ";
                result += x_result * y.compute_with_output(output, rng, skip_dropped, colour)?;
            }
            Self::Divide(x, y) => {
                let x_result = x.compute_with_output(output, rng, skip_dropped, colour)?;
                *output += " / ";
                result += x_result / y.compute_with_output(output, rng, skip_dropped, colour)?;
            },
            Self::Power(x, y) => {
                let x_result = x.compute_with_output(output, rng, skip_dropped, colour)?;
                *output += " ^ "; result += x_result.pow(y.compute_with_output(output, rng, skip_dropped, colour)?.try_into()?);
            }
            Self::RollOrConstant(x) => {
                result += x.roll_with_output(output, rng, skip_dropped, colour)?;
            }
            Self::HiddenConstant(x) => {
                result += *x;
            }
        }
        Ok(result)
    }

    pub fn compute(&mut self, rng: &mut ThreadRng) -> Result<i64, Box<dyn std::error::Error>> {
        let mut result: i64 = 0;
        match self {
            Self::Add(x, y) => {
                let x_result = x.compute(rng)?;
                result += x_result + y.compute(rng)?;
            }
            Self::Subtract(x, y) => {
                let x_result = x.compute(rng)?;
                result += x_result - y.compute(rng)?;
            }
            Self::Multiply(x, y) => {
                let x_result = x.compute(rng)?;
                result += x_result * y.compute(rng)?;
            }
            Self::Divide(x, y) => {
                let x_result = x.compute(rng)?;
                result += x_result / y.compute(rng)?;
            },
            Self::Power(x, y) => {
                let x_result = x.compute(rng)?;
                result += x_result.pow(y.compute(rng)?.try_into()?);
            }
            Self::RollOrConstant(x) => {
                result += x.roll(rng)?;
            }
            Self::HiddenConstant(x) => {
                result += *x;
            }
        }
        Ok(result)
    }

    pub fn consume_input(input: &mut Vec<ASTInput>) -> Result<Self, Box<dyn std::error::Error>> {
        for operator_char in OPERATOR_CHARS_ORDERED {
            let mut input_idx: usize = 0;
            // While loop since the size of input will change here
            while input_idx < input.len() {
                let input_singlet = &input[input_idx];
                match input_singlet {
                    ASTInput::Character(x) => {
                        if x == operator_char {
                            match x {
                                '^' => handle_operator!(Power, input, input_idx),
                                '/' => handle_operator!(Divide, input, input_idx),
                                '*' => handle_operator!(Multiply, input, input_idx),
                                '+' => handle_operator!(Add, input, input_idx),
                                '-' => handle_operator!(Subtract, input, input_idx),
                                _ => {}
                            }
                        }
                    }
                    ASTInput::PartialAST(_x) => {}
                }
                input_idx += 1;
            }
        }
        if let ASTInput::PartialAST(result) = input.remove(0) {
            return Ok(result.ast_type);
        }
        return Err("IUNNO".into());
    }
}

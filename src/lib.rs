#![feature(box_patterns)]
#![feature(box_syntax)]

pub mod parser;
pub mod program;

#[cfg(test)]
mod tests {
    use program::{*, node::Node};
    use parser::*;

    fn parse_str(script: &'static str)
        -> Result<Node, &'static str> 
    {
        parse(script.to_string())
    }

    fn exec_str_pre(script: &'static str)
        -> Result<Num, String> 
    {
        execute(&parse_str(script).unwrap())
    }

    fn exec_str(script: &'static str)
        -> Num
    {
        exec_str_pre(script).unwrap()
    }

    #[test]
    fn parse_simple() 
    {
        // addition
        assert_eq!(exec_str("1+1"), 2.0);
        assert_eq!(exec_str("18+18"), 36.0);

        // subtraction
        assert_eq!(exec_str("18-18"), 0.0);

        // multiplication
        assert_eq!(exec_str("18*18"), 324.0);

        // division
        assert_eq!(exec_str("18/18"), 1.0);

        // power
        assert_eq!(exec_str("10^3"), 1000.0);

        // division with zero
        assert!(exec_str_pre("18/0").is_err(), "division with zero is not possible");
    }

    #[test]
    fn parse_long()
    {
        // addition & subtraction
        assert_eq!(exec_str("1+1-1+1-1+1-1+1-1"), 1.0);

        // multiplication & division 
        assert_eq!(exec_str("2*5/2*5/2*5"), 62.5);

        // mixed
        assert_eq!(exec_str("2+10/2-2*1+1"), 6.0);
        assert_eq!(exec_str("10*(2+1)"), 30.0);
        assert_eq!(exec_str("10*(2*(2+1)-1)-1"), 49.0);
        assert_eq!(exec_str("10*[2+1]"), 30.0);
        assert_eq!(exec_str("10*[2*(2+1)-1]-1"), 49.0);
    }

    #[test]
    fn parse_complex()
    {
        // FIXME: should round a little
        // constants
        assert_eq!(exec_str("pi"), 3.141592653589793);
        assert_eq!(exec_str("2*pi"), 6.283185307179586);

        // assignments
        assert_eq!(exec_str("x=10"), 10.0);
        assert_eq!(exec_str("x=[(10*19)+10]*2"), 400.0);
    }

    #[test]
    fn parse_errors()
    {
        // two numbers
        assert!(parse_str("10 10").is_err(), "two numbers not allowed without operator");

        // paren nesting incorrect
        assert!(parse_str("10*((2*(2+1)-1)-1").is_err(), "nesting is not valid");
        assert!(parse_str("10*[(2*(2+1)-1]]-1").is_err(), "nesting is not valid");

        // empty expression
        assert!(parse_str("[]").is_err(), "empty expression is an error");
        assert!(parse_str("(())").is_err(), "empty expression is an error");

        // assignments
        assert!(exec_str_pre("2=10").is_err(), "assignment to number is not allowed");

        // function calls
        assert!(exec_str_pre("unknown()").is_err(), "unknown function called");
        assert!(exec_str_pre("sqrt()").is_err(), "too few arguments");
        assert!(exec_str_pre("sqrt(16,16)").is_err(), "too many arguments");
    }

    /*
    #[test]
    fn parse_function() {
        let script = String::from("sqrt(16)");
        let program = parse(script).unwrap();
        assert_eq!(execute(&program).unwrap(), 4.0);
    }
    */
}

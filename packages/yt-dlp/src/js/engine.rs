use anyhow::Result;
use boa_engine::{Context, Source};

/// Wrapper around boa_engine for executing JavaScript functions.
/// Used for YouTube signature decryption and n-parameter transformation.
pub struct JsEngine {
    context: Context,
}

impl JsEngine {
    pub fn new() -> Self {
        Self {
            context: Context::default(),
        }
    }

    /// Execute a JavaScript expression and return the result as a string.
    pub fn eval(&mut self, code: &str) -> Result<String> {
        let result = self
            .context
            .eval(Source::from_bytes(code))
            .map_err(|e| anyhow::anyhow!("JS eval error: {:?}", e))?;

        let string_result = result
            .to_string(&mut self.context)
            .map_err(|e| anyhow::anyhow!("JS toString error: {:?}", e))?;

        Ok(string_result.to_std_string_escaped())
    }

    /// Load JavaScript code into the engine context (e.g., helper functions).
    pub fn load(&mut self, code: &str) -> Result<()> {
        self.context
            .eval(Source::from_bytes(code))
            .map_err(|e| anyhow::anyhow!("JS load error: {:?}", e))?;
        Ok(())
    }

    /// Execute a function with a single string argument and return the result.
    /// The function_code should be a JS function expression like "function(a) { ... }".
    pub fn call_function(&mut self, function_code: &str, argument: &str) -> Result<String> {
        let escaped_arg = argument
            .replace('\\', "\\\\")
            .replace('"', "\\\"")
            .replace('\n', "\\n")
            .replace('\r', "\\r");

        let script = format!("({})(\"{}\");", function_code, escaped_arg);
        self.eval(&script)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_eval() {
        let mut engine = JsEngine::new();
        let result = engine.eval("1 + 2").unwrap();
        assert_eq!(result, "3");
    }

    #[test]
    fn test_string_eval() {
        let mut engine = JsEngine::new();
        let result = engine.eval("'hello' + ' ' + 'world'").unwrap();
        assert_eq!(result, "hello world");
    }

    #[test]
    fn test_call_function() {
        let mut engine = JsEngine::new();
        let result = engine
            .call_function("function(a) { return a.split('').reverse().join(''); }", "hello")
            .unwrap();
        assert_eq!(result, "olleh");
    }

    #[test]
    fn test_array_manipulation() {
        let mut engine = JsEngine::new();
        let func = r#"function(a) {
            var b = a.split('');
            b.splice(0, 1);
            b.reverse();
            b.splice(0, 2);
            return b.join('');
        }"#;
        let result = engine.call_function(func, "abcdefg").unwrap();
        assert_eq!(result, "edcb");
    }

    #[test]
    fn test_load_and_call() {
        let mut engine = JsEngine::new();
        engine
            .load("function myHelper(x) { return x * 2; }")
            .unwrap();
        let result = engine.eval("myHelper(21)").unwrap();
        assert_eq!(result, "42");
    }
}

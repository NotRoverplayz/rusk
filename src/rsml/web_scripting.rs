use crate::ast::{Expr, Program, Statement};

pub struct WebRuskCompiler;

impl WebRuskCompiler {
    pub fn compile(program: &Program) -> Result<String, String> {
        let mut statements = Vec::new();
        for statement in &program.statements {
            statements.push(Self::compile_statement(statement, 2)?);
        }

        let mut output = String::from("(() => {\n");
        output.push_str("  const ruskState = {};\n");
        output.push_str("  const ruskFunctions = {};\n");
        output.push_str("  let ruskLocals = {};\n");
        output.push_str("  const ruskBind = (name, value) => {\n");
        output.push_str("    if (!/^[A-Za-z_][A-Za-z0-9_]*$/.test(name)) throw new Error(`Invalid RUSK variable name: ${name}`);\n");
        output.push_str("    if (Object.prototype.hasOwnProperty.call(ruskLocals, name)) ruskLocals[name] = value;\n");
        output.push_str("    else ruskState[name] = value;\n");
        output.push_str("    if (window.RuskWeb && typeof window.RuskWeb.setState === \"function\") window.RuskWeb.setState(name, value);\n");
        output.push_str("    return value;\n");
        output.push_str("  };\n");
        output.push_str("  const ruskGet = name => Object.prototype.hasOwnProperty.call(ruskLocals, name) ? ruskLocals[name] : ruskState[name];\n");
        output.push_str("  const ruskBool = value => typeof value === \"boolean\" ? value : typeof value === \"number\" ? value !== 0 : typeof value === \"string\" ? value.length > 0 : Array.isArray(value) ? value.length > 0 : value != null;\n");
        output.push_str("  const ruskString = value => value == null ? \"null\" : Array.isArray(value) ? `[${value.map(ruskString).join(\", \")}]` : String(value);\n");
        output.push_str("  const ruskAdd = (left, right) => typeof left === \"string\" || typeof right === \"string\" ? ruskString(left) + ruskString(right) : left + right;\n");
        output.push_str("  const ruskEqual = (left, right) => ruskString(left) === ruskString(right);\n");
        output.push_str("  const ruskRepeat = (count, body) => {\n");
        output.push_str("    count = Number(count);\n");
        output.push_str("    if (!Number.isFinite(count) || count < 0 || count > 1000000 || !Number.isInteger(count)) throw new Error(\"Repeat count must be an integer between 0 and 1,000,000\");\n");
        output.push_str("    for (let i = 0; i < count; i++) body();\n");
        output.push_str("  };\n");
        output.push_str("  const ruskDesktopOnly = name => { throw new Error(`RUSK built-in '${name}()' is desktop-only and unavailable in the web runtime`); };\n");
        output.push_str("  const ruskBuiltin = (name, args) => {\n");
        output.push_str("    switch (name) {\n");
        output.push_str("      case \"length\": if (args.length !== 1) throw new Error(\"length() expects exactly one argument\"); if (typeof args[0] === \"string\" || Array.isArray(args[0])) return args[0].length; throw new Error(\"length() only supports strings and arrays\");\n");
        output.push_str("      case \"to_number\": if (args.length !== 1) throw new Error(\"to_number() expects exactly one argument\"); { const value = Number(args[0]); if (!Number.isFinite(value)) throw new Error(`'${args[0]}' is not a valid number`); return value; }\n");
        output.push_str("      case \"random\": if (args.length !== 2) throw new Error(\"random() expects minimum and maximum\"); if (args[0] > args[1]) throw new Error(\"Random minimum cannot exceed maximum\"); return args[0] + Math.random() * (args[1] - args[0]);\n");
        output.push_str("      case \"random_int\": if (args.length !== 2) throw new Error(\"random_int() expects minimum and maximum\"); { const min = Number(args[0]), max = Number(args[1]); if (!Number.isInteger(min) || !Number.isInteger(max) || min > max) throw new Error(\"random_int() bounds must be ordered integers\"); return Math.floor(Math.random() * (max - min + 1)) + min; }\n");
        output.push_str("      case \"say\": case \"print\": if (args.length !== 1) throw new Error(`${name}() expects exactly one argument`); console.log(ruskString(args[0])); return null;\n");
        output.push_str("      case \"window\": case \"input_box\": case \"input_value\": case \"input_submitted\": case \"clear_input\": case \"clear\": case \"rect\": case \"pixel\": case \"rect_outline\": case \"rounded_rect\": case \"rounded_rect_outline\": case \"circle\": case \"circle_outline\": case \"line\": case \"triangle\": case \"polygon\": case \"polygon_outline\": case \"arc\": case \"ellipse\": case \"ellipse_outline\": case \"text\": case \"color\": case \"rgb\": case \"rgba\": case \"gradient_vertical\": case \"gradient_horizontal\": case \"key_down\": case \"key_pressed\": case \"mouse_down\": case \"mouse_pressed\": case \"mouse_x\": case \"mouse_y\": case \"delta_time\": case \"time\": case \"fps\": case \"screen_width\": case \"screen_height\": case \"quit\": ruskDesktopOnly(name);\n");
        output.push_str("      default: throw new Error(`Unknown RUSK built-in '${name}()'`);\n");
        output.push_str("    }\n");
        output.push_str("  };\n");
        output.push_str("  const ruskCall = (name, args = []) => {\n");
        output.push_str("    if (Object.prototype.hasOwnProperty.call(ruskFunctions, name)) {\n");
        output.push_str("      const fn = ruskFunctions[name];\n");
        output.push_str("      if (args.length !== fn.parameters.length) throw new Error(`Function '${name}' expected ${fn.parameters.length} arguments, got ${args.length}`);\n");
        output.push_str("      const previous = ruskLocals;\n");
        output.push_str("      ruskLocals = { ...previous };\n");
        output.push_str("      fn.parameters.forEach((parameter, index) => { ruskLocals[parameter] = args[index]; });\n");
        output.push_str("      try { fn.body(); } finally { ruskLocals = previous; }\n");
        output.push_str("      return null;\n");
        output.push_str("    }\n");
        output.push_str("    return ruskBuiltin(name, args);\n");
        output.push_str("  };\n");

        for statement in &program.statements {
            if let Statement::Function { name, parameters, .. } = statement {
                let body = match statement {
                    Statement::Function { body, .. } => body,
                    _ => unreachable!(),
                };
                output.push_str(&format!("  ruskFunctions[{}] = {{ parameters: {:?}, body: () => {{\n", Self::js_string(name), parameters));
                for body_statement in body {
                    output.push_str(&Self::compile_statement(body_statement, 4)?);
                    output.push('\n');
                }
                output.push_str("  } };\n");
            }
        }
        for statement in &program.statements {
            if !matches!(statement, Statement::Function { .. }) {
                output.push_str(&Self::compile_statement(statement, 2)?);
                output.push('\n');
            }
        }

        output.push_str("  const externalApp = window.RuskApp && typeof window.RuskApp === \"object\" ? window.RuskApp : {};\n");
        output.push_str("  window.RuskApp = { ...externalApp };\n");
        output.push_str("  Object.entries(ruskFunctions).forEach(([name, fn]) => {\n");
        output.push_str("    if (!Object.prototype.hasOwnProperty.call(window.RuskApp, name)) window.RuskApp[name] = payload => ruskCall(name, payload && payload.ruskArgs ? payload.ruskArgs : []);\n");
        output.push_str("  });\n");
        output.push_str("  window.RuskWeb.execute = (name, args = []) => ruskCall(name, args);\n");
        output.push_str("})();\n");

        Ok(output)
    }

    fn compile_statement(statement: &Statement, indent: usize) -> Result<String, String> {
        let pad = " ".repeat(indent);
        match statement {
            Statement::Let(name, expression) => Ok(format!("{pad}ruskBind({}, {});", Self::js_string(name), Self::compile_expr(expression)?)),
            Statement::Set(name, expression) => Ok(format!("{pad}ruskBind({}, {});", Self::js_string(name), Self::compile_expr(expression)?)),
            Statement::SetIndex { name, index, value } => Ok(format!(
                "{pad}{{ const ruskArray = ruskGet({name}); const ruskIndex = {index}; if (!Array.isArray(ruskArray)) throw new Error(\"'{name}' is not an array\"); ruskArray[ruskIndex] = {value}; ruskBind({name}, ruskArray); }}",
                name = Self::js_string(name),
                index = Self::compile_expr(index)?,
                value = Self::compile_expr(value)?,
            )),
            Statement::Append { value, array } => Ok(format!(
                "{pad}{{ const ruskArray = ruskGet({array}); if (!Array.isArray(ruskArray)) throw new Error(\"'{array}' is not an array\"); ruskArray.push({value}); ruskBind({array}, ruskArray); }}",
                array = Self::js_string(array),
                value = Self::compile_expr(value)?,
            )),
            Statement::Say(expression) => Ok(format!("{pad}console.log(ruskString({}));", Self::compile_expr(expression)?)),
            Statement::Expression(expression) => Ok(format!("{pad}{};", Self::compile_expr(expression)?)),
            Statement::Function { .. } => Ok(String::new()),
            Statement::If { condition, then_branch, else_branch } => {
                let mut output = format!("{pad}if (ruskBool({})) {{\n", Self::compile_expr(condition)?);
                for statement in then_branch {
                    output.push_str(&Self::compile_statement(statement, indent + 2)?);
                    output.push('\n');
                }
                output.push_str(&format!("{pad}}}"));
                if !else_branch.is_empty() {
                    output.push_str(" else {\n");
                    for statement in else_branch {
                        output.push_str(&Self::compile_statement(statement, indent + 2)?);
                        output.push('\n');
                    }
                    output.push_str(&format!("{pad}}}"));
                }
                Ok(output)
            }
            Statement::Repeat { count, body } => {
                let mut output = format!("{pad}ruskRepeat({}, () => {{\n", Self::compile_expr(count)?);
                for statement in body {
                    output.push_str(&Self::compile_statement(statement, indent + 2)?);
                    output.push('\n');
                }
                output.push_str(&format!("{pad}}});"));
                Ok(output)
            }
        }
    }

    fn compile_expr(expression: &Expr) -> Result<String, String> {
        match expression {
            Expr::Number(value) if value.is_finite() => Ok(value.to_string()),
            Expr::Number(_) => Err("RUSK web compiler cannot emit a non-finite number".into()),
            Expr::String(value) => Ok(Self::js_string(value)),
            Expr::Boolean(value) => Ok(value.to_string()),
            Expr::Variable(name) => Ok(format!("ruskGet({})", Self::js_string(name))),
            Expr::Array(values) => {
                let values: Result<Vec<String>, String> = values.iter().map(Self::compile_expr).collect();
                Ok(format!("[{}]", values?.join(", ")))
            }
            Expr::Index { array, index } => Ok(format!(
                "({}[{}])",
                Self::compile_expr(array)?,
                Self::compile_expr(index)?
            )),
            Expr::Unary { operator, expression } => {
                let value = Self::compile_expr(expression)?;
                match operator.as_str() {
                    "-" => Ok(format!("(-({value}))")),
                    "not" => Ok(format!("(!ruskBool({value}))")),
                    _ => Err(format!("Unknown unary operator '{operator}'")),
                }
            }
            Expr::Binary { left, operator, right } => {
                let left = Self::compile_expr(left)?;
                let right = Self::compile_expr(right)?;
                let expression = match operator.as_str() {
                    "+" => format!("ruskAdd({left}, {right})"),
                    "-" => format!("(({left}) - ({right}))"),
                    "*" => format!("(({left}) * ({right}))"),
                    "/" => format!("(({right}) === 0 ? (() => {{ throw new Error(\"Division by zero\") }})() : (({left}) / ({right})))"),
                    "==" => format!("ruskEqual({left}, {right})"),
                    "!=" => format!("(!ruskEqual({left}, {right}))"),
                    "<" => format!("(({left}) < ({right}))"),
                    "<=" => format!("(({left}) <= ({right}))"),
                    ">" => format!("(({left}) > ({right}))"),
                    ">=" => format!("(({left}) >= ({right}))"),
                    "and" => format!("(ruskBool({left}) && ruskBool({right}))"),
                    "or" => format!("(ruskBool({left}) || ruskBool({right}))"),
                    _ => return Err(format!("Unknown RUSK operator '{operator}'")),
                };
                Ok(expression)
            }
            Expr::Call { name, arguments } => {
                let arguments: Result<Vec<String>, String> = arguments.iter().map(Self::compile_expr).collect();
                Ok(format!("ruskCall({}, [{}])", Self::js_string(name), arguments?.join(", ")))
            }
        }
    }

    fn js_string(value: &str) -> String {
        let mut output = String::from("\"");
        for character in value.chars() {
            match character {
                '"' => output.push_str("\\\""),
                '\\' => output.push_str("\\\\"),
                '\n' => output.push_str("\\n"),
                '\r' => output.push_str("\\r"),
                '\t' => output.push_str("\\t"),
                '\u{2028}' => output.push_str("\\u2028"),
                '\u{2029}' => output.push_str("\\u2029"),
                c if c.is_control() => output.push_str(&format!("\\u{:04x}", c as u32)),
                c => output.push(c),
            }
        }
        output.push('"');
        output
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lexer::Lexer;
    use crate::parser::Parser;

    fn compile(source: &str) -> String {
        let tokens = Lexer::new(source).tokenize().unwrap();
        let program = Parser::new(tokens).parse().unwrap();
        WebRuskCompiler::compile(&program).unwrap()
    }

    #[test]
    fn compiles_variables_functions_conditions_and_loops() {
        let script = compile(
            r#"let counter be 0
function increment() {
    set counter to counter + 1
}
repeat 2 times
    set counter to counter + 1
end
"#
        );

        assert!(script.contains("ruskBind(\"counter\", 0"));
        assert!(script.contains("ruskFunctions[\"increment\"]"));
        assert!(script.contains("ruskRepeat(2"));
        assert!(script.contains("ruskBind(\"counter\", ruskAdd"));
        assert!(script.contains("window.RuskApp"));
    }

    #[test]
    fn compiles_web_supported_and_desktop_only_builtins() {
        let script = compile(
            r#"say "hello"
let values be [1, 2]
random_int(1, 3)"#
        );

        assert!(script.contains("ruskBuiltin"));
        assert!(script.contains("case \"say\""));
        assert!(script.contains("case \"random_int\""));
        assert!(script.contains("case \"rect\""));
        assert!(script.contains("desktop-only"));
    }
}

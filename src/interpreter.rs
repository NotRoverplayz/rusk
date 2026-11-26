
use std::collections::HashMap;

use macroquad::prelude::{
    clear_background, draw_text, get_char_pressed, is_key_pressed,
    is_quit_requested, next_frame, request_new_screen_size,
    Color, KeyCode, BLACK, WHITE,
};

use crate::ast::{Expr, Program, Statement};
use crate::graphics::Graphics;

#[derive(Debug, Clone)]
pub enum Value {
    Number(f64),
    String(String),
    Boolean(bool),
    Array(Vec<Value>),
    Color(Color),
    Null,
}

impl Value {
    fn as_number(&self) -> Result<f64, String> {
        match self {
            Value::Number(value) => Ok(*value),
            other => Err(format!("Expected number, got {:?}", other)),
        }
    }

    fn as_bool(&self) -> bool {
        match self {
            Value::Boolean(value) => *value,
            Value::Number(value) => *value != 0.0,
            Value::String(value) => !value.is_empty(),
            Value::Array(value) => !value.is_empty(),
            Value::Color(_) => true,
            Value::Null => false,
        }
    }

    fn as_string(&self) -> String {
        match self {
            Value::Number(value) if value.fract() == 0.0 => {
                format!("{}", *value as i64)
            }
            Value::Number(value) => value.to_string(),
            Value::String(value) => value.clone(),
            Value::Boolean(value) => value.to_string(),
            Value::Array(values) => {
                let items: Vec<String> = values
                    .iter()
                    .map(Value::as_string)
                    .collect();

                format!("[{}]", items.join(", "))
            }
            Value::Color(color) => format!(
                "#{:02X}{:02X}{:02X}{:02X}",
                (color.r * 255.0).round() as u8,
                (color.g * 255.0).round() as u8,
                (color.b * 255.0).round() as u8,
                (color.a * 255.0).round() as u8,
            ),
            Value::Null => "null".to_string(),
        }
    }

    fn as_index(&self) -> Result<usize, String> {
        let value = self.as_number()?;

        if !value.is_finite()
            || value < 0.0
            || value.fract() != 0.0
            || value > usize::MAX as f64
        {
            return Err("Array index must be a non-negative integer".into());
        }

        Ok(value as usize)
    }
}

#[derive(Clone)]
struct Function {
    parameters: Vec<String>,
    body: Vec<Statement>,
}

pub struct Interpreter {
    variables: HashMap<String, Value>,
    scopes: Vec<HashMap<String, Value>>,
    functions: HashMap<String, Function>,
    program: Program,
    output: Vec<String>,
    window_requested: bool,
    input_buffer: String,
    input_submitted: bool,
}

impl Interpreter {
    pub fn new(program: Program) -> Self {
        Self {
            variables: HashMap::new(),
            scopes: Vec::new(),
            functions: HashMap::new(),
            program,
            output: Vec::new(),
            window_requested: false,
            input_buffer: String::new(),
            input_submitted: false,
        }
    }

    pub async fn run(&mut self) -> Result<(), String> {
        let statements = self.program.statements.clone();
        self.execute_block(&statements)?;

        let has_game_loop =
            self.functions.contains_key("update")
            || self.functions.contains_key("draw");

        if self.window_requested || has_game_loop {
            loop {
                if is_quit_requested() {
                    break;
                }

                self.handle_input();

                if self.functions.contains_key("update") {
                    self.call_function("update", vec![])?;
                }

                if self.functions.contains_key("draw") {
                    self.call_function("draw", vec![])?;
                } else {
                    clear_background(BLACK);
                }

                for (index, line) in self.output.iter().enumerate() {
                    draw_text(
                        line,
                        20.0,
                        35.0 + index as f32 * 26.0,
                        24.0,
                        WHITE,
                    );
                }

                next_frame().await;
            }
        }

        Ok(())
    }

    fn handle_input(&mut self) {
        while let Some(character) = get_char_pressed() {
            if !character.is_control() {
                self.input_buffer.push(character);
            }
        }

        if is_key_pressed(KeyCode::Backspace) {
            self.input_buffer.pop();
        }

        if is_key_pressed(KeyCode::Enter) {
            self.input_submitted = true;
        }
    }

    fn execute_block(
        &mut self,
        statements: &[Statement],
    ) -> Result<(), String> {
        for statement in statements {
            self.execute(statement)?;
        }

        Ok(())
    }

    fn execute(&mut self, statement: &Statement) -> Result<(), String> {
        match statement {
            Statement::Let(name, expression) => {
                let value = self.evaluate(expression)?;
                if let Some(scope) = self.scopes.last_mut() {
                    scope.insert(name.clone(), value);
                } else {
                    self.variables.insert(name.clone(), value);
                }
            }

            Statement::Set(name, expression) => {
                let value = self.evaluate(expression)?;
                if let Some(scope) = self.scopes.iter_mut().rev().find(|scope| scope.contains_key(name)) {
                    scope.insert(name.clone(), value);
                } else if self.variables.contains_key(name) {
                    self.variables.insert(name.clone(), value);
                } else {
                    return Err(format!(
                        "Cannot set unknown variable '{}'",
                        name
                    ));
                }
            }

            Statement::SetIndex { name, index, value } => {
                let index = self.evaluate(index)?.as_index()?;
                let value = self.evaluate(value)?;

                let array = if let Some(scope) = self.scopes.iter_mut().rev().find(|scope| scope.contains_key(name)) {
                    scope.get_mut(name)
                } else {
                    self.variables.get_mut(name)
                }.ok_or_else(|| format!("Unknown variable '{}'", name))?;

                match array {
                    Value::Array(elements) => {
                        let length = elements.len();

                        if index >= length {
                            return Err(format!(
                                "Array index {} out of bounds (length {})",
                                index,
                                length
                            ));
                        }

                        elements[index] = value;
                    }

                    _ => {
                        return Err(format!(
                            "'{}' is not an array",
                            name
                        ));
                    }
                }
            }

            Statement::Append { value, array } => {
                let value = self.evaluate(value)?;

                let target = if let Some(scope) = self.scopes.iter_mut().rev().find(|scope| scope.contains_key(array)) {
                    scope.get_mut(array)
                } else {
                    self.variables.get_mut(array)
                }.ok_or_else(|| format!("Unknown variable '{}'", array))?;

                match target {
                    Value::Array(elements) => {
                        elements.push(value);
                    }

                    _ => {
                        return Err(format!(
                            "'{}' is not an array",
                            array
                        ));
                    }
                }
            }

            Statement::Say(expression) => {
                let value = self.evaluate(expression)?;
                let output = value.as_string();

                println!("{}", output);
                self.output.push(output);
            }

            Statement::Expression(expression) => {
                self.evaluate(expression)?;
            }

            Statement::Function {
                name,
                parameters,
                body,
            } => {
                self.functions.insert(
                    name.clone(),
                    Function {
                        parameters: parameters.clone(),
                        body: body.clone(),
                    },
                );
            }

            Statement::If {
                condition,
                then_branch,
                else_branch,
            } => {
                let condition = self.evaluate(condition)?;

                if condition.as_bool() {
                    self.execute_block(then_branch)?;
                } else {
                    self.execute_block(else_branch)?;
                }
            }

            Statement::Repeat { count, body } => {
                let count = self.evaluate(count)?.as_number()?;

                if count < 0.0 || count > 1_000_000.0 {
                    return Err(
                        "Repeat count must be between 0 and 1,000,000".into()
                    );
                }

                for _ in 0..count as usize {
                    self.execute_block(body)?;
                }
            }
        }

        Ok(())
    }

    fn evaluate(&mut self, expression: &Expr) -> Result<Value, String> {
        match expression {
            Expr::Number(value) => Ok(Value::Number(*value)),

            Expr::String(value) => {
                Ok(Value::String(value.clone()))
            }

            Expr::Boolean(value) => {
                Ok(Value::Boolean(*value))
            }

            Expr::Variable(name) => {
                let scoped = self.scopes.iter().rev()
                    .find_map(|scope| scope.get(name).cloned());
                scoped.or_else(|| self.variables.get(name).cloned())
                    .ok_or_else(|| format!("Unknown variable '{}'", name))
            }

            Expr::Array(elements) => {
                let mut values = Vec::new();

                for element in elements {
                    values.push(self.evaluate(element)?);
                }

                Ok(Value::Array(values))
            }

            Expr::Index { array, index } => {
                let array = self.evaluate(array)?;
                let index = self.evaluate(index)?.as_index()?;

                match array {
                    Value::Array(elements) => {
                        elements.get(index)
                            .cloned()
                            .ok_or_else(|| {
                                format!(
                                    "Array index {} out of bounds (length {})",
                                    index,
                                    elements.len()
                                )
                            })
                    }

                    _ => Err("Cannot index a non-array value".into()),
                }
            }

            Expr::Unary {
                operator,
                expression,
            } => {
                let value = self.evaluate(expression)?;

                match operator.as_str() {
                    "-" => Ok(Value::Number(-value.as_number()?)),
                    "not" => Ok(Value::Boolean(!value.as_bool())),

                    _ => Err(format!(
                        "Unknown unary operator '{}'",
                        operator
                    )),
                }
            }

            Expr::Binary {
                left,
                operator,
                right,
            } => {
                let left = self.evaluate(left)?;

                if operator == "and" && !left.as_bool() {
                    return Ok(Value::Boolean(false));
                }

                if operator == "or" && left.as_bool() {
                    return Ok(Value::Boolean(true));
                }

                let right = self.evaluate(right)?;

                match operator.as_str() {
                    "+" => match (&left, &right) {
                        (Value::String(_), _) | (_, Value::String(_)) => {
                            Ok(Value::String(format!(
                                "{}{}",
                                left.as_string(),
                                right.as_string()
                            )))
                        }

                        _ => Ok(Value::Number(
                            left.as_number()? + right.as_number()?
                        )),
                    },

                    "-" => Ok(Value::Number(
                        left.as_number()? - right.as_number()?
                    )),

                    "*" => Ok(Value::Number(
                        left.as_number()? * right.as_number()?
                    )),

                    "/" => {
                        let divisor = right.as_number()?;

                        if divisor == 0.0 {
                            return Err("Division by zero".into());
                        }

                        Ok(Value::Number(
                            left.as_number()? / divisor
                        ))
                    }

                    "==" => Ok(Value::Boolean(
                        left.as_string() == right.as_string()
                    )),

                    "!=" => Ok(Value::Boolean(
                        left.as_string() != right.as_string()
                    )),

                    "<" => Ok(Value::Boolean(
                        left.as_number()? < right.as_number()?
                    )),

                    "<=" => Ok(Value::Boolean(
                        left.as_number()? <= right.as_number()?
                    )),

                    ">" => Ok(Value::Boolean(
                        left.as_number()? > right.as_number()?
                    )),

                    ">=" => Ok(Value::Boolean(
                        left.as_number()? >= right.as_number()?
                    )),

                    "and" => Ok(Value::Boolean(
                        left.as_bool() && right.as_bool()
                    )),

                    "or" => Ok(Value::Boolean(
                        left.as_bool() || right.as_bool()
                    )),

                    _ => Err(format!(
                        "Unknown operator '{}'",
                        operator
                    )),
                }
            }

            Expr::Call { name, arguments } => {
                let mut values = Vec::new();

                for argument in arguments {
                    values.push(self.evaluate(argument)?);
                }

                self.call_builtin(name, values)
            }
        }
    }

    fn call_function(
        &mut self,
        name: &str,
        arguments: Vec<Value>,
    ) -> Result<Value, String> {
        let function = self.functions
            .get(name)
            .cloned()
            .ok_or_else(|| {
                format!("Unknown function '{}'", name)
            })?;

        if arguments.len() != function.parameters.len() {
            return Err(format!(
                "Function '{}' expected {} arguments, got {}",
                name,
                function.parameters.len(),
                arguments.len()
            ));
        }

        let mut scope = HashMap::new();
        for (parameter, value) in function.parameters.iter().zip(arguments) {
            scope.insert(parameter.clone(), value);
        }
        self.scopes.push(scope);
        let result = self.execute_block(&function.body);
        self.scopes.pop();
        result?;

        Ok(Value::Null)
    }

    fn call_builtin(
        &mut self,
        name: &str,
        arguments: Vec<Value>,
    ) -> Result<Value, String> {
        let number = |index: usize| -> Result<f64, String> {
            arguments
                .get(index)
                .ok_or_else(|| {
                    format!("Missing argument {}", index + 1)
                })?
                .as_number()
        };

        let string = |index: usize| -> Result<String, String> {
            arguments
                .get(index)
                .map(Value::as_string)
                .ok_or_else(|| {
                    format!("Missing argument {}", index + 1)
                })
        };

        let color = |index: usize| -> Result<Color, String> {
            match arguments.get(index) {
                Some(Value::String(value)) => Graphics::parse_color(value),
                Some(Value::Color(value)) => Ok(*value),
                Some(other) => Err(format!("Expected color, got {:?}", other)),
                None => Err(format!("Missing argument {}", index + 1)),
            }
        };

        let check_arity = |expected: usize| -> Result<(), String> {
            if arguments.len() == expected {
                Ok(())
            } else {
                Err(format!(
                    "{}() expects {} arguments, got {}",
                    name,
                    expected,
                    arguments.len()
                ))
            }
        };

        match name {
            "length" => {
                if arguments.len() != 1 {
                    return Err(
                        "length() expects exactly one argument".into()
                    );
                }

                let length = match &arguments[0] {
                    Value::Array(values) => values.len(),
                    Value::String(value) => value.chars().count(),

                    other => {
                        return Err(format!(
                            "length() does not support {:?}",
                            other
                        ));
                    }
                };

                Ok(Value::Number(length as f64))
            }

           "window" => {
    if arguments.len() != 3 {
        return Err(
            "window() expects title, width and height".into()
        );
    }

    let _title = string(0)?;
    let width = number(1)?;
    let height = number(2)?;

    if !width.is_finite()
        || !height.is_finite()
        || width <= 0.0
        || height <= 0.0
        || width > 16384.0
        || height > 16384.0
    {
        return Err(
            "Window dimensions must be between 1 and 16384".into()
        );
    }

    self.window_requested = true;

    Graphics::set_reference_size(
        width as f32,
        height as f32,
    );

    request_new_screen_size(
        width as f32,
        height as f32,
    );

    Ok(Value::Null)
}

            "input_box" => {
                if arguments.len() != 5 {
                    return Err(
                        "input_box() expects prompt, x, y, width and height".into()
                    );
                }

                let prompt = string(0)?;
                let x = number(1)? as f32;
                let y = number(2)? as f32;
                let width = number(3)? as f32;
                let height = number(4)? as f32;

                Graphics::text(
                    &prompt,
                    x,
                    y - 10.0,
                    20.0,
                    Graphics::parse_color("white")?,
                );

                Graphics::rect(
                    x,
                    y,
                    width,
                    height,
                    Graphics::parse_color("gray")?,
                );

                Graphics::text(
                    &self.input_buffer,
                    x + 10.0,
                    y + height * 0.68,
                    24.0,
                    Graphics::parse_color("black")?,
                );

                Ok(Value::Null)
            }

            "input_value" => {
                Ok(Value::String(self.input_buffer.clone()))
            }

            "input_submitted" => {
                Ok(Value::Boolean(self.input_submitted))
            }

            "clear_input" => {
                self.input_buffer.clear();
                self.input_submitted = false;
                Ok(Value::Null)
            }

            "to_number" => {
                let value = string(0)?;

                let number = value.parse::<f64>()
                    .map_err(|_| {
                        format!("'{}' is not a valid number", value)
                    })?;

                Ok(Value::Number(number))
            }

            "clear" => {
                check_arity(1)?;
                Graphics::clear(color(0)?);
                Ok(Value::Null)
            }

            "rect" => {
                check_arity(5)?;
                Graphics::rect(
                    number(0)? as f32,
                    number(1)? as f32,
                    number(2)? as f32,
                    number(3)? as f32,
                    color(4)?,
                );

                Ok(Value::Null)
            }

            "pixel" => {
                check_arity(3)?;
                Graphics::pixel(number(0)? as f32, number(1)? as f32, color(2)?);
                Ok(Value::Null)
            }

            "rect_outline" => {
                check_arity(6)?;
                Graphics::rect_outline(
                    number(0)? as f32,
                    number(1)? as f32,
                    number(2)? as f32,
                    number(3)? as f32,
                    color(4)?,
                    number(5)? as f32,
                );

                Ok(Value::Null)
            }

            "rounded_rect" => {
                check_arity(6)?;
                Graphics::rounded_rect(
                    number(0)? as f32,
                    number(1)? as f32,
                    number(2)? as f32,
                    number(3)? as f32,
                    number(4)? as f32,
                    color(5)?,
                );

                Ok(Value::Null)
            }

            "rounded_rect_outline" => {
                check_arity(7)?;
                Graphics::rounded_rect_outline(
                    number(0)? as f32,
                    number(1)? as f32,
                    number(2)? as f32,
                    number(3)? as f32,
                    number(4)? as f32,
                    color(5)?,
                    number(6)? as f32,
                );

                Ok(Value::Null)
            }

            "circle" => {
                check_arity(4)?;
                Graphics::circle(
                    number(0)? as f32,
                    number(1)? as f32,
                    number(2)? as f32,
                    color(3)?,
                );

                Ok(Value::Null)
            }

            "circle_outline" => {
                check_arity(5)?;
                Graphics::circle_outline(
                    number(0)? as f32,
                    number(1)? as f32,
                    number(2)? as f32,
                    color(3)?,
                    number(4)? as f32,
                );

                Ok(Value::Null)
            }

            "line" => {
                check_arity(6)?;
                Graphics::line(
                    number(0)? as f32,
                    number(1)? as f32,
                    number(2)? as f32,
                    number(3)? as f32,
                    color(4)?,
                    number(5)? as f32,
                );

                Ok(Value::Null)
            }

            "triangle" => {
                check_arity(7)?;
                Graphics::triangle(
                    number(0)? as f32,
                    number(1)? as f32,
                    number(2)? as f32,
                    number(3)? as f32,
                    number(4)? as f32,
                    number(5)? as f32,
                    color(6)?,
                );

                Ok(Value::Null)
            }

            "polygon" => {
                check_arity(6)?;
                let sides = number(2)?;
                if sides.fract() != 0.0 || !(3.0..=255.0).contains(&sides) {
                    return Err(
                        "polygon() sides must be an integer from 3 to 255".into()
                    );
                }

                Graphics::polygon(
                    number(0)? as f32,
                    number(1)? as f32,
                    sides as u8,
                    number(3)? as f32,
                    number(4)? as f32,
                    color(5)?,
                );

                Ok(Value::Null)
            }

            "polygon_outline" => {
                check_arity(7)?;
                let sides = number(2)?;
                if sides.fract() != 0.0 || !(3.0..=255.0).contains(&sides) {
                    return Err(
                        "polygon_outline() sides must be an integer from 3 to 255".into()
                    );
                }

                Graphics::polygon_outline(
                    number(0)? as f32,
                    number(1)? as f32,
                    sides as u8,
                    number(3)? as f32,
                    number(4)? as f32,
                    color(5)?,
                    number(6)? as f32,
                );

                Ok(Value::Null)
            }

            "arc" => {
                check_arity(7)?;
                Graphics::arc(
                    number(0)? as f32,
                    number(1)? as f32,
                    number(2)? as f32,
                    number(3)? as f32,
                    number(4)? as f32,
                    color(5)?,
                    number(6)? as f32,
                );

                Ok(Value::Null)
            }

            "ellipse" => {
                check_arity(6)?;
                Graphics::ellipse(
                    number(0)? as f32,
                    number(1)? as f32,
                    number(2)? as f32,
                    number(3)? as f32,
                    number(4)? as f32,
                    color(5)?,
                );

                Ok(Value::Null)
            }

            "ellipse_outline" => {
                check_arity(7)?;
                Graphics::ellipse_outline(
                    number(0)? as f32,
                    number(1)? as f32,
                    number(2)? as f32,
                    number(3)? as f32,
                    number(4)? as f32,
                    color(5)?,
                    number(6)? as f32,
                );

                Ok(Value::Null)
            }

            "text" => {
                check_arity(5)?;
                Graphics::text(
                    &string(0)?,
                    number(1)? as f32,
                    number(2)? as f32,
                    number(3)? as f32,
                    color(4)?,
                );

                Ok(Value::Null)
            }

            "color" => {
                check_arity(1)?;
                match &arguments[0] {
                    Value::String(value) => {
                        Ok(Value::Color(Graphics::parse_color(value)?))
                    }
                    other => Err(format!(
                        "color() expects a color string, got {:?}",
                        other
                    )),
                }
            }

            "rgb" => {
                check_arity(3)?;
                Ok(Value::Color(Graphics::rgb(
                    number(0)?,
                    number(1)?,
                    number(2)?,
                )?))
            }

            "rgba" => {
                check_arity(4)?;
                Ok(Value::Color(Graphics::rgba(
                    number(0)?,
                    number(1)?,
                    number(2)?,
                    number(3)?,
                )?))
            }

            "gradient_vertical" => {
                check_arity(2)?;
                Graphics::gradient_vertical(color(0)?, color(1)?);
                Ok(Value::Null)
            }

            "gradient_horizontal" => {
                check_arity(2)?;
                Graphics::gradient_horizontal(color(0)?, color(1)?);
                Ok(Value::Null)
            }

            "key_down" => {
                check_arity(1)?;
                Ok(Value::Boolean(
                    Graphics::key_down(&string(0)?)
                ))
            }

            "key_pressed" => {
                check_arity(1)?;
                Ok(Value::Boolean(
                    Graphics::key_pressed(&string(0)?)
                ))
            }

            "mouse_down" => {
                check_arity(1)?;
                Ok(Value::Boolean(
                    Graphics::mouse_down(&string(0)?)?
                ))
            }

            "mouse_pressed" => {
                check_arity(1)?;
                Ok(Value::Boolean(
                    Graphics::mouse_pressed(&string(0)?)?
                ))
            }

            "mouse_x" => {
                check_arity(0)?;
                Ok(Value::Number(Graphics::mouse_x() as f64))
            }

            "mouse_y" => {
                check_arity(0)?;
                Ok(Value::Number(Graphics::mouse_y() as f64))
            }

            "delta_time" => {
                check_arity(0)?;
                Ok(Value::Number(
                    Graphics::delta_time() as f64
                ))
            }

            "time" => {
                check_arity(0)?;
                Ok(Value::Number(Graphics::time()))
            }

            "fps" => {
                check_arity(0)?;
                Ok(Value::Number(Graphics::fps() as f64))
            }

            "screen_width" => {
                check_arity(0)?;

                Ok(Value::Number(
                    Graphics::screen_width() as f64
                ))
            }

            "screen_height" => {
                check_arity(0)?;

                Ok(Value::Number(
                    Graphics::screen_height() as f64
                ))
            }

            "quit" => {
                std::process::exit(0);
            }

            "random" => {
                use rand::Rng;

                let min = number(0)?;
                let max = number(1)?;

                if min > max {
                    return Err(
                        "Random minimum cannot exceed maximum".into()
                    );
                }

                let mut rng = rand::thread_rng();

                Ok(Value::Number(rng.gen_range(min..=max)))
            }

            "random_int" => {
                if arguments.len() != 2 {
                    return Err(
                        "random_int() expects minimum and maximum".into()
                    );
                }

                use rand::Rng;

                let min = number(0)?;
                let max = number(1)?;
                let max_i64_exclusive = -(i64::MIN as f64);

                if !min.is_finite()
                    || !max.is_finite()
                    || min.fract() != 0.0
                    || max.fract() != 0.0
                    || min < i64::MIN as f64
                    || max >= max_i64_exclusive
                {
                    return Err(
                        "random_int() bounds must be integers in the i64 range".into()
                    );
                }

                if min > max {
                    return Err(
                        "Random minimum cannot exceed maximum".into()
                    );
                }

                let mut rng = rand::thread_rng();
                Ok(Value::Number(
                    rng.gen_range(min as i64..=max as i64) as f64
                ))
            }

            _ => self.call_function(name, arguments),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::Program;
    use crate::lexer::Lexer;
    use crate::parser::Parser;

    fn interpreter() -> Interpreter {
        Interpreter::new(Program { statements: vec![] })
    }

    #[test]
    fn function_parameters_do_not_overwrite_globals() {
        let tokens = Lexer::new(
            r#"let counter be 10
function add(value) {
    set value to value + 1
}
add(3)"#
        ).tokenize().unwrap();
        let program = Parser::new(tokens).parse().unwrap();
        let mut interpreter = Interpreter::new(program.clone());

        interpreter.execute_block(&program.statements).unwrap();

        assert!(matches!(
            interpreter.variables.get("counter"),
            Some(Value::Number(value)) if *value == 10.0
        ));
        assert!(!interpreter.variables.contains_key("value"));
    }

    #[test]
    fn random_int_returns_an_integer_in_the_inclusive_range() {
        let mut interpreter = interpreter();

        for _ in 0..100 {
            let value = interpreter
                .call_builtin(
                    "random_int",
                    vec![Value::Number(0.0), Value::Number(39.0)],
                )
                .unwrap()
                .as_number()
                .unwrap();

            assert!(value.fract() == 0.0);
            assert!((0.0..=39.0).contains(&value));
        }
    }

    #[test]
    fn random_int_rejects_fractional_bounds() {
        let mut interpreter = interpreter();

        let result = interpreter.call_builtin(
            "random_int",
            vec![Value::Number(0.5), Value::Number(39.0)],
        );

        assert!(result.is_err());
    }

    #[test]
    fn color_builtins_create_validated_color_values() {
        let mut interpreter = interpreter();
        let color = interpreter
            .call_builtin(
                "rgba",
                vec![
                    Value::Number(255.0),
                    Value::Number(100.0),
                    Value::Number(50.0),
                    Value::Number(128.0),
                ],
            )
            .unwrap();

        assert!(matches!(
            color,
            Value::Color(color)
                if color == Color::from_rgba(255, 100, 50, 128)
        ));

        let invalid = interpreter.call_builtin(
            "rgb",
            vec![
                Value::Number(256.0),
                Value::Number(100.0),
                Value::Number(50.0),
            ],
        );
        assert!(invalid.is_err());
    }
}

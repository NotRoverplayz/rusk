
use macroquad::prelude::*;
use std::sync::atomic::{AtomicU32, Ordering};

static REFERENCE_WIDTH: AtomicU32 = AtomicU32::new(800.0f32.to_bits());
static REFERENCE_HEIGHT: AtomicU32 = AtomicU32::new(500.0f32.to_bits());

pub struct Graphics;

impl Graphics {
    // ---------------------------------------------------------
    // SCALING SYSTEM
    // ---------------------------------------------------------

    pub fn set_reference_size(width: f32, height: f32) {
        if width.is_finite() && width > 0.0 {
            REFERENCE_WIDTH.store(width.to_bits(), Ordering::Relaxed);
        }

        if height.is_finite() && height > 0.0 {
            REFERENCE_HEIGHT.store(height.to_bits(), Ordering::Relaxed);
        }
    }

    fn reference_width() -> f32 {
        f32::from_bits(REFERENCE_WIDTH.load(Ordering::Relaxed))
    }

    fn reference_height() -> f32 {
        f32::from_bits(REFERENCE_HEIGHT.load(Ordering::Relaxed))
    }

    fn scale_x() -> f32 {
        (screen_width() / Self::reference_width()).max(0.001)
    }

    fn scale_y() -> f32 {
        (screen_height() / Self::reference_height()).max(0.001)
    }

    fn scale() -> f32 {
        (Self::scale_x() * Self::scale_y()).sqrt()
    }

    fn x(value: f32) -> f32 {
        value * Self::scale_x()
    }

    fn y(value: f32) -> f32 {
        value * Self::scale_y()
    }

    fn width(value: f32) -> f32 {
        value * Self::scale_x()
    }

    fn height(value: f32) -> f32 {
        value * Self::scale_y()
    }

    fn thickness(value: f32) -> f32 {
        value * Self::scale()
    }

    // ---------------------------------------------------------
    // COLORS
    // ---------------------------------------------------------

    pub fn parse_color(value: &str) -> Result<Color, String> {
        let named = if value.eq_ignore_ascii_case("black") {
            Some(BLACK)
        } else if value.eq_ignore_ascii_case("white") {
            Some(WHITE)
        } else if value.eq_ignore_ascii_case("red") {
            Some(RED)
        } else if value.eq_ignore_ascii_case("green") {
            Some(GREEN)
        } else if value.eq_ignore_ascii_case("blue") {
            Some(BLUE)
        } else if value.eq_ignore_ascii_case("yellow") {
            Some(YELLOW)
        } else if value.eq_ignore_ascii_case("orange") {
            Some(ORANGE)
        } else if value.eq_ignore_ascii_case("purple") {
            Some(PURPLE)
        } else if value.eq_ignore_ascii_case("pink") {
            Some(PINK)
        } else if value.eq_ignore_ascii_case("gray")
            || value.eq_ignore_ascii_case("grey")
        {
            Some(GRAY)
        } else if value.eq_ignore_ascii_case("lime") {
            Some(LIME)
        } else if value.eq_ignore_ascii_case("cyan")
            || value.eq_ignore_ascii_case("skyblue")
        {
            Some(SKYBLUE)
        } else if value.eq_ignore_ascii_case("darkgray")
            || value.eq_ignore_ascii_case("darkgrey")
        {
            Some(Color::from_rgba(64, 64, 64, 255))
        } else if value.eq_ignore_ascii_case("lightgray")
            || value.eq_ignore_ascii_case("lightgrey")
        {
            Some(Color::from_rgba(211, 211, 211, 255))
        } else if value.eq_ignore_ascii_case("teal") {
            Some(Color::from_rgba(0, 128, 128, 255))
        } else if value.eq_ignore_ascii_case("navy") {
            Some(Color::from_rgba(0, 0, 128, 255))
        } else if value.eq_ignore_ascii_case("maroon") {
            Some(Color::from_rgba(128, 0, 0, 255))
        } else if value.eq_ignore_ascii_case("olive") {
            Some(Color::from_rgba(128, 128, 0, 255))
        } else if value.eq_ignore_ascii_case("silver") {
            Some(Color::from_rgba(192, 192, 192, 255))
        } else if value.eq_ignore_ascii_case("gold") {
            Some(Color::from_rgba(255, 215, 0, 255))
        } else if value.eq_ignore_ascii_case("brown") {
            Some(Color::from_rgba(165, 42, 42, 255))
        } else if value.eq_ignore_ascii_case("violet") {
            Some(Color::from_rgba(238, 130, 238, 255))
        } else if value.eq_ignore_ascii_case("transparent") {
            Some(Color::from_rgba(0, 0, 0, 0))
        } else {
            None
        };

        if let Some(color) = named {
            return Ok(color);
        }

        let hex = value.strip_prefix('#').unwrap_or(value);

        if hex.len() == 6 || hex.len() == 8 {
            if !hex.bytes().all(|byte| byte.is_ascii_hexdigit()) {
                return Err(format!("Invalid color '{}'", value));
            }

            let channel = |start| {
                u8::from_str_radix(&hex[start..start + 2], 16)
                    .map_err(|_| format!("Invalid color '{}'", value))
            };

            let alpha = if hex.len() == 8 {
                channel(6)?
            } else {
                255
            };

            return Ok(Color::from_rgba(
                channel(0)?,
                channel(2)?,
                channel(4)?,
                alpha,
            ));
        }

        Err(format!("Invalid color '{}'", value))
    }

    pub fn rgb(red: f64, green: f64, blue: f64) -> Result<Color, String> {
        Self::rgba(red, green, blue, 255.0)
    }

    pub fn rgba(
        red: f64,
        green: f64,
        blue: f64,
        alpha: f64,
    ) -> Result<Color, String> {
        let channel = |value: f64| -> Result<u8, String> {
            if !value.is_finite()
                || value.fract() != 0.0
                || !(0.0..=255.0).contains(&value)
            {
                return Err(
                    "Color channels must be integers from 0 to 255".into()
                );
            }

            Ok(value as u8)
        };

        Ok(Color::from_rgba(
            channel(red)?,
            channel(green)?,
            channel(blue)?,
            channel(alpha)?,
        ))
    }

    // ---------------------------------------------------------
    // BASIC DRAWING
    // ---------------------------------------------------------

    pub fn clear(color: Color) {
        clear_background(color);
    }

    pub fn rect(
        x: f32,
        y: f32,
        width: f32,
        height: f32,
        color: Color,
    ) {
        draw_rectangle(
            Self::x(x),
            Self::y(y),
            Self::width(width),
            Self::height(height),
            color,
        );
    }

    pub fn pixel(x: f32, y: f32, color: Color) {
        Self::rect(x, y, 1.0, 1.0, color);
    }

    pub fn rect_outline(
        x: f32,
        y: f32,
        width: f32,
        height: f32,
        color: Color,
        thickness: f32,
    ) {
        draw_rectangle_lines(
            Self::x(x),
            Self::y(y),
            Self::width(width),
            Self::height(height),
            Self::thickness(thickness),
            color,
        );
    }

    // ---------------------------------------------------------
    // ROUNDED RECTANGLES
    // ---------------------------------------------------------

    pub fn rounded_rect(
        x: f32,
        y: f32,
        width: f32,
        height: f32,
        radius: f32,
        color: Color,
    ) {
        let x = Self::x(x);
        let y = Self::y(y);
        let width = Self::width(width);
        let height = Self::height(height);

        let radius_x = Self::width(radius)
            .max(0.0)
            .min(width.min(height) / 2.0);

        let radius_y = Self::height(radius)
            .max(0.0)
            .min(width.min(height) / 2.0);

        if radius_x <= 0.0 || radius_y <= 0.0 {
            draw_rectangle(x, y, width, height, color);
            return;
        }

        draw_rectangle(
            x + radius_x,
            y,
            width - 2.0 * radius_x,
            height,
            color,
        );

        draw_rectangle(
            x,
            y + radius_y,
            width,
            height - 2.0 * radius_y,
            color,
        );

        draw_ellipse(x + radius_x, y + radius_y, radius_x, radius_y, 0.0, color);
        draw_ellipse(x + width - radius_x, y + radius_y, radius_x, radius_y, 0.0, color);
        draw_ellipse(x + width - radius_x, y + height - radius_y, radius_x, radius_y, 0.0, color);
        draw_ellipse(x + radius_x, y + height - radius_y, radius_x, radius_y, 0.0, color);
    }

    pub fn rounded_rect_outline(
        x: f32,
        y: f32,
        width: f32,
        height: f32,
        radius: f32,
        color: Color,
        thickness: f32,
    ) {
        let x = Self::x(x);
        let y = Self::y(y);
        let width = Self::width(width);
        let height = Self::height(height);

        let radius_x = Self::width(radius)
            .max(0.0)
            .min(width.min(height) / 2.0);

        let radius_y = Self::height(radius)
            .max(0.0)
            .min(width.min(height) / 2.0);

        let thickness = Self::thickness(thickness);

        if radius_x <= 0.0 || radius_y <= 0.0 {
            draw_rectangle_lines(x, y, width, height, thickness, color);
            return;
        }

        draw_line(
            x + radius_x,
            y,
            x + width - radius_x,
            y,
            thickness,
            color,
        );

        draw_line(
            x + width,
            y + radius_y,
            x + width,
            y + height - radius_y,
            thickness,
            color,
        );

        draw_line(
            x + width - radius_x,
            y + height,
            x + radius_x,
            y + height,
            thickness,
            color,
        );

        draw_line(
            x,
            y + height - radius_y,
            x,
            y + radius_y,
            thickness,
            color,
        );

        Self::rounded_corner(
            x + radius_x,
            y + radius_y,
            radius_x,
            radius_y,
            std::f32::consts::PI,
            1.5 * std::f32::consts::PI,
            color,
            thickness,
        );

        Self::rounded_corner(
            x + width - radius_x,
            y + radius_y,
            radius_x,
            radius_y,
            1.5 * std::f32::consts::PI,
            2.0 * std::f32::consts::PI,
            color,
            thickness,
        );

        Self::rounded_corner(
            x + width - radius_x,
            y + height - radius_y,
            radius_x,
            radius_y,
            0.0,
            0.5 * std::f32::consts::PI,
            color,
            thickness,
        );

        Self::rounded_corner(
            x + radius_x,
            y + height - radius_y,
            radius_x,
            radius_y,
            0.5 * std::f32::consts::PI,
            std::f32::consts::PI,
            color,
            thickness,
        );
    }

    fn rounded_corner(
        center_x: f32,
        center_y: f32,
        radius_x: f32,
        radius_y: f32,
        start_angle: f32,
        end_angle: f32,
        color: Color,
        thickness: f32,
    ) {
        let segments = 8;
        let mut previous_angle = start_angle;

        for segment in 1..=segments {
            let angle = start_angle
                + (end_angle - start_angle) * segment as f32 / segments as f32;

            draw_line(
                center_x + radius_x * previous_angle.cos(),
                center_y + radius_y * previous_angle.sin(),
                center_x + radius_x * angle.cos(),
                center_y + radius_y * angle.sin(),
                thickness,
                color,
            );

            previous_angle = angle;
        }
    }

    // ---------------------------------------------------------
    // CIRCLES AND LINES
    // ---------------------------------------------------------

    pub fn circle(x: f32, y: f32, radius: f32, color: Color) {
        draw_ellipse(
            Self::x(x),
            Self::y(y),
            Self::width(radius),
            Self::height(radius),
            0.0,
            color,
        );
    }

    pub fn circle_outline(
        x: f32,
        y: f32,
        radius: f32,
        color: Color,
        thickness: f32,
    ) {
        draw_ellipse_lines(
            Self::x(x),
            Self::y(y),
            Self::width(radius),
            Self::height(radius),
            0.0,
            Self::thickness(thickness),
            color,
        );
    }

    pub fn line(
        x1: f32,
        y1: f32,
        x2: f32,
        y2: f32,
        color: Color,
        thickness: f32,
    ) {
        draw_line(
            Self::x(x1),
            Self::y(y1),
            Self::x(x2),
            Self::y(y2),
            Self::thickness(thickness),
            color,
        );
    }

    // ---------------------------------------------------------
    // TRIANGLES AND POLYGONS
    // ---------------------------------------------------------

    pub fn triangle(
        x1: f32,
        y1: f32,
        x2: f32,
        y2: f32,
        x3: f32,
        y3: f32,
        color: Color,
    ) {
        draw_triangle(
            vec2(Self::x(x1), Self::y(y1)),
            vec2(Self::x(x2), Self::y(y2)),
            vec2(Self::x(x3), Self::y(y3)),
            color,
        );
    }

    pub fn polygon(
        x: f32,
        y: f32,
        sides: u8,
        radius: f32,
        rotation: f32,
        color: Color,
    ) {
        if sides < 3 {
            return;
        }

        let center = vec2(Self::x(x), Self::y(y));
        let radius_x = Self::width(radius);
        let radius_y = Self::height(radius);

        let mut previous = vec2(
            center.x + radius_x * rotation.to_radians().cos(),
            center.y + radius_y * rotation.to_radians().sin(),
        );

        for i in 1..=sides {
            let angle =
                (rotation + i as f32 * 360.0 / sides as f32).to_radians();

            let next = vec2(
                center.x + radius_x * angle.cos(),
                center.y + radius_y * angle.sin(),
            );

            draw_triangle(center, previous, next, color);
            previous = next;
        }
    }

    pub fn polygon_outline(
        x: f32,
        y: f32,
        sides: u8,
        radius: f32,
        rotation: f32,
        color: Color,
        thickness: f32,
    ) {
        if sides < 3 {
            return;
        }

        let center = vec2(Self::x(x), Self::y(y));
        let radius_x = Self::width(radius);
        let radius_y = Self::height(radius);
        let thickness = Self::thickness(thickness);

        let mut previous = vec2(
            center.x + radius_x * rotation.to_radians().cos(),
            center.y + radius_y * rotation.to_radians().sin(),
        );

        for i in 1..=sides {
            let angle =
                (rotation + i as f32 * 360.0 / sides as f32).to_radians();

            let next = vec2(
                center.x + radius_x * angle.cos(),
                center.y + radius_y * angle.sin(),
            );

            draw_line(
                previous.x,
                previous.y,
                next.x,
                next.y,
                thickness,
                color,
            );

            previous = next;
        }
    }

    // ---------------------------------------------------------
    // ARCS
    // ---------------------------------------------------------

    pub fn arc(
        x: f32,
        y: f32,
        radius: f32,
        start_degrees: f32,
        end_degrees: f32,
        color: Color,
        thickness: f32,
    ) {
        let start = start_degrees.to_radians();
        let end = end_degrees.to_radians();

        let segments = (((end - start).abs() / 0.08).ceil() as usize)
            .clamp(1, 180);

        let mut previous_angle = start;

        for segment in 1..=segments {
            let angle = start
                + (end - start) * segment as f32 / segments as f32;

            draw_line(
                Self::x(x + radius * previous_angle.cos()),
                Self::y(y + radius * previous_angle.sin()),
                Self::x(x + radius * angle.cos()),
                Self::y(y + radius * angle.sin()),
                Self::thickness(thickness),
                color,
            );

            previous_angle = angle;
        }
    }

    // ---------------------------------------------------------
    // ELLIPSES
    // ---------------------------------------------------------

    pub fn ellipse(
        x: f32,
        y: f32,
        width: f32,
        height: f32,
        rotation: f32,
        color: Color,
    ) {
        draw_ellipse(
            Self::x(x),
            Self::y(y),
            Self::width(width / 2.0),
            Self::height(height / 2.0),
            rotation,
            color,
        );
    }

    pub fn ellipse_outline(
        x: f32,
        y: f32,
        width: f32,
        height: f32,
        rotation: f32,
        color: Color,
        thickness: f32,
    ) {
        draw_ellipse_lines(
            Self::x(x),
            Self::y(y),
            Self::width(width / 2.0),
            Self::height(height / 2.0),
            rotation,
            Self::thickness(thickness),
            color,
        );
    }

    // ---------------------------------------------------------
    // TEXT
    // ---------------------------------------------------------

    pub fn text(
        value: &str,
        x: f32,
        y: f32,
        size: f32,
        color: Color,
    ) {
        draw_text(
            value,
            Self::x(x),
            Self::y(y),
            Self::thickness(size),
            color,
        );
    }

    // ---------------------------------------------------------
    // GRADIENTS
    // ---------------------------------------------------------

    pub fn gradient_vertical(start: Color, end: Color) {
        Self::gradient(start, end, false);
    }

    pub fn gradient_horizontal(start: Color, end: Color) {
        Self::gradient(start, end, true);
    }

    fn gradient(start: Color, end: Color, horizontal: bool) {
        let width = screen_width();
        let height = screen_height();

        let strips = 128;
        let strip_length = if horizontal { width } else { height };
        let strip_size = strip_length / strips as f32;

        for index in 0..strips {
            let amount = index as f32 / (strips - 1) as f32;

            let color = Color::new(
                start.r + (end.r - start.r) * amount,
                start.g + (end.g - start.g) * amount,
                start.b + (end.b - start.b) * amount,
                start.a + (end.a - start.a) * amount,
            );

            let position = index as f32 * strip_size;
            let remaining = strip_length - position;
            let size = strip_size.min(remaining);

            if horizontal {
                draw_rectangle(position, 0.0, size, height, color);
            } else {
                draw_rectangle(0.0, position, width, size, color);
            }
        }
    }

    // ---------------------------------------------------------
    // KEYBOARD INPUT
    // ---------------------------------------------------------

    pub fn key_down(key: &str) -> bool {
        match key.to_uppercase().as_str() {
            "UP" => is_key_down(KeyCode::Up),
            "DOWN" => is_key_down(KeyCode::Down),
            "LEFT" => is_key_down(KeyCode::Left),
            "RIGHT" => is_key_down(KeyCode::Right),
            "SPACE" => is_key_down(KeyCode::Space),
            "ENTER" => is_key_down(KeyCode::Enter),
            "ESCAPE" => is_key_down(KeyCode::Escape),
            "W" => is_key_down(KeyCode::W),
            "A" => is_key_down(KeyCode::A),
            "S" => is_key_down(KeyCode::S),
            "D" => is_key_down(KeyCode::D),
            _ => false,
        }
    }

    pub fn key_pressed(key: &str) -> bool {
        match key.to_uppercase().as_str() {
            "UP" => is_key_pressed(KeyCode::Up),
            "DOWN" => is_key_pressed(KeyCode::Down),
            "LEFT" => is_key_pressed(KeyCode::Left),
            "RIGHT" => is_key_pressed(KeyCode::Right),
            "SPACE" => is_key_pressed(KeyCode::Space),
            "ENTER" => is_key_pressed(KeyCode::Enter),
            "ESCAPE" => is_key_pressed(KeyCode::Escape),
            "W" => is_key_pressed(KeyCode::W),
            "A" => is_key_pressed(KeyCode::A),
            "S" => is_key_pressed(KeyCode::S),
            "D" => is_key_pressed(KeyCode::D),
            _ => false,
        }
    }

    // ---------------------------------------------------------
    // MOUSE INPUT
    // ---------------------------------------------------------

    fn mouse_button(button: &str) -> Result<MouseButton, String> {
        match button.to_ascii_uppercase().as_str() {
            "LEFT" => Ok(MouseButton::Left),
            "RIGHT" => Ok(MouseButton::Right),
            "MIDDLE" => Ok(MouseButton::Middle),
            _ => Err(format!("Unknown mouse button '{}'", button)),
        }
    }

    pub fn mouse_down(button: &str) -> Result<bool, String> {
        Ok(is_mouse_button_down(Self::mouse_button(button)?))
    }

    pub fn mouse_pressed(button: &str) -> Result<bool, String> {
        Ok(is_mouse_button_pressed(Self::mouse_button(button)?))
    }

    pub fn mouse_x() -> f32 {
        mouse_position().0 / Self::scale_x()
    }

    pub fn mouse_y() -> f32 {
        mouse_position().1 / Self::scale_y()
    }

    // ---------------------------------------------------------
    // TIME AND SCREEN INFORMATION
    // ---------------------------------------------------------

    pub fn delta_time() -> f32 {
        get_frame_time()
    }

    pub fn time() -> f64 {
        get_time()
    }

    pub fn fps() -> i32 {
        get_fps()
    }

    // These return the logical RUSK design dimensions.
    pub fn screen_width() -> f32 {
        Self::reference_width()
    }

    pub fn screen_height() -> f32 {
        Self::reference_height()
    }
}

// -------------------------------------------------------------
// TESTS
// -------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_hex_rgb_and_rgba_colors() {
        let rgb = Graphics::parse_color("#8B5CF6").unwrap();
        let rgba = Graphics::parse_color("#12122580").unwrap();

        assert_eq!(rgb, Color::from_rgba(139, 92, 246, 255));
        assert_eq!(rgba, Color::from_rgba(18, 18, 37, 128));
    }

    #[test]
    fn parses_named_colors() {
        assert_eq!(
            Graphics::parse_color("red").unwrap(),
            RED
        );

        assert_eq!(
            Graphics::parse_color("blue").unwrap(),
            BLUE
        );

        assert_eq!(
            Graphics::parse_color("transparent").unwrap(),
            Color::from_rgba(0, 0, 0, 0)
        );
    }

    #[test]
    fn rejects_invalid_colors_and_channels() {
        assert!(Graphics::parse_color("#12GG00").is_err());
        assert!(Graphics::rgb(256.0, 0.0, 0.0).is_err());
        assert!(Graphics::rgba(0.0, 0.0, 0.0, f64::NAN).is_err());
    }

    #[test]
    fn rejects_fractional_color_channels() {
        assert!(Graphics::rgb(10.5, 20.0, 30.0).is_err());
    }
}

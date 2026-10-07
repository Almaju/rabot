struct Canvas {
    dpi: Dpi,
    height: Height,
    margin: Margin,
    width: Width,
}

struct Style {
    background: Color,
    color: Color,
    font: Font,
}

impl Renderer {
    fn render(&self, title: &str, canvas: &Canvas, style: &Style) -> Image {
        self.blank(canvas)
            .fill(style.background)
            .text(title, &style.font, style.color, canvas.margin)
    }
}

impl Renderer {
    fn render(
        &self,
        title: &str,
        width: Width,
        height: Height,
        dpi: Dpi,
        margin: Margin,
        font: &Font,
        color: Color,
        background: Color,
    ) -> Image {
        self.blank(width, height, dpi)
            .fill(background)
            .text(title, font, color, margin)
    }
}

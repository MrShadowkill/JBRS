use renderer_gl21::Renderer;
// Antialiased DejaVu font atlas embedded in the renderer; license is bundled.
pub fn text(r: &mut Renderer, x: f32, y: f32, s: f32, t: &str, c: [f32; 3]) {
    r.label(x, y, s, t, c);
}

pub fn button(r: &mut Renderer, rect: [f32; 4], label: &str, color: [f32; 3]) {
    r.panel(rect, color, 0.94, 14.);
    text(
        r,
        rect[0] + 16.,
        rect[1] + rect[3] * 0.3,
        2.4,
        label,
        [1., 1., 1.],
    );
}
pub fn contains(rect: [f32; 4], x: f32, y: f32) -> bool {
    x >= rect[0] && x <= rect[0] + rect[2] && y >= rect[1] && y <= rect[1] + rect[3]
}

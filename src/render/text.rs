use crate::creatures::AnimState;

pub fn render_text(name: &str, emoji: &str, state: &AnimState) -> String {
    format!("{}{} [{}]", emoji, name, state)
}

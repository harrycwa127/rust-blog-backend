use slug;

pub fn slugify(text: &str) -> String {
    slug::slugify(text)
}
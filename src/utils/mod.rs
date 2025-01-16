use mime_guess::MimeGuess;

pub const DATABASE_NAME: &str = "post-archiver.db";
pub const TEMPLATE_DATABASE_UP_SQL: &str = include_str!("template.up.sql");
pub const TEMPLATE_DATABASE_DOWN_SQL: &str = include_str!("template.down.sql");

pub fn get_mime(filename: &str) -> String {
    let guess = MimeGuess::from_path(filename);
    let mime = guess.first_or_text_plain();
    let mime = mime.to_string();

    mime
}

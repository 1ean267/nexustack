#[derive(Clone, Debug)]
pub struct Note {
    /// The unique identifier of the note.
    pub id: u32,

    /// The title of the note.
    pub title: String,

    /// The content of the note.
    pub content: String,

    /// Whether the note is published.
    pub is_published: bool,

    /// The creation timestamp of the note.
    pub created_at: chrono::DateTime<chrono::Utc>,

    /// The last updated timestamp of the note.
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

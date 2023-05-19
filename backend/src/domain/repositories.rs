use crate::domain::entities::Bookmark;

trait BookmarkRepository {
    fn save(&self, bookmark: Bookmark);
}

use super::SearchContext;

impl<'a> SearchContext<'a> {
    pub const fn extensions(_in_check: bool, _extension_count: i16) -> i16 {
        0
    }
}

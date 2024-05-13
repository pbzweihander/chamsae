use crate::entity::sea_orm_active_enums::Visibility;

impl Visibility {
    pub fn is_visible(&self) -> bool {
        matches!(self, Self::Public | Self::Home)
    }
}

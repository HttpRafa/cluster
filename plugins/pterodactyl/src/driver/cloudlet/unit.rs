use common::name::TimedName;

#[derive(Clone)]
pub struct PanelUnit {
    pub id: u32,
    pub identifier: String,
    pub name: TimedName,
}

impl PanelUnit {
    pub fn new(id: u32, identifier: String, name: TimedName) -> Self {
        Self {
            id,
            identifier,
            name,
        }
    }
}

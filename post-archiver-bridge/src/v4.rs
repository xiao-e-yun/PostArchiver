use crate::MigrationDatabase;

#[derive(Debug, Clone, Default)]
pub struct Bridge;

impl MigrationDatabase for Bridge {
    const VERSION: &'static str = "0.3";
    const SQL: &'static str = "
    
    ";

    fn upgrade(&mut self, path: &Path) {}
}

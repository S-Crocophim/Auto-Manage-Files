use std::sync::Arc;

use crate::database::Database;
use crate::file_ops::FileService;
use crate::watcher::WatcherManager;

#[derive(Clone)]
pub struct AppState {
    pub database: Arc<Database>,
    pub files: Arc<FileService>,
    pub watcher: Arc<WatcherManager>,
}

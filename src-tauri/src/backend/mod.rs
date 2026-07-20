pub(crate) mod app_update;
pub(crate) mod appearance;
pub(crate) mod backup;
pub(crate) mod config_editor;
pub(crate) mod core;
pub(crate) mod dictionaries;
pub(crate) mod downloads;
pub(crate) mod http;
pub(crate) mod phrases;
pub(crate) mod proxy;
pub(crate) mod schemas;
pub(crate) mod settings;
pub(crate) mod system;

#[cfg(test)]
#[path = "tests.rs"]
mod integration_tests;

// Re-export for convenience across sibling modules
pub(crate) use self::app_update::*;
pub(crate) use self::appearance::*;
pub(crate) use self::backup::*;
pub(crate) use self::config_editor::*;
pub(crate) use self::core::*;
pub(crate) use self::dictionaries::*;
pub(crate) use self::downloads::*;
pub(crate) use self::http::*;
pub(crate) use self::phrases::*;
pub(crate) use self::proxy::*;
pub(crate) use self::schemas::*;
pub(crate) use self::settings::*;
pub(crate) use self::system::*;

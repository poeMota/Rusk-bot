use crate::prelude::*;
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use serenity::all::RoleId;
use std::{collections::HashMap, sync::Arc};
use tokio::sync::RwLock;

pub static ROLEMANAGER: Lazy<Arc<RwLock<RoleManager>>> =
    Lazy::new(|| Arc::new(RwLock::new(RoleManager::new())));

#[derive(Debug, Deserialize, Serialize)]
pub struct RoleManager {
    #[serde(rename = "ChangeRolePermissions")]
    change_role_permissions: HashMap<RoleId, Vec<RoleId>>,
}

impl Default for RoleManager {
    fn default() -> Self {
        Self {
            change_role_permissions: HashMap::new(),
        }
    }
}

impl RoleManager {
    fn new() -> Self {
        let content = read_file(&DATA_PATH.join("role_manager_config.toml"));
        toml::from_str(&content).unwrap_or(Self::default())
    }

    async fn write_data(&self) {
        write_file(
            &DATA_PATH.join("role_manager_config.toml"),
            match toml::to_string(&self) {
                Ok(c) => c,
                Err(e) => {
                    Logger::error("role_man.serialize", e.to_string().as_str()).await;
                    return;
                }
            },
        );
    }

    pub async fn set_permissions(&mut self, role: RoleId, role_permissions: Vec<RoleId>) {
        Logger::high(
            "role_man.set_permissions",
            &format!(
                "permissions for role {} setted to {:?}",
                role, role_permissions
            ),
        )
        .await;

        self.change_role_permissions.insert(role, role_permissions);
        self.write_data().await;
    }

    pub fn have_permission(&self, member: &Member, role_permission: RoleId) -> bool {
        for role in member.roles.iter() {
            if let Some(permissions) = self.change_role_permissions.get(&role) {
                if permissions.contains(&role_permission) {
                    return true;
                }
            }
        }

        false
    }

    pub fn get_permissons(&self, role: RoleId) -> Option<&Vec<RoleId>> {
        self.change_role_permissions.get(&role)
    }
}

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
    #[serde(rename = "SaveDBPermissions")]
    save_db_permissions: HashMap<String, Vec<RoleId>>,
}

impl Default for RoleManager {
    fn default() -> Self {
        Self {
            change_role_permissions: HashMap::new(),
            save_db_permissions: HashMap::new(),
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

    pub async fn set_role_permissions(&mut self, role: RoleId, permissions: Vec<RoleId>) {
        Logger::high(
            "role_man.set_role_perms",
            &format!("permissions for role {} setted to {:?}", role, permissions),
        )
        .await;

        self.change_role_permissions.insert(role, permissions);
        self.write_data().await;
    }

    pub fn have_role_permission(&self, member: &Member, permission: RoleId) -> bool {
        for role in member.roles.iter() {
            if let Some(perms) = self.change_role_permissions.get(&role) {
                if perms.contains(&permission) {
                    return true;
                }
            }
        }

        false
    }

    pub fn get_role_permissons(&self, role: RoleId) -> Option<&Vec<RoleId>> {
        self.change_role_permissions.get(&role)
    }

    pub async fn set_db_permissions(&mut self, db: String, permissions: Vec<RoleId>) {
        Logger::high(
            "role_man.set_db_perms",
            &format!("permissions for DB {} setted to {:?}", db, permissions),
        )
        .await;

        self.save_db_permissions.insert(db, permissions);
        self.write_data().await;
    }

    pub fn have_db_permission(&self, member: &Member, db: &String) -> bool {
        for role in self.save_db_permissions.get(db).unwrap_or(&Vec::new()) {
            if member.roles.contains(role) {
                return true;
            }
        }

        false
    }

    pub fn member_db_permissons(&self, member: &Member) -> Vec<&String> {
        let mut dbs = Vec::new();

        for (db, perms) in self.save_db_permissions.iter() {
            for role in perms {
                if member.roles.contains(role) {
                    dbs.push(db);
                    break;
                }
            }
        }

        dbs
    }

    pub fn get_db_permissions(&self, db: &String) -> Option<&Vec<RoleId>> {
        self.save_db_permissions.get(db)
    }

    pub fn get_dbs(&self) -> Vec<&String> {
        self.save_db_permissions.keys().collect()
    }
}

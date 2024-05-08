use reqwest::Body;
use serde::{Deserialize, Serialize};
use std::fmt::Display;

use crate::common::config::CONF;

#[derive(Serialize, Deserialize, Debug)]
pub struct AuthPayload {
    pub auth: Auth,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Auth {
    pub identity: Identity,
    pub scope: Scope,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Identity {
    pub methods: Vec<String>,
    pub password: Password,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Password {
    pub user: User,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct User {
    pub name: String,
    pub password: String,
    pub domain: Domain,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Domain {
    pub name: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Scope {
    pub project: Project,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Project {
    pub domain: Domain,
    pub name: String,
}

impl AuthPayload {
    pub fn new(name: String, password: String) -> Self {
        Self {
            auth: Auth {
                identity: Identity {
                    methods: vec!["password".to_string()],
                    password: Password {
                        user: User {
                            name,
                            password,
                            domain: Domain {
                                name: CONF::global().openstack.domain.to_string(),
                            },
                        },
                    },
                },
                scope: Scope {
                    project: Project {
                        domain: Domain {
                            name: CONF::global().openstack.domain.to_string(),
                        },
                        name: CONF::global().openstack.project.to_string(),
                    },
                },
            },
        }
    }
}

impl Display for AuthPayload {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", serde_json::to_string(&self).unwrap().to_string())
    }
}

impl Into<Body> for AuthPayload {
    fn into(self) -> Body {
        Body::from(serde_json::to_string(&self).unwrap_or_default())
    }
}

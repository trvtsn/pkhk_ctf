/// src/server/backend.rs
/// 
/// Even though the server is technically the backend, I feel like all the code here
/// deserves to be in its own file, and not combined with `src/server/mod.rs`, as it deals
/// with stuff that's more specific to authentication, identity, and session handling, a 
/// whole group of categories that is better categorized into one, backend.rs.
/// It's more distinct from the rest of the server, and should also be looked at from a 
/// more security-sensitive side.

use argon2::{Argon2, PasswordHash, PasswordVerifier, password_hash};
use argon2::PasswordHasher;
#[cfg(feature = "ssr")]
use axum_login::{AuthnBackend, AuthUser, UserId};
use cfg_if::cfg_if;
#[cfg(feature = "ssr")]
use ldap3::{LdapConnAsync, LdapConnSettings, SearchEntry};
#[cfg(feature = "ssr")]
use of_dn_parser::{DistinguishedName, RdnType};
use password_hash::SaltString;
#[cfg(feature = "ssr")]
use password_hash::rand_core::OsRng;
#[cfg(feature = "ssr")]
use rand::{rngs::SmallRng, Rng, SeedableRng};
use zeroize::Zeroizing;
use std::{collections::HashSet, str::FromStr};
use tracing::instrument;

#[cfg(feature = "ssr")]
use crate::server::backend::enums::AuthType;
#[cfg(feature = "ssr")]
use crate::server::{BroadcastScope, build_and_broadcast, is_host_reachable};
use crate::server::{ServerEventPayload, db::structs::{Attachment, LdapArgs}};

#[cfg(feature = "ssr")]
pub type AuthSession = axum_login::AuthSession<Backend>;

#[cfg(feature = "ssr")]
pub mod structs {
    use sqlx::MySqlPool;

    #[derive(Debug, Clone)]
    pub struct Backend {
        pub pool: MySqlPool
    }
}

pub mod enums {
    use serde::{Deserialize, Serialize};

    #[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
    #[serde(rename_all = "camelCase")]
    pub enum AuthType {
        Normal,
        Ldap
    }

    impl std::fmt::Display for AuthType {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            let s = match self {
                AuthType::Normal => "normal",
                AuthType::Ldap => "ldap",
            };
            write!(f, "{s}")
        }
    }
}

cfg_if! {
    if #[cfg(feature = "ssr")] {
        use crate::{error_template::AppError, server::{backend::structs::Backend, db::{enums::{UserIdentifier, UserRole}, structs::DbUser}, structs::{Credentials, User}}};
        use sqlx::MySqlPool;

        impl AuthUser for User {
            type Id = String;
            fn id(&self) -> Self::Id {
                self.id.clone()
            }
            fn session_auth_hash(&self) -> &[u8] {
                self.session_auth_hash.as_ref()
            }
        }

        impl Backend {
            pub fn new(pool: MySqlPool) -> Self {
                Self { pool }
            }

            /// Insert a new user into the database. Success only if the user doesn't already exist
            /// and the data meets criteria (which are *very* weak in this example!).
            pub async fn add_user(&self, email: &str, password: Zeroizing<String>) -> Result<Option<User>, AppError> {
                // First validate the data. You must do better than this.
                if email.len() < 2 || password.len() < 10 {
                    return Err(AppError::InvalidData("Username and password have to be at least 2 characters each!".into()));
                }

                let pw_hash_str = tokio::task::spawn_blocking(move || {
                    let argon2 = Argon2::default();
                    let salt = SaltString::generate(&mut OsRng);
                    let hash = argon2.hash_password(password.as_bytes(), &salt)?;
                    Ok::<String, password_hash::Error>(hash.to_string())
                }).await.map_err(|e| AppError::InternalError(e.to_string()))?
                  .map_err(|e| AppError::InternalError(format!("Password hashing error: {e}")))?;

                let username_prefix = email.split_once("@").map(|(l, _)| l).unwrap_or(email);
                let taken = DbUser::get_taken_usernames(username_prefix, &self.pool)
                    .await?
                    .into_iter()
                    .collect::<HashSet<String>>();

                let mut rng = SmallRng::from_os_rng();
                let mut username = String::new();
                for _ in 0..100 {
                    let candidate = format!("{username_prefix}{}", rng.random_range(1000..9999));
                    if !taken.contains(&candidate) {
                        username = candidate;
                        break;
                    }
                }
                if username.is_empty() {
                    return Err(AppError::InternalError("Could not generate unique username".to_string()));
                }

                let new_user = DbUser { 
                    id: "".to_string(), 
                    username, 
                    email: email.to_string(), 
                    pw_hash: pw_hash_str.clone(), 
                    created_at: chrono::Local::now(), 
                    last_active_at: chrono::Local::now(), 
                    role: UserRole::Competitor,
                    points: 0,
                    groups: "unassigned".to_string(),
                    auth_type: "normal".to_string()
                };
                let new_user_id = new_user.add(&self.pool).await?;

                let pw_hash = PasswordHash::parse(&pw_hash_str, password_hash::Encoding::B64).map_err(|e| 
                    AppError::InternalError(format!("Decode password: {e}"))
                )?;

                if let Some(hash_bytes) = pw_hash.hash {
                    Ok(Some(
                        User {
                            id: new_user_id,
                            session_auth_hash: hash_bytes.as_bytes().to_owned(),
                        }
                    ))
                } else {
                    Err(AppError::InternalError("Password hash/digest empty?".to_string()))
                }

            }
        }

        //#[async_trait]
        impl AuthnBackend for Backend {
            type User = User;
            type Credentials = Credentials;
            type Error = AppError;

            #[instrument(skip(creds))]
            async fn authenticate(&self, creds: Self::Credentials) -> Result<Option<Self::User>, Self::Error> {
                let user = match creds.auth_type {
                    AuthType::Normal => {
                        match DbUser::get(&creds.user_identifier, &self.pool).await {
                            Ok(Some(user)) => user,
                            Ok(None) => return Ok(None),
                            Err(e) => return Err(e.into())
                        }
                    },
                    AuthType::Ldap => {
                        crate::server::proxmox::sync_realm().await?;
                        
                        let ldap_args = match LdapArgs::get(&self.pool).await {
                            Ok(Some(args)) => args,
                            Ok(None) => return Ok(None),
                            Err(e) => return Err(e.into())
                        };
                        let ldap_url = url::Url::parse(&ldap_args.url)?;
                        is_host_reachable(&ldap_url.to_string()).await?;

                        let login_id = match &creds.user_identifier {
                            UserIdentifier::Email(e) => ldap3::ldap_escape(e.clone()).to_string(),
                            UserIdentifier::Username(u) => ldap3::ldap_escape(u.clone()).to_string(),
                            _ => return Ok(None),
                        };
                        let mut email = String::new();
                        let mut username = String::new();
                        let mut groups_result = String::new();

                        let certificate = match Attachment::get_certificate(&self.pool).await {
                            Ok(cert) => cert,
                            Err(e) => return Err(e.into())
                        };

                        #[allow(unused)]
                        let mut settings = LdapConnSettings::default();
                        if let Some(cert) = certificate && ldap_url.scheme() == "ldaps" {
                            let cert = native_tls::Certificate::from_pem(&cert.file_blob)?;
                            let connector = native_tls::TlsConnector::builder().add_root_certificate(cert).build()?;
                            settings = LdapConnSettings::new().set_connector(connector);
                        } else {
                            settings = LdapConnSettings::new().set_no_tls_verify(true).set_starttls(false);
                        }

                        let (conn, mut ldap) = LdapConnAsync::with_settings(settings, ldap_url.as_str()).await?;
                        ldap3::drive!(conn);

                        // Bind with service account first
                        match ldap.simple_bind(ldap_args.bind_dn.as_str(), ldap_args.bind_pw.as_str()).await {
                            Ok(res) => if res.success().is_err() {
                                ldap.unbind().await?;
                                return Ok(None)
                            },
                            Err(e) => {
                                ldap.unbind().await?;
                                return Err(e.into())
                            }
                        };

                        // Search by either userPrincipalName or sAMAccountName
                        let filter = format!("(|(userPrincipalName={})(sAMAccountName={}))", login_id, login_id);
                        let attrs = vec!["distinguishedName", "userPrincipalName", "sAMAccountName", "memberOf"];
                        let (entries, _res) = ldap.search(ldap_args.base_dn.as_str(), ldap3::Scope::Subtree, &filter, attrs).await?.success()?;
                        if entries.is_empty() {
                            ldap.unbind().await?;
                            return Ok(None);
                        }

                        let mut user_dn = String::new();
                        if let Some(entry) = entries.into_iter().next() {
                            let se = SearchEntry::construct(entry);

                            user_dn = se.dn;
                            username = se.attrs.get("sAMAccountName").and_then(|v| v.get(0)).cloned().unwrap_or_default();
                            email = se.attrs.get("userPrincipalName").and_then(|v| v.get(0)).cloned().unwrap_or_default();
                            if let Some(groups) = se.attrs.get("memberOf") {
                                groups_result = groups.into_iter().filter_map(|s| {
                                    DistinguishedName::from_str(s).ok().and_then(|dn| dn.find(RdnType::Cn).map(String::from))
                                })
                                .collect::<Vec<String>>()
                                .join(",");
                            }
                        }

                        // Re-bind as the user to verify their password
                        match ldap.simple_bind(user_dn.as_str(), creds.password.as_str()).await {
                            Ok(res) => if res.success().is_err() {
                                ldap.unbind().await?;
                                return Ok(None)
                            },
                            Err(e) => {
                                ldap.unbind().await?;
                                return Err(e.into())
                            }
                        };

                        ldap.unbind().await?;
                        let pw_hash = hash_string(&creds.password).await?;

                        if let Ok(None) = DbUser::get_ldap(&creds.user_identifier, &self.pool).await {
                            let mut tx = self.pool.begin().await?;
                            let new_user = DbUser { 
                                id: "".to_string(), 
                                username, 
                                email, 
                                pw_hash, 
                                created_at: chrono::Local::now(), 
                                last_active_at: chrono::Local::now(), 
                                role: UserRole::Competitor,
                                points: 0,
                                groups: groups_result,
                                auth_type: "ldap".to_string()
                            };

                            let new_user_id = match new_user.add_ldap(&mut *tx).await {
                                Ok(id) => id,
                                Err(e) => {
                                    tx.rollback().await?;
                                    return Err(e.into());
                                }
                            };
                            
                            match DbUser::get_ldap(&UserIdentifier::Id(new_user_id), &mut *tx).await {
                                Ok(Some(user)) => {
                                    tx.commit().await?;
                                    let broadcast_user = user.clone();
                                    tokio::spawn(async move {
                                        _ = build_and_broadcast(ServerEventPayload::UserCreated(broadcast_user), vec![BroadcastScope::Admin]).await;
                                    });
                                    user
                                },
                                Ok(None) => {
                                    tx.rollback().await?;
                                    return Ok(None)
                                },
                                Err(e) => {
                                    tx.rollback().await?;
                                    return Err(e.into())
                                }
                            }
                        } else if let Ok(Some(user)) = DbUser::get_ldap(&creds.user_identifier, &self.pool).await {
                            user
                        } else {
                            return Ok(None)
                        }
                    }
                };

                if let Ok(()) = verify_hash(&creds.password, &user.pw_hash).await {
                    Ok(Some(user.to_user().await?))
                } else {
                    Ok(None)
                }
            }

            #[instrument]
            async fn get_user(&self, user_id: &UserId<Self>) -> Result<Option<Self::User>, Self::Error> {
                match DbUser::get(&UserIdentifier::Id(user_id.clone()), &self.pool).await {
                    Ok(Some(user)) => Ok(Some(user.to_user().await?)),
                    Ok(None) => Err(Self::Error::InternalError("User not found".to_string())),
                    Err(e) => Err(Self::Error::DatabaseError(e.to_string()))
                }
            }
        }


        impl DbUser {
            #[instrument]
            pub async fn to_user(self) -> anyhow::Result<User> {
                // parse the hash data out of the string representation that we kept in the database
                let PasswordHash {hash, ..} = PasswordHash::parse(&self.pw_hash, password_hash::Encoding::B64).map_err(|e| AppError::InternalError(format!("Decode password: {e}")))?;
                // This is where we dig into the password hash data structure and pull out just
                // the actual hash bytes that came out of argon2. These are used to identify the session
                // so that this user always gets the same session data.
                let hash: Vec<u8> = hash.map(|output| {
                    output.as_bytes().to_owned()
                }).ok_or_else(||AppError::InternalError("Badly formatted password hash".into()))?;
                
                Ok(User {
                    id: self.id,
                    session_auth_hash: hash,
                })
            }
        }

        pub async fn hash_string(string: &str) -> Result<String, AppError> {
            let string = string.to_owned();
            tokio::task::spawn_blocking(move || {
                let argon2 = Argon2::default();
                let salt = argon2::password_hash::SaltString::generate(&mut OsRng);
                let flag_hash = argon2.hash_password(string.as_bytes(), &salt)?;
                Ok::<String, argon2::password_hash::Error>(flag_hash.to_string())
            }).await.map_err(|e| AppError::InternalError(e.to_string()))?
              .map_err(|e| AppError::InternalError(e.to_string()))
        }

        pub async fn verify_hash(string: &str, hash: &str) -> Result<(), AppError> {
            let string = string.to_owned();
            let hash = hash.to_owned();
            tokio::task::spawn_blocking(move || {
                let hasher = Argon2::default();
                let parsed = argon2::PasswordHash::parse(hash.as_ref(), argon2::password_hash::Encoding::B64)?;
                hasher.verify_password(string.as_bytes(), &parsed)
            }).await.map_err(|e| AppError::InternalError(e.to_string()))?
              .map_err(|e| AppError::InternalError(e.to_string()))
        }

    }
}


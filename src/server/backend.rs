use argon2::{Argon2, PasswordHash, PasswordVerifier, password_hash};
use argon2::PasswordHasher;
#[cfg(feature = "ssr")]
use axum_login::{AuthnBackend, AuthUser, UserId};
use cfg_if::cfg_if;
use password_hash::SaltString;
#[cfg(feature = "ssr")]
use password_hash::rand_core::OsRng;
#[cfg(feature = "ssr")]
use rand::{rngs::SmallRng, Rng, SeedableRng};
use time::OffsetDateTime;
use tracing::instrument;

#[cfg(feature = "ssr")]
pub type AuthSession = axum_login::AuthSession<Backend>;

#[cfg(feature = "ssr")]
pub mod structs {
    use crate::server::db::enums::UserIdentifier;
    use serde::{Deserialize, Serialize};
    #[cfg(feature = "ssr")]
    use sqlx::MySqlPool;

    #[derive(Debug, Clone)]
    pub struct Backend {
        pub pool: MySqlPool
    }

    #[derive(Debug, Clone, Deserialize, Serialize)]
    pub struct Credentials {
        pub user_identifier: UserIdentifier,
        pub password: String
    }
}

cfg_if! {
    if #[cfg(feature = "ssr")] {
        use crate::{error_template::AppError, server::{backend::structs::{Backend, Credentials}, db::{enums::{UserIdentifier, UserRole}, structs::DbUser}, structs::User}};
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
            pub async fn add_user(&self, email: String, password: String) -> Result<Option<User>, AppError> {
                // First validate the data. You must do better than this.
                if email.len() < 2 || password.len() < 2 {
                    return Err(AppError::InvalidData("Username and password have to be at least 2 characters each!".into()));
                }
                // Hash the password and insert the new user.
                // This does the hashing
                let argon2 = Argon2::default();
                // The salt is used to prevent certain attacks against stored passwords (see the Internet for more)
                let salt = SaltString::generate(&mut OsRng);
                // This gives back a data structure with various parts, which can be encoded using
                // a standard format into a string that's suitable for use in plain-text environments. Argon2id is the
                // recommended hashing algorithm at the time of this code being published (2024)
                let pw_hash: PasswordHash = argon2.hash_password(password.as_bytes(), &salt)
                    .map_err(|e| AppError::InternalError(format!("Password hashing error: {e}")))?;
                // Now *this* part is what will be put directly into the database as the user's password hash. This is not just
                // the 32-byte hash function output, it also has other data attached (like the salt). It has to have
                // a let-binding outside of the macro or the compiler complains.
                let pw_hash_str = pw_hash.to_string();

                let mut rng = SmallRng::from_os_rng();
                let username_prefix = email.split_once("@").map(|(l, _)| l.to_string()).unwrap_or(email.clone());
                // let username = username_prefix;
                let mut username = "".to_string();
                while username.is_empty() {
                    let username_suffix = rng.random_range(1000..9999);
                    let possible_username = format!("{username_prefix}{username_suffix}");
                    match DbUser::is_username_available(&possible_username, &self.pool).await {
                        Ok(result) => {
                            if result {
                                username = possible_username;
                            } else {
                                continue;
                            }
                        },
                        Err(e) => {
                            tracing::error!("db query error (DbUser::is_username_available): {}", e);
                            continue
                        },
                    }
                }
                let new_user = DbUser { 
                    id: "".to_string(), 
                    username: username.clone(), 
                    email, 
                    pw_hash: pw_hash_str, 
                    created_at: OffsetDateTime::now_utc(), 
                    last_active_at: OffsetDateTime::now_utc(), 
                    role: UserRole::Competitor
                };
                let new_user_id = new_user.add(&self.pool).await?;

                // Now we need to make sure we can make a good session key. In this case, we're using the raw bytes
                // that were output from the password hash (in this case, 32 bytes). This does *not* include the salt
                // or other associated data that's bulit into the pass_hash_str
                let hash_bytes = pw_hash.hash.unwrap().as_bytes().to_owned();
                Ok(Some(
                    User {
                        id: new_user_id,
                        username,
                        session_auth_hash: hash_bytes,
                        role: UserRole::Competitor
                    }
                ))
            }
        }

        //#[async_trait]
        impl AuthnBackend for Backend {
            type User = User;
            type Credentials = Credentials;
            type Error = AppError;

            #[instrument]
            async fn authenticate(&self, creds: Self::Credentials) -> Result<Option<Self::User>, Self::Error> {
                let db_user = match DbUser::get(&creds.user_identifier, &self.pool).await {
                    Ok(Some(user)) => user,
                    Ok(None) => return Ok(None),
                    Err(e) => return Err(Self::Error::DatabaseError(e.to_string()))
                };

                if let Ok(()) = verify_hash(creds.password, db_user.clone().pw_hash) {
                    Ok(Some(db_user.to_user().await?))
                } else {
                    Err(Self::Error::InternalError("wrong pw".to_string()))
                }
            }

            #[instrument]
            async fn get_user(&self, user_id: &UserId<Self>) -> Result<Option<Self::User>, Self::Error> {
                tracing::info!("DbUser::get()");
                match DbUser::get(&UserIdentifier::Id(user_id.clone()), &self.pool).await {
                    Ok(Some(user)) => Ok(Some(user.to_user().await.expect(""))),
                    Ok(None) => Err(Self::Error::InternalError("".to_string())),
                    Err(e) => Err(Self::Error::DatabaseError(e.to_string()))
                }
            }
        }

        // impl AuthzBackend for Backend {
        //     type Permission = Permission;

        //     async fn get_user_permissions(
        //         &self,
        //         _user: &Self::User,
        //     ) -> Result<HashSet<Self::Permission>, Self::Error> {
                
        //     }
        // }

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
                    username: self.username,
                    session_auth_hash: hash,
                    role: self.role
                })
            }
        }

        pub fn hash_string(string: String) -> Result<String, argon2::password_hash::Error> {
            let argon2 = Argon2::default();
            // The salt is used to prevent certain attacks against stored passwords (see the Internet for more)
            let salt = argon2::password_hash::SaltString::generate(&mut OsRng);
            // This gives back a data structure with various parts, which can be encoded using
            // a standard format into a string that's suitable for use in plain-text environments. Argon2id is the
            // recommended hashing algorithm at the time of this code being published (2024)
            let flag_hash= argon2.hash_password(string.as_bytes(), &salt)?;
            // Now *this* part is what will be put directly into the database as the user's password hash. This is not just
            // the 32-byte hash function output, it also has other data attached (like the salt). It has to have
            // a let-binding outside of the macro or the compiler complains.
            Ok(flag_hash.to_string())
        }

        pub fn verify_hash(string: String, hash: String) -> Result<(), argon2::password_hash::Error> {
            let hasher = Argon2::default();
            let hash = argon2::PasswordHash::parse(hash.as_ref(), argon2::password_hash::Encoding::B64)?;
            // Use the existing implementation to verify the password. I was doing this myself until
            // I noticed that there is a PasswordVerifier trait, so this is better in every way.
            match hasher.verify_password(string.as_bytes(), &hash) {
                Ok(_) => Ok(()),
                Err(e) => Err(e)
            }
        }

    }
}


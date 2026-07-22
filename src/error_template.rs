use cfg_if::cfg_if;
use http::status::StatusCode;
use leptos::{prelude::*, server_fn::codec::JsonEncoding};
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Clone, Debug, Error, Serialize, Deserialize, PartialEq)]
pub enum AppError {
    #[error("Not Found")]
    NotFound,
    #[error("InvalidSessionID(\"{0}\")")]
    InvalidSessionId(String),
    #[error("Unauthorized")]
    Unauthorized,
    #[error("InvalidData(\"{0}\")")]
    InvalidData(String),
    #[error("InternalError(\"{0}\")")]
    InternalError(String),
    #[error("DatabaseError(\"{0}\")")]
    DatabaseError(String),
    #[error("ServerFnErrorErr({0})")]
    ServerFnErrorErr(ServerFnErrorErr),
    #[error("ServerFnError({0})")]
    ServerFnError(ServerFnError),
    #[error("NoServerConnection")]
    NoServerConnection,
    #[error("BadRequest(\"{0}\")")]
    BadRequest(String),
    #[error("Forbidden")]
    Forbidden,
    #[error("NetworkError(\"{0}\")")]
    NetworkError(String),
    #[error("PoisonError:(\"{0}\")")]
    PoisonError(String),
}

impl FromServerFnError for AppError {
    type Encoder = JsonEncoding;

    fn from_server_fn_error(value: ServerFnErrorErr) -> Self {
        Self::ServerFnErrorErr(value)
    }
}

impl AppError {
    pub fn status_code(&self) -> StatusCode {
        match self {
            AppError::NotFound => StatusCode::NOT_FOUND,
            AppError::InvalidSessionId(_) => StatusCode::UNAUTHORIZED,
            AppError::Unauthorized => StatusCode::UNAUTHORIZED,
            AppError::InvalidData(_) => StatusCode::NOT_ACCEPTABLE,
            AppError::InternalError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            AppError::DatabaseError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            AppError::ServerFnErrorErr(_) => StatusCode::INTERNAL_SERVER_ERROR,
            AppError::ServerFnError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            AppError::NoServerConnection => StatusCode::INTERNAL_SERVER_ERROR,
            AppError::BadRequest(_) => StatusCode::BAD_REQUEST,
            AppError::Forbidden => StatusCode::FORBIDDEN,
            AppError::NetworkError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            AppError::PoisonError(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

impl From<url::ParseError> for AppError {
    fn from(value: url::ParseError) -> Self {
        tracing::error!(error = ?value, "Failed to parse URL");
        Self::InternalError("Failed to parse URL".to_string())
    }
}

impl From<std::io::Error> for AppError {
    fn from(value: std::io::Error) -> Self {
        tracing::error!(error = ?value, "An error occurred in std::io");
        Self::InternalError("A file operation failed".to_string())
    }
}

impl From<std::num::ParseIntError> for AppError {
    fn from(value: std::num::ParseIntError) -> Self {
        tracing::error!(error = ?value, "Failed to parse integer");
        Self::InternalError("Failed to parse integer".to_string())
    }
}

impl From<serde_json::Error> for AppError {
    fn from(value: serde_json::Error) -> Self {
        tracing::error!(error = ?value, "An error occurred in serde_json");
        Self::InternalError("A parsing error occurred".to_string())
    }
}

impl From<chrono::ParseError> for AppError {
    fn from(value: chrono::ParseError) -> Self {
        tracing::error!(error = ?value, "A ParseError occurred in chrono");
        Self::InternalError("Failed to parse date/time".to_string())
    }
}

cfg_if! {
    if #[cfg(feature = "ssr")] {
        use crate::server::backend::structs::Backend;
        use ldap3::LdapError;

        impl From<sqlx::Error> for AppError {
            fn from(_value: sqlx::Error) -> Self {
                Self::DatabaseError("A database error occurred".to_string())
            }
        }

        impl From<argon2::password_hash::Error> for AppError {
            fn from(error: argon2::password_hash::Error) -> Self {
                tracing::error!(error = ?error, "A hashing error occurred");
                Self::InternalError("A hashing error occurred".to_string())
            }
        }

        impl From<axum_login::Error<Backend>> for AppError {
            fn from(value: axum_login::Error<Backend>) -> Self {
                tracing::error!(error = ?value, "An error occurred in axum_login");
                Self::InternalError("An authentication error occurred".to_string())
            }
        }

        impl From<LdapError> for AppError {
            fn from(value: LdapError) -> Self {
                tracing::error!(error = ?value, "An error occurred in ldap3");
                Self::InternalError("An LDAP error occurred".to_string())
            }
        }

        impl From<reqwest::Error> for AppError {
            fn from(value: reqwest::Error) -> Self {
                tracing::error!(error = ?value, "An error occurred in the reqwest module");
                Self::InternalError("A request to an external service failed".to_string())
            }
        }

        impl From<tokio::task::JoinError> for AppError {
            fn from(value: tokio::task::JoinError) -> Self {
                tracing::error!(error = ?value, "An error occurred in tokio::task");
                Self::InternalError("A background task failed".to_string())
            }
        }

        impl From<native_tls::Error> for AppError {
            fn from(value: native_tls::Error) -> Self {
                tracing::error!(error = ?value, "An error occurred in native_tls");
                Self::InternalError("A secure connection could not be established".to_string())
            }
        }

        impl From<serde_urlencoded::de::Error> for AppError {
            fn from(value: serde_urlencoded::de::Error) -> Self {
                tracing::error!(error = ?value, "An error occurred in serde_urlencoded");
                Self::InternalError("A parsing error occurred".to_string())
            }
        }

        impl<T> From<std::sync::PoisonError<T>> for AppError {
            fn from(value: std::sync::PoisonError<T>) -> Self {
                tracing::error!(error = ?value, "An error occurred in std::sync");
                Self::InternalError("An internal synchronization error occurred".to_string())
            }
        }
    }
}

pub trait LogErr<T, E> {
    /// A method that returns `Result<T, E>`, while also logging the error using `tracing::error!`
    fn log_err(self) -> Result<T, E>;

    /// A method that returns `T::default()` on error, while also logging the error using `tracing::error!`
    fn or_log_and_default(self) -> T where T: Default;
}

impl<T, E: std::fmt::Debug> LogErr<T, E> for Result<T, E> {
    #[track_caller]
    fn log_err(self) -> Result<T, E> {
        if let Err(e) = &self {
            let loc = std::panic::Location::caller();
            tracing::error!(error = ?e, location = %format!("{}:{}", loc.file(), loc.line()));
        }
        self
    }

    #[track_caller]
    fn or_log_and_default(self) -> T where T: Default {
        self.log_err().unwrap_or_default()
    }
}

// A basic function to display errors served by the error boundaries.
// Feel free to do more complicated things here than just displaying the error.
#[component]
pub fn ErrorTemplate(
    #[prop(optional)] outside_errors: Option<Errors>,
    #[prop(optional)] errors: Option<RwSignal<Errors>>,
) -> impl IntoView {
    let errors = match outside_errors {
        Some(e) => RwSignal::new(e),
        None => match errors {
            Some(e) => e,
            None => panic!("No Errors found and we expected errors!"),
        },
    };
    // Get Errors from Signal
    let errors = errors.get_untracked();

    // Downcast lets us take a type that implements `std::error::Error`
    let errors: Vec<AppError> = errors
        .into_iter()
        .filter_map(|(_k, v)| v.downcast_ref::<AppError>().cloned())
        .collect();
    println!("Errors: {errors:#?}");

    // Only the response code for the first error is actually sent from the server
    // this may be customized by the specific application
    #[cfg(feature = "ssr")]
    {
        use leptos_axum::ResponseOptions;
        let response = use_context::<ResponseOptions>();
        if let Some(response) = response {
            response.set_status(errors[0].status_code());
        }
    }

    view! {
        <h1>{if errors.len() > 1 { "Errors" } else { "Error" }}</h1>
        <For
            // a function that returns the items we're iterating over; a signal is fine
            each=move || { errors.clone().into_iter().enumerate() }
            // a unique key for each item as a reference
            key=|(index, _error)| *index
            // renders each item to a view
            children=move |error| {
                let error_string = error.1.to_string();
                let error_code = error.1.status_code();
                view! {
                    <h2>{error_code.to_string()}</h2>
                    <p>"Error: " {error_string}</p>
                }
            }
        />
    }
}

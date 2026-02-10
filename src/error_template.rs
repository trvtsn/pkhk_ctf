use cfg_if::cfg_if;
use http::status::StatusCode;
use leptos::{prelude::*, server_fn::codec::JsonEncoding};
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Clone, Debug, Error, Serialize, Deserialize)]
pub enum AppError {
    #[error("Not Found")]
    NotFound,
    #[error("Invalid Session ID: {0}")]
    InvalidSessionId(String),
    #[error("Unauthorized")]
    Unauthorized,
    #[error("Invalid data provided: {0}")]
    InvalidData(String),
    #[error("Internal Error: {0}")]
    InternalError(String),
    #[error("Database Error: {0}")]
    DatabaseError(String),
    #[error("Anyhow Error: {0}")]
    Anyhow(String),
    #[error("ServerFnErrorErr: {0}")]
    ServerFnErrorErr(ServerFnErrorErr),
    #[error("ServerFnError: {0}")]
    ServerFnError(ServerFnError),
    #[error("No connection with server")]
    NoServerConnection,
    #[error("Bad Request: {0}")]
    BadRequest(String),
    #[error("Forbidden")]
    Forbidden,
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
            AppError::Anyhow(_) => StatusCode::INTERNAL_SERVER_ERROR,
            AppError::ServerFnErrorErr(_) => StatusCode::INTERNAL_SERVER_ERROR,
            AppError::ServerFnError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            AppError::NoServerConnection => StatusCode::INTERNAL_SERVER_ERROR,
            AppError::BadRequest(_) => StatusCode::BAD_REQUEST,
            AppError::Forbidden => StatusCode::FORBIDDEN
        }
    }
}

impl From<anyhow::Error> for AppError {
    fn from(value: anyhow::Error) -> Self {
        AppError::Anyhow(value.to_string())
    }
}

impl From<url::ParseError> for AppError {
    fn from(value: url::ParseError) -> Self {
        Self::InternalError(value.to_string())
    }
}

impl From<std::io::Error> for AppError {
    fn from(value: std::io::Error) -> Self {
        Self::InternalError(value.to_string())
    }
}

impl From<std::num::ParseIntError> for AppError {
    fn from(value: std::num::ParseIntError) -> Self {
        Self::InternalError(value.to_string())
    }
}

impl From<serde_json::Error> for AppError {
    fn from(value: serde_json::Error) -> Self {
        Self::InternalError(value.to_string())
    }
}

impl FromServerFnError for AppError {
    type Encoder = JsonEncoding;

    fn from_server_fn_error(value: ServerFnErrorErr) -> Self {
        Self::ServerFnErrorErr(value)
    }
}

cfg_if! {
    if #[cfg(feature = "ssr")] {
        use crate::server::backend::structs::Backend;
        use ldap3::LdapError;
        
        impl From<sqlx::Error> for AppError {
            fn from(value: sqlx::Error) -> Self {
                Self::DatabaseError(value.to_string())
            }
        }

        impl From<argon2::password_hash::Error> for AppError {
            fn from(error: argon2::password_hash::Error) -> Self {
                Self::InternalError(error.to_string())
            }
        }

        impl From<axum_login::Error<Backend>> for AppError {
            fn from(value: axum_login::Error<Backend>) -> Self {
                Self::InternalError(value.to_string())
            }
        }

        impl From<LdapError> for AppError {
            fn from(value: LdapError) -> Self {
                Self::InternalError(value.to_string())
            }
        }

        impl From<reqwest::Error> for AppError {
            fn from(value: reqwest::Error) -> Self {
                Self::InternalError(value.to_string())
            }
        }

        impl From<tokio::task::JoinError> for AppError {
            fn from(value: tokio::task::JoinError) -> Self {
                Self::InternalError(value.to_string())
            }
        }
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
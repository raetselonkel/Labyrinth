/**
 * Copyright (c) 2022 Oliver Lau <oliver@ersatzworld.net>
 * All rights reserved.
 */
use log;
use serde::Serialize;
use std::convert::Infallible;
use thiserror::Error;
use warp::{http::StatusCode, Rejection, Reply};

#[derive(Error, Debug)]
pub enum Error {
    #[error("mongodb error: {0}")]
    MongoError(#[from] mongodb::error::Error),
    #[error("error during mongodb query: {0}")]
    MongoQueryError(mongodb::error::Error),
    #[error("could not access field in document: {0}")]
    MongoDataError(#[from] bson::document::ValueAccessError),
    #[error("could not parse ObjectID {0}")]
    BsonOidError(#[from] bson::oid::Error),
    #[error("invalid id used: {0}")]
    InvalidIDError(String),
    #[error("data base query error: {0}")]
    DatabaseQueryError(String),
    #[error("script environment error")]
    ScriptEnvironmentError,
    #[error("hashing error")]
    HashingError,
    #[error("password must be at least 8 characters long")]
    PasswordTooShortError,
    #[error("unsafe password")]
    UnsafePasswordError,
    #[error("TOTP key missing error")]
    TotpKeyMissingError,
    #[error("TOTP QR code generation error")]
    TotpQrCodeGenerationError,
    #[error("user not found")]
    UserNotFoundError,
    #[error("username is not valid")]
    InvalidUsernameError,
    #[error("username or email not available")]
    UsernameOrEmailNotAvailableError,
    #[error("combination of username and mail address is not valid")]
    MalformedAddressError,
    #[error("mail address is not valid")]
    InvalidEmailError,
    #[error("building mail failed")]
    MailBuilderError,
    #[error("sending mail failed")]
    SmtpTransportError,
    #[error("user update failed")]
    UserUpdateError,
    #[error("user is no admin")]
    UserIsNoAdminError,
    #[error("riddle not found")]
    RiddleNotFoundError,
    #[error("room not found")]
    RoomNotFoundError,
    #[error("user is in no room")]
    UserIsInNoRoom,
    #[error("riddle has not been seen")]
    RiddleHasNotBeenSeenByUser,
    #[error("user not associated with riddle")]
    UserNotAssociatedWithRiddle,
    #[error("neighbor not found")]
    NeighborNotFoundError,
    #[error("room behind not found")]
    RoomBehindNotFoundError,
    #[error("riddle not solved")]
    RiddleNotSolvedError,
    #[error("wrong credentials")]
    WrongCredentialsError,
    #[error("unsufficient rights")]
    UnsufficentRightsError,
    #[error("cannot change user's role")]
    CannotPromoteUserError,
    #[error("user cannot change own role")]
    UserCannotChangeOwnRoleError,
    #[error("cannot change to same or lower-ranked role")]
    CannotChangeToSameRole,
    #[error("pointless FIDO2")]
    PointlessFido2Error,
    #[error("pointless TOTP")]
    PointlessTotpError,
    #[error("TOTP missing")]
    TotpMissingError,
    #[error("jwt token not valid")]
    JWTTokenError,
    #[error("jwt token creation error")]
    JWTTokenCreationError,
    #[error("no auth header")]
    NoAuthHeaderError,
    #[error("invalid auth header")]
    InvalidAuthHeaderError,
    #[error("no permission")]
    NoPermissionError,
    #[error("cheating is taboo")]
    CheatError,
    #[error("WebAuthn error")]
    WebauthnError,
}

#[derive(Serialize, Debug)]
struct ErrorResponse {
    ok: bool,
    code: u16,
    status: String,
    message: String,
}

impl warp::reject::Reject for Error {}

pub async fn handle_rejection(err: Rejection) -> std::result::Result<impl Reply, Infallible> {
    dbg!(&err);
    let (code, message) = if err.is_not_found() {
        (StatusCode::NOT_FOUND, "Not Found".to_string())
    } else if let Some(e) = err.find::<Error>() {
        match e {
            Error::CheatError => (StatusCode::PAYMENT_REQUIRED, e.to_string()),
            Error::RoomBehindNotFoundError => (StatusCode::CONFLICT, e.to_string()),
            Error::NeighborNotFoundError => (StatusCode::CONFLICT, e.to_string()),
            Error::UnsafePasswordError => (StatusCode::CONFLICT, e.to_string()),
            Error::InvalidEmailError => (StatusCode::CONFLICT, e.to_string()),
            Error::InvalidUsernameError => (StatusCode::CONFLICT, e.to_string()),
            Error::UsernameOrEmailNotAvailableError => (StatusCode::CONFLICT, e.to_string()),
            Error::WrongCredentialsError => (StatusCode::FORBIDDEN, e.to_string()),
            Error::NoPermissionError => (StatusCode::UNAUTHORIZED, e.to_string()),
            Error::JWTTokenError => (StatusCode::UNAUTHORIZED, e.to_string()),
            Error::JWTTokenCreationError => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Internal Server Error".to_string(),
            ),
            _ => (StatusCode::BAD_REQUEST, e.to_string()),
        }
    } else if err
        .find::<warp::filters::body::BodyDeserializeError>()
        .is_some()
    {
        (StatusCode::BAD_REQUEST, "BodyDeserializeError".to_string())
    } else if err.find::<warp::reject::MethodNotAllowed>().is_some() {
        (
            StatusCode::METHOD_NOT_ALLOWED,
            "Method Not Allowed".to_string(),
        )
    } else {
        log::error!("unhandled error: {:?}", err);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            "Internal Server Error".to_string(),
        )
    };
    let json = warp::reply::json(&ErrorResponse {
        ok: false,
        code: code.as_u16(),
        status: code.to_string(),
        message: message,
    });
    Ok(warp::reply::with_status(json, code))
}

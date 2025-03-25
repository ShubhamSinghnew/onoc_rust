use axum::{
    extract::{Json, State},
    http::StatusCode,
    response::IntoResponse,
};
use lettre::transport::smtp::authentication::Credentials;
use lettre::{Message, SmtpTransport, Transport};
use rand::Rng;
use serde::Deserialize;
use serde_json::json;
use std::env;
use std::borrow::Cow;
use crate::AppState;
// Update this import path if needed
use crate::db::users::Otp;
use crate::db::users::User;
use chrono::{Duration, Utc};


#[derive(Deserialize)]
pub struct OTPRequest {
    pub email: String,
    pub mobile: String,
    pub username: String,
}

#[derive(Deserialize)]
pub struct Admin {
    pub id: Option<i32>,  // Or whatever type you're using
    pub email: String,
    pub mobile: String,
    pub username: String,
    pub pincode: String,
    pub regocde: Option<String>,  // Assuming this is optional based on your code
}

#[derive(Deserialize)]
pub struct Admin_Users {
    pub id : Option<i32>,
    pub admin_id : i32,
    pub regcode: Option<String>,
    pub username:  String,
    pub mobile: String,
    pub email: String,
    pub pincode: String
}


#[derive(Deserialize)]
pub struct OtpVerify {
    pub otp: u32,
}

// Using concrete types instead of impl Trait
pub async fn login(
    State(state): State<AppState>,
    Json(payload): Json<OTPRequest>,
) -> impl IntoResponse {
    let pool = &state.pool;
    let OTPRequest {
        email,
        mobile,
        username,
    } = payload;

    if email.is_empty() || mobile.is_empty() {
        return Json(json!({
            "status": "error",
            "message": "Missing email or mobile"
        }));
    }

    let otp = generate_otp();
    println!("Generated OTP: {}", otp);

    let smtp_user = env::var("EMAIL_USER").unwrap_or_else(|_| "default@example.com".to_string());
    let smtp_pass = env::var("EMAIL_PASS").unwrap_or_else(|_| "default_password".to_string());

    let email_message = match Message::builder()
        .from(format!("OTP Service <{}>", smtp_user).parse().unwrap())
        .to(email.parse().unwrap())
        .subject("Your OTP Code")
        .body(format!("Your OTP is: {}", otp))
    {
        Ok(message) => message,
        Err(_) => {
            return Json(json!({
                "status": "error",
                "message": "Failed to create email message"
            }));
        }
    };

    let creds = Credentials::new(smtp_user.clone(), smtp_pass);

    let mailer = match SmtpTransport::relay("smtp.gmail.com") {
        Ok(transport) => transport.credentials(creds).build(),
        Err(_) => {
            return Json(json!({
                "status": "error",
                "message": "Failed to create email transport"
            }));
        }
    };

    match mailer.send(&email_message) {
        Ok(_) => {
            let otp_value = otp as i32;
            match Otp::add_otp(pool, otp_value).await {
                Ok(updated_otp) => {
                    println!("OTP array updated: {:?}", updated_otp.otp);
                    Json(json!({
                        "status": "success",
                        "message": format!("OTP sent to {} and {} and added to DB", mobile, email)
                    }))
                }
                Err(e) => {
                    println!("Error adding OTP to DB: {:?}", e);
                    Json(json!({
                        "status": "error",
                        "message": "DB Error"
                    }))
                }
            }
        }
        Err(e) => {
            println!("Error sending email: {:?}", e);
            Json(json!({
                "status": "error",
                "message": "Email send error"
            }))
        }
    }
}

fn generate_otp() -> u32 {
    rand::thread_rng().gen_range(100000..999999)
}

pub async fn verify_otp(
    State(state): State<AppState>,
    Json(payload): Json<OtpVerify>,
) -> impl IntoResponse {
    let pool = &state.pool;
    let OtpVerify { otp } = payload;

    // Convert u32 -> i32 safely
    let otp_value = otp as i32;

    match Otp::fetch_all_otps(pool).await {
        Ok(updated_otps) => {
            // find the matching otp record with ID
            if let Some(otp_record) = updated_otps
                .iter()
                .find(|record| record.otp.contains(&otp_value))
            {
                let now = Utc::now().naive_utc();
                let elapsed = now - otp_record.created_at;

                if elapsed > Duration::minutes(1) {
                    // OTP expired, delete using otp_record.id
                    let _ = Otp::delete_all_otps(pool, otp_record.id).await;
                    return Json(json!({
                        "status": "error",
                        "message": "OTP expired"
                    }));
                } else {
                    // OTP is still valid
                    println!("OTP {:?} verified successfully", otp_record);
                    let _ = Otp::delete_all_otps(pool, otp_record.id).await;
                    return Json(json!({
                        "status": "success",
                        "message": "OTP verified successfully"
                    }));
                }
            } else {
                // OTP not found
                return Json(json!({
                    "status": "error",
                    "message": "OTP not found"
                }));
            }
        }
        Err(e) => {
            println!("Error fetching OTPs from DB: {:?}", e);
            Json(json!({
                "status": "error",
                "message": "DB Error"
            }))
        }
    }
}


pub async fn register(
    State(state): State<AppState>,
    Json(payload): Json<OTPRequest>,
) -> impl IntoResponse {
    let pool = &state.pool;
    let OTPRequest {
        email,
        mobile,
        username,
    } = payload;

   
    // Basic validation
    if email.is_empty() || mobile.is_empty() || username.is_empty() {
        return Json(json!({
            "status": "error",
            "message": "Missing email, mobile, or username"
        }));
    }

    // Example logic: Insert or trigger OTP generation here
    match User::create_user(pool, &email, &mobile, &username).await {
        Ok(_) => {
            // Successfully generated OTP
            Json(json!({
                "status": "success",
                "message": "Regritration successsfull !!"
            }))
        }
        Err(sqlx::Error::Database(db_err)) if db_err.code() == Some(Cow::Borrowed("23505")) => {
            Json(json!({
                "status": "error",
                "message": "User with this email or phone already exists"
            }))
        }
        Err(e) => {
            // Handle DB or logic error
            println!("Failed to Regrister user: {:?}", e);
            Json(json!({
                "status": "error",
                "message": "Failed to Regrister user"
            }))
        }
    }
}


pub async fn create_admin(
    State(state): State<AppState>,
    Json(payload): Json<Admin>,
) -> impl IntoResponse {
    let pool = &state.pool;

    let Admin {
        id: _,
        email,
        mobile,
        username,
        pincode,
        regocde: _,
    } = payload;

    // Validate required fields
    if email.is_empty() || mobile.is_empty() || username.is_empty() || pincode.is_empty() {
        return Json(json!({
            "status": "error",
            "message": "Missing required fields"
        }));
    }

    // Generate OTP code
    let code = match Otp::generate_code(pool).await {
        Ok(code) => code,
        Err(e) => {
            return Json(json!({
                "status": "error",
                "message": format!("Failed to generate code: {}", e)
            }));
        }
    };


    // Insert admin with OTP
    match Otp::insert_admin(pool, &code, &username, &mobile, &email, &pincode).await {
        Ok(_) => Json(json!({
            "status": "success",
            "message": "Registration successful!"
        })),
        Err(e) => Json(json!({
            "status": "error",
            "message": format!("Registration failed: {}", e)
        })),
    }
}


pub async fn create_admin_users(
    State(state): State<AppState>,
    Json(payload): Json<Admin_Users>,
) -> impl IntoResponse {
    let pool = &state.pool;

    let Admin_Users {
        id: _,
        admin_id,
        regcode: _,
        email,
        mobile,
        username,
        pincode,
    } = payload;


    // Validate required fields
    if email.is_empty() || mobile.is_empty() || username.is_empty() || pincode.is_empty() {
        return Json(json!({
            "status": "error",
            "message": "Missing required fields"
        }));
    }

    // Generate OTP code
    let code = match Otp::generate_code_for_users(pool, &admin_id).await {
        Ok(code) => code,
        Err(e) => {
            return Json(json!({
                "status": "error",
                "message": format!("Failed to generate code: {}", e)
            }));
        }
    };


    // Insert admin with OTP
    match Otp::insert_admin_users(pool, &admin_id, &code,&username, &mobile, &email, &pincode).await {
        Ok(_) => Json(json!({
            "status": "success",
            "message": "Registration successful!"
        })),
        Err(e) => Json(json!({
            "status": "error",
            "message": format!("Registration failed: {}", e)
        })),
    }
}

use dotenv::dotenv;
use sqlx::postgres::PgPoolOptions;
use sqlx::PgPool;
use std::env;

#[derive(Debug)]
pub struct Config {
    pub database_url: String,
    pub app_name: String,
}

impl Config {
    pub fn from_env() -> Self {
        dotenv().ok();
        Self {
            database_url: env::var("DATABASE_URL").expect("DATABASE_URL must be set"),
            app_name: env::var("APP_NAME").unwrap_or_else(|_| "X-ERP".to_string()),
        }
    }
    
}

    // Send SMS using Twilio
    // let twilio_sid = env::var("TWILIO_SID").expect("TWILIO_SID not set");
    // let twilio_token = env::var("TWILIO_AUTH_TOKEN").expect("TWILIO_AUTH_TOKEN not set");
    // let twilio_from = env::var("TWILIO_PHONE_NUMBER").expect("TWILIO_PHONE_NUMBER not set");

    // let client = Client::new();
    // let params = [
    //     ("To", mobile.clone()),
    //     ("From", twilio_from),
    //     ("Body", format!("Your OTP is: {}", otp)),
    // ];

    // let res = client
    //     .post(format!("https://api.twilio.com/2010-04-01/Accounts/{}/Messages.json", twilio_sid))
    //     .basic_auth(&twilio_sid, Some(&twilio_token))
    //     .form(&params)
    //     .send()
    //     .await
    //     .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    //     if !res.status().is_success() {
    //         println!("Error sending email: {:?}", res.text().await.unwrap_or_else(|e| format!("Failed to read response text: {:?}", e)));
    //         return Err(StatusCode::INTERNAL_SERVER_ERROR);
    //     }
        

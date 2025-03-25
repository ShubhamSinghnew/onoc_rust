use std::error::Error;

use serde::{Deserialize, Serialize};
use sqlx::{PgPool, FromRow, Error as SqlxError};

use sqlx::types::Json;
use chrono::NaiveDateTime;
#[derive(FromRow, Serialize, Deserialize, Debug)]
pub struct User {
    pub id: i32,           // Primary Key
    pub username: String,
    pub email: String,
    pub mobile: String,
}

#[derive(FromRow, Serialize, Deserialize, Debug)]
pub struct Otp {
    pub id: i32,
    pub otp: Vec<i32>, // or Json<Vec<i32>> depending on your DB column type
    pub created_at: NaiveDateTime,
}

#[derive(FromRow, Serialize, Deserialize, Debug)]
pub struct Admin {
    pub id: i32,
    pub regcode: String,
    pub user_name: String,
    pub mobile: String,
    pub email: String,
    pub pincode: String
}


#[derive(FromRow, Serialize, Deserialize, Debug)]
pub struct Admin_Users {
    pub id : i32,
    pub admin_id : i32,
    pub regcode: String,
    pub user_name:  String,
    pub mobile: String,
    pub email: String,
    pub pincode: String
}


impl User {
    // Create a new user in the database and return the created user object
    pub async fn create_user(pool: &PgPool, username: &str, email: &str, mobile: &str) -> Result<User, sqlx::Error> {
        let user = sqlx::query_as::<_, User>(
            "INSERT INTO registration (username, email, mobile) VALUES ($1, $2, $3) RETURNING *"
        )
        .bind(username)
        .bind(email)
        .bind(mobile)
        .fetch_one(pool)
        .await?;

        Ok(user)
    }
}


impl Otp {
    pub async fn add_otp(pool: &PgPool, otp: i32) -> Result<Otp, sqlx::Error> {
        let otp_record = sqlx::query_as::<_, Otp>(
           "INSERT INTO otp (otp) VALUES (ARRAY[$1]) RETURNING id, otp, created_at"
        )
        .bind(otp)
        .fetch_one(pool)
        .await?;
    
        Ok(otp_record)
    }
    
    pub async fn fetch_all_otps(pool: &PgPool) -> Result<Vec<Otp>, sqlx::Error> {
        let otps = sqlx::query_as::<_, Otp>("SELECT id, otp, created_at FROM otp")
            .fetch_all(pool)
            .await?;
    
        Ok(otps)
    }
    
    // Assuming you want to delete all OTPs (or you can make it by id)
    pub async fn delete_all_otps(pool: &PgPool, id: i32) -> Result<u64, sqlx::Error> {
        let result = sqlx::query("DELETE FROM otp where id = $1")
            .bind(id)
            .execute(pool)
            .await?;
    
        Ok(result.rows_affected())
    }

    pub async fn generate_code(pool: &PgPool) -> Result<String, sqlx::Error> {
        let count1: Vec<(String,)> = sqlx::query_as("SELECT regcode FROM admins ORDER BY id ASC")
            .fetch_all(pool)
            .await?;
    
        let lst1: String;

        if let Some((last_m_code,)) = count1.last() {
            // Try to parse the numeric part and increment it
            if last_m_code.len() >= 2 {
                // Get prefix (usually "G")
                let prefix = &last_m_code[0..1];
               
                // Remove prefix and try to parse the rest as a number
                let num_part = &last_m_code[1..];
                
                // Try to parse as number and increment
                if let Ok(num) = num_part.parse::<i32>() {
                    lst1 = format!("{}{:05}", prefix, num + 1);
                } else {
                    // If parsing fails, start with 1
                    lst1 = "G00001".to_string();
                }
            } else {
                // If the code is too short, start with 1
                lst1 = "G00001".to_string();
            }
        } else {
            // Fallback if no records
            lst1 = "G00001".to_string();
        }
    
        Ok(lst1)
    }

    pub async fn generate_code_for_users(pool: &PgPool, admin_id: &i32) -> Result<String, sqlx::Error> {
        let mut lst = String::new();
        let mut truncated_du_no = String::new();
    
        // Query to find records with the specified m_code
        let count: Vec<Admin> = sqlx::query_as("SELECT * FROM admins WHERE admin_id = $1")
        .bind(admin_id)
        .fetch_all(pool)
        .await?;
    
        if !count.is_empty() {
            // Find records with m_code LIKE '%M%D%' and matching parent_id
            let find: Vec<Admin_Users> = sqlx::query_as( "SELECT * FROM admins_users WHERE regcode LIKE '%G%U%' AND admin_id = $1")
            .bind(admin_id)
            .fetch_all(pool)
            .await?;
    
            // Sort by m_id in descending order
            let mut sorted_rows = find;
            sorted_rows.sort_by(|a, b| b.id.cmp(&a.id));
    
            if !sorted_rows.is_empty() {
                let last_record = &sorted_rows[0];
                truncated_du_no = last_record.regcode[0..last_record.regcode.len() - 1].to_string();
    
                // Check if last 2 characters are "09"
                if last_record.regcode.len() >= 2 && &last_record.regcode[last_record.regcode.len() - 2..] == "09" {
                    truncated_du_no = last_record.regcode[0..last_record.regcode.len() - 2].to_string();
                    
                    // Parse the last 2 digits, add 1, and concatenate
                    if let Ok(last_digits) = last_record.regcode[last_record.regcode.len() - 2..].parse::<i32>() {
                        lst = format!("{}{}", truncated_du_no, last_digits + 1);
                    }
                } 
                // Check if last digits are greater than 9
                else if last_record.regcode.len() >= 1 {
                    // Use regex to find the trailing digits
                    let re = regex::Regex::new(r"\d+$").unwrap();
                    if let Some(captures) = re.find(&last_record.regcode) {
                        let last_digits_str = captures.as_str();
                        if let Ok(last_digits) = last_digits_str.parse::<i32>() {
                            let num_digits = last_digits_str.len();
                            truncated_du_no = last_record.regcode[0..last_record.regcode.len() - num_digits].to_string();
                            lst = format!("{}{}", truncated_du_no, last_digits + 1);
                        }
                    } else {
                        // If no trailing digits found, handle the last character
                        if let Ok(last_digit) = last_record.regcode[last_record.regcode.len() - 1..].parse::<i32>() {
                            lst = format!("{}{}", truncated_du_no, last_digit + 1);
                        }
                    }
                }
            }
        }
    
        Ok(lst)
    }
    pub async fn insert_admin(
        pool: &PgPool,
        code: &str,
        user_name: &str,
        mobile: &str,
        email: &str,
        pincode: &str
    ) -> Result<Admin, sqlx::Error> {
        let admin_record = sqlx::query_as::<_, Admin>(
            "INSERT INTO admins (regcode, user_name, mobile, email, pincode) 
             VALUES ($1, $2, $3, $4, $5) 
             RETURNING id, regcode, user_name, mobile, email, pincode"
        )
        .bind(code)
        .bind(user_name)
        .bind(mobile)
        .bind(email)
        .bind(pincode)
        .fetch_one(pool)
        .await?;
    
        Ok(admin_record)
    }

    pub async fn insert_admin_users(
        pool: &PgPool,
        admin_id : &i32,
        code: &str,
        user_name: &str,
        mobile: &str,
        email: &str,
        pincode: &str
    ) -> Result<Admin_Users, sqlx::Error> {
        let admin_record = sqlx::query_as::<_, Admin_Users>(
            "INSERT INTO admins_users (regcode, admin_id, user_name, mobile, email, pincode) 
             VALUES ($1, $2, $3, $4, $5, $6) 
             RETURNING id, regcode, user_name, mobile, email, pincode"
        )
        .bind(code)
        .bind(admin_id)
        .bind(user_name)
        .bind(mobile)
        .bind(email)
        .bind(pincode)
        .fetch_one(pool)
        .await?;
    
        Ok(admin_record)
    }
    
    
}

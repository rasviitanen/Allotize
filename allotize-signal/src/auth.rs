use chrono::prelude::*;
use eyre::Result;
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};

const SECRET: &str = "mightysecretb000o";

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    sub: String,
    aud: String,
    #[serde(with = "jwt_numeric_date")]
    exp: DateTime<Utc>,
}

impl Claims {
    pub fn new(sub: String, aud: String, valid_for: chrono::Duration) -> Self {
        let exp = Utc::now() + valid_for;
        Self {
            sub,
            aud,
            exp: exp
                .date()
                .and_hms_milli(exp.hour(), exp.minute(), exp.second(), 0),
        }
    }

    pub fn tokenize(&self) -> Result<String> {
        let token = encode(
            &Header::default(),
            self,
            &EncodingKey::from_secret(SECRET.as_ref()),
        )?;

        Ok(base64::encode(token))
    }

    #[allow(dead_code)]
    pub fn decode(token: &str) -> Result<Self> {
        let claims = decode::<Self>(
            &String::from_utf8(base64::decode(token)?)?,
            &DecodingKey::from_secret(SECRET.as_ref()),
            &Validation::default(),
        )?;

        Ok(claims.claims)
    }
}

mod jwt_numeric_date {
    //! Custom serialization of DateTime<Utc> to conform with the JWT spec (RFC 7519 section 2, "Numeric Date")
    use chrono::{DateTime, TimeZone, Utc};
    use serde::{self, Deserialize, Deserializer, Serializer};

    /// Serializes a DateTime<Utc> to a Unix timestamp (milliseconds since 1970/1/1T00:00:00T)
    pub fn serialize<S>(date: &DateTime<Utc>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let timestamp = date.timestamp();
        serializer.serialize_i64(timestamp)
    }

    /// Attempts to deserialize an i64 and use as a Unix timestamp
    pub fn deserialize<'de, D>(deserializer: D) -> Result<DateTime<Utc>, D::Error>
    where
        D: Deserializer<'de>,
    {
        Utc.timestamp_opt(i64::deserialize(deserializer)?, 0)
            .single() // If there are multiple or no valid DateTimes from timestamp, return None
            .ok_or_else(|| serde::de::Error::custom("invalid Unix timestamp value"))
    }
}

pub const KEYS: [&str; 1] = [
    // rasvi
    "ZXlKMGVYQWlPaUpLVjFRaUxDSmhiR2NpT2lKSVV6STFOaUo5LmV5SnpkV0lpT2lKeVlYTjJhU0lzSW1GMVpDSTZJbWgwZEhCek9pOHZZV3hzYjNScGVtVXVZMjl0SWl3aVpYaHdJam94TmpVM01Ea3hNVGN4ZlEuR09YQnRRTGJYUHRtYkhDSy00b3pMSnI1Q09QRzgtMXNzTjgtMWROeXlmQQ==",
];

#[cfg(test)]
mod tests {
    use jsonwebtoken::errors::ErrorKind;

    use super::*;

    #[test]
    fn generate_api_key() -> Result<()> {
        let username = "rasvi";
        let audience = "https://allotize.com";

        let claims = Claims::new(
            username.to_string(),
            audience.to_string(),
            chrono::Duration::days(365),
        );
        let token = claims.tokenize();

        dbg!(&token);

        Ok(())
    }

    #[test]
    fn verify() -> Result<()> {
        let token = "ZXlKMGVYQWlPaUpLVjFRaUxDSmhiR2NpT2lKSVV6STFOaUo5LmV5SnpkV0lpT2lKeVlYTjJhU0lzSW1GMVpDSTZJbWgwZEhCek9pOHZZV3hzYjNScGVtVXVZMjl0SWl3aVpYaHdJam94TmpVM01Ea3hNVGN4ZlEuR09YQnRRTGJYUHRtYkhDSy00b3pMSnI1Q09QRzgtMXNzTjgtMWROeXlmQQ==";

        let validation = Validation {
            sub: Some("rasvi".to_string()),
            ..Validation::default()
        };
        let token_data = match decode::<Claims>(
            &String::from_utf8(base64::decode(token)?)?,
            &DecodingKey::from_secret(SECRET.as_ref()),
            &validation,
        ) {
            Ok(c) => c,
            Err(err) => match *err.kind() {
                ErrorKind::InvalidToken => panic!("Token is invalid"), // Example on how to handle a specific error
                ErrorKind::InvalidIssuer => panic!("Issuer is invalid"), // Example on how to handle a specific error
                ref e => panic!("Some other errors: {:?}", e),
            },
        };

        dbg!(token_data);

        Ok(())
    }

    #[test]
    fn issue() -> Result<()> {
        let claims = Claims::new(
            "rasmus".to_string(),
            "localhost".to_string(),
            chrono::Duration::days(1),
        );
        let token = claims.tokenize()?;
        dbg!(&token);

        let decoded = Claims::decode(&token).expect("Failed to decode token");
        dbg!(decoded);

        Ok(())
    }
}

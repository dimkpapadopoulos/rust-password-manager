use secrecy::{ExposeSecret, Secret};
use serde::{Deserialize, Serialize, Serializer};

fn serialize_secret<S>(secret: &Secret<String>, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    secret.expose_secret().serialize(serializer)
}
#[derive(Serialize, Deserialize, Debug)]
pub struct Entry {
    pub name: String,
    pub url: String,
    pub username: String,
    #[serde(serialize_with = "serialize_secret")]
    pub password: Secret<String>,
}

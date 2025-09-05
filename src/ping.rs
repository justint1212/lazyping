const PING_HOST: &str = "https://local.forgerock.agiledigital.co";
const AM_ENDPOINT_JSON: &str = "am/json/realms/root";
const AM_ENDPOINT: &str = "am/realms/root";
const IDM_ENDPOINT: &str = "openidm";
const CLIENT_SECRET: &str = "openidm";

use reqwest::redirect::Policy;
use serde::{Deserialize, Serialize};

use std::collections::HashMap;
use std::error::Error;

use reqwest::blocking::Client;
use reqwest::header::{AUTHORIZATION, HeaderMap, HeaderValue};

use crate::model::{Device, User};

type DynError = Box<dyn Error + Send + Sync>;

fn am_endpoint_json(endpoint: &str) -> String {
    format!("{}/{}/{}", PING_HOST, AM_ENDPOINT_JSON, endpoint)
}

fn am_endpoint(endpoint: &str) -> String {
    format!("{}/{}/{}", PING_HOST, AM_ENDPOINT, endpoint)
}

fn idm_endpoint(endpoint: &str) -> String {
    format!("{}/{}/{}", PING_HOST, IDM_ENDPOINT, endpoint)
}

pub struct PingSession {
    idm_token: String,
    am_client: Client,
    client: Client,
}

#[derive(Deserialize)]
struct DeviceRaw {
    #[serde(default)]
    uuid: String,
    #[serde(default, rename = "deviceName")]
    device_name: String,
}

#[derive(Deserialize)]
struct DevicesResponse {
    result: Vec<DeviceRaw>,
}


impl PingSession {
    pub fn new(username: &str, password: &str) -> Result<Self, DynError> {
        let unauthd_client = Client::builder()
            .danger_accept_invalid_certs(true)
            .build()?;

        let build_am_client = || {
            let res = unauthd_client
                .post(am_endpoint_json("authenticate"))
                .header("X-OpenAM-Username", username)
                .header("X-OpenAM-Password", password)
                .send()
                .unwrap();

            let json: serde_json::Value = res.json().unwrap();

            let token = match json.get("tokenId") {
                Some(serde_json::Value::String(tok)) => Ok(tok.clone()),
                _ => Err("invalid am credentials".to_string()),
            }
            .unwrap();

            let mut headers = HeaderMap::new();
            headers.insert(
                "iPlanetDirectoryPro",
                HeaderValue::from_str(&token).unwrap(),
            );

            Client::builder()
                .danger_accept_invalid_certs(true)
                .default_headers(headers)
                .build()
        };

        let build_idm_client = || {
            // authenticate to IDM through AM
            #[derive(Deserialize, Default, Debug)]
            struct AuthData {
                #[serde(default)]
                access_token: String,
            }

            let mut params = HashMap::new();
            params.insert("grant_type", "client_credentials");
            params.insert("client_id", "idm-provisioning");
            params.insert("client_secret", CLIENT_SECRET);
            params.insert("scope", "fr:idm:*");

            let host = format!("{}/am/oauth2/access_token", PING_HOST);

            let res = unauthd_client
                .post(&host)
                .form(&params)
                .header("X-OpenAM-Username", username)
                .header("X-OpenAM-Password", password)
                .header("Accept-API-Version", "resource=2.0, protocol=1.0")
                .send()
                .unwrap();

            let AuthData { access_token } = res.json::<AuthData>()?;

            let mut headers = HeaderMap::new();
            headers.insert(
                AUTHORIZATION,
                HeaderValue::from_str(&format!("{}", access_token)).unwrap(),
            );
            headers.insert(
                "Accept-API-Version",
                HeaderValue::from_static("resource=2.0, protocol=1.0"),
            );

            let result: Result<String, DynError> = Ok(format!("Bearer {}", access_token));
            return result;

            /*
            let result = Client::builder()
                .danger_accept_invalid_certs(true)
                //.default_headers(headers)
                .build();

            result
            */
        };

        let am_client = build_am_client()?;

        Ok(PingSession {
            am_client,
            idm_token: build_idm_client()?,
            client: unauthd_client,
        })
    }

    pub fn oath_list_devices(&self, user_id: &str) -> Result<Vec<Device>, DynError> {
        let res = self
            .am_client
            .get(am_endpoint_json(&format!(
                "users/{}/devices/2fa/oath?_queryFilter=true",
                user_id
            )))
            .send()?;

        let DevicesResponse { result } = res.json::<DevicesResponse>()?;
        Ok(result
            .into_iter()
            .map(|r| Device {
                id: r.uuid,
                name: r.device_name,
            })
            .collect())
    }

    pub fn list_users(&self) -> Result<Vec<User>, DynError> {
        // Try the canonical URL (many IDM installs require the trailing slash)
        let url = idm_endpoint("managed/user?_queryFilter=true");

        // Build to inspect applied headers
        let built = self
            .client
            .get(&url)
            .header("Authorization", self.idm_token.to_string())
            .build()?;

        let res = self.client.execute(built)?; // execute the built request

        #[derive(Deserialize)]
        struct UserRaw {
            #[serde(default)]
            _id: String,

            #[serde(default, rename = "userName")]
            user_name: String,

            #[serde(default, rename = "givenName")]
            given_name: String,

            #[serde(default)]
            sn: String,
        }

        #[derive(Deserialize)]
        struct UsersResponse {
            result: Vec<UserRaw>,
        }

        let UsersResponse { result } = res.json::<UsersResponse>()?;

        Ok(result
            .into_iter()
            .map(|r| User {
                id: r._id,
                username: r.user_name,
                display_name: format!("{} {}", r.given_name, r.sn),
            })
            .collect())
    }
}

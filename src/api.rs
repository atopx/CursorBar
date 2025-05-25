use anyhow::Context;
use anyhow::Result;
use base64::Engine;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use retry::{OperationResult, delay::Fixed, retry};
use serde::Deserialize;
use serde::Serialize;
use ureq::typestate::WithoutBody;
use ureq::RequestBuilder;
use std::time::Duration;

use crate::config::UsageData;
use crate::utils::TokenExtractor;

#[derive(Debug, Deserialize)]
pub struct Gpt4Usage {
    #[serde(rename = "numRequests")]
    pub num_requests: Option<i32>,
    #[serde(rename = "maxRequestUsage")]
    pub max_request_usage: Option<i32>,
}

#[derive(Debug, Deserialize)]
struct ApiUsageResponse {
    #[serde(rename = "gpt-4")]
    gpt4: Option<Gpt4Usage>,
}

#[derive(Debug, Deserialize)]
struct UserInfo {
    pub email: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
struct JwtPayload {
    sub: String,
}

pub struct CursorClient {
    token: Option<String>,
    user_id: Option<String>,
    agent: ureq::Agent,
}

impl CursorClient {
    pub fn new() -> Self {
        let config = ureq::Agent::config_builder().timeout_connect(Some(Duration::from_secs(5))).build();
        let agent = ureq::Agent::new_with_config(config);
        CursorClient { token: None, user_id: None, agent }
    }

    fn get_token(&mut self) -> Result<bool> {
        let token = TokenExtractor::get_access_token()?;
        if let Some(token_str) = token {
            self.token = Some(token_str);
            return Ok(true);
        }
        Ok(false)
    }

    fn extract_user_id(&mut self) -> Result<bool> {
        if let Some(token) = &self.token {
            let parts: Vec<&str> = token.split('.').collect();
            if parts.len() != 3 {
                return Ok(false);
            }

            // 解码JWT负载
            let payload_part = parts[1];
            let decoded = URL_SAFE_NO_PAD.decode(payload_part).context("Failed to decode JWT")?;
            let payload: JwtPayload = serde_json::from_slice(&decoded).context("Failed to parse JWT payload")?;

            let user_id = if payload.sub.contains('|') {
                payload.sub.split('|').nth(1).ok_or_else(|| anyhow::anyhow!("Invalid JWT subject format"))?.to_string()
            } else {
                payload.sub
            };

            self.user_id = Some(user_id);
            return Ok(true);
        }
        Ok(false)
    }

    fn build_cookie(&self) -> Option<String> {
        if let (Some(token), Some(user_id)) = (&self.token, &self.user_id) {
            Some(format!("NEXT_LOCALE=cn; WorkosCursorSessionToken={}%3A%3A{}", user_id, token))
        } else {
            None
        }
    }

    fn set_common_headers(&self, request: RequestBuilder<WithoutBody>) -> RequestBuilder<WithoutBody> {
        // request.header(key, value)
        request
            .header("Accept", "*/*")
            .header(
                "Accept-Language",
                "zh-CN,zh;q=0.9,en-US;q=0.8,en;q=0.7,en-GB;q=0.6",
            )
            .header("Cache-Control", "no-cache")
            .header("Connection", "keep-alive")
            .header("Pragma", "no-cache")
            .header("Referer", "https://www.cursor.com/settings")
            .header(
                "User-Agent",
                "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/116.0.0.0 Safari/537.36",
            )
            .header(
                "sec-ch-ua",
                "\"Chromium\";v=\"116\", \"Not)A;Brand\";v=\"24\", \"Google Chrome\";v=\"116\"",
            )
            .header("sec-ch-ua-mobile", "?0")
            .header("sec-ch-ua-platform", "\"macOS\"")
            .header("Sec-Fetch-Dest", "empty")
            .header("Sec-Fetch-Mode", "cors")
            .header("Sec-Fetch-Site", "same-origin")
    }

    fn get_user_info(&self) -> Result<Option<UserInfo>> {
        if let Some(cookie) = self.build_cookie() {
            let url = "https://www.cursor.com/api/auth/me";
            let cookie_clone = cookie.clone();

            let result = retry(Fixed::from_millis(500).take(2), || {
                let request: ureq::RequestBuilder<ureq::typestate::WithoutBody> = self.agent.get(url);
                let request = self.set_common_headers(request);
                match request.header("Cookie", &cookie_clone).call() {
                    Ok(response) => OperationResult::Ok(response),
                    Err(e) => OperationResult::Retry(anyhow::anyhow!("Request failed: {}", e)),
                }
            });

            match result {
                Ok(mut response) if response.status() == 200 => {
                    let user_info: UserInfo = response.body_mut().read_json()?;
                    return Ok(Some(user_info));
                }
                Ok(_) => return Ok(None),
                Err(e) => return Err(anyhow::anyhow!("Failed after retries: {}", e)),
            }
        }
        Ok(None)
    }

    fn get_usage(&self) -> Result<Option<Gpt4Usage>> {
        if let Some(cookie) = self.build_cookie() {
            let url = "https://www.cursor.com/api/usage";
            let cookie_clone = cookie.clone();

            let result = retry(Fixed::from_millis(500).take(2), || {
                let request = self.agent.get(url);
                let request = self.set_common_headers(request);
                match request.header("Cookie", &cookie_clone).call() {
                    Ok(response) => OperationResult::Ok(response),
                    Err(e) => OperationResult::Retry(anyhow::anyhow!("Request failed: {}", e)),
                }
            });

            match result {
                Ok(mut response) if response.status() == 200 => {
                    let usage: ApiUsageResponse = response.body_mut().read_json()?;
                    return Ok(usage.gpt4);
                }
                Ok(_) => return Ok(None),
                Err(e) => return Err(anyhow::anyhow!("Failed after retries: {}", e)),
            }
        }
        Ok(None)
    }

    pub fn fetch_usage_data(&mut self) -> Result<UsageData> {
        let mut usage_data = UsageData::default();

        // 尝试获取token
        if !self.get_token()? {
            usage_data.error = Some(
                "Unable to obtain access token, please ensure that Cursor is installed and logged in.".to_string(),
            );
            return Ok(usage_data);
        }

        // 提取用户ID
        if !self.extract_user_id()? {
            usage_data.error = Some("Cannot extract user ID from Token".to_string());
            return Ok(usage_data);
        }

        // 获取用户信息(可选)
        if let Ok(Some(user_info)) = self.get_user_info() {
            usage_data.email = user_info.email;
        }

        // 获取用量数据
        match self.get_usage() {
            Ok(Some(usage)) => {
                usage_data.used = usage.num_requests.unwrap_or(0);
                usage_data.total = usage.max_request_usage.unwrap_or(0);
                usage_data.percentage = usage_data.calculate_percentage();
                usage_data.update_time();
            }
            _ => {
                usage_data.error =
                    Some("Unable to retrieve usage data, please check your network connection.".to_string());
            }
        }

        Ok(usage_data)
    }
}

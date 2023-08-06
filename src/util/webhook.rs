// Copyright (c) 2023 VJ <root@5ht2.me>
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use reqwest::Client;
use serde::Serialize;

use crate::{
    config::{WebhookConfig, WebhookLevel},
    CONFIG,
};

pub struct Webhook {
    config: &'static WebhookConfig,
    client: Client,
}

pub enum WebhookColour {
    Green = 0x42_f5_98,
    Orange = 0xde_95_3c,
    Blue = 0x64_95_ed,
}

#[derive(Serialize)]
struct WebhookRequest<'a> {
    embeds: [Embed<'a>; 1],
    avatar_url: &'a str,
    username: &'a str,
}

#[derive(Serialize)]
struct Embed<'a> {
    author: Author<'a>,
    title: String,
    description: String,
    color: u32,
}

#[derive(Serialize)]
struct Author<'a> {
    name: &'a str,
    url: &'a str,
    icon_url: &'a str,
}

impl Webhook {
    pub fn new(config: &'static WebhookConfig) -> Self {
        Self {
            config,
            client: Client::new(),
        }
    }

    pub async fn execute(
        &self,
        title: String,
        message: String,
        level: WebhookLevel,
        colour: WebhookColour,
    ) -> Result<(), String> {
        if self.config.level > level {
            return Ok(());
        }
        let server_url = &CONFIG.live_url;
        let wh_url = &self.config.url;
        if wh_url.is_empty() {
            return Ok(());
        }
        let host = match reqwest::Url::parse(server_url) {
            Ok(url) => url.host_str().unwrap_or(&CONFIG.server_name).to_string(),
            Err(_) => return Err("Invalid server URL".into()),
        };
        let avatar = format!("{server_url}/favicon.png");
        let body = serde_json::to_string(&WebhookRequest {
            embeds: [Embed {
                author: Author {
                    name: &host,
                    url: server_url,
                    icon_url: &avatar,
                },
                title,
                description: message,
                color: colour as u32,
            }],
            avatar_url: &avatar,
            username: &host,
        })
        .unwrap();
        let response = match self.client.post(wh_url).json(&body).send().await {
            Ok(r) => r,
            Err(e) => return Err(format!("failed to send webhook: {e}")),
        };
        match response.status().as_u16() {
            200..=299 => Ok(()),
            _ => Err(format!(
                "failed to send webhook: {}",
                response.text().await.unwrap_or_default()
            )),
        }
    }
}

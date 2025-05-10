mod tests;

use reqwest::Url;
use serde::Deserialize;
use std::fmt::Display;
use typed_builder::TypedBuilder;

///! [`XepherClient`] is the internal handler for all webui.xepher.fun request
#[derive(Debug, TypedBuilder)]
pub struct XepherClient {
    #[builder(setter(
        transform = |session_id: impl Display, user_id: impl Display| format!("SessionID={}; UserID={}", session_id, user_id)
    ))]
    cookie: String,
    #[builder(default = reqwest::ClientBuilder::new().build().unwrap())]
    http_client: reqwest::Client,
    #[builder(default = "Mozilla/5.0 (X11; Linux x86_64; rv:138.0) Gecko/20100101 Firefox/138.0".into())]
    user_agent: String,
}

impl XepherClient {
    pub async fn get_all_scores(&self, user_id: u64) -> anyhow::Result<Vec<IIDXScoreResult>> {
        let referer = Url::parse(&format!("https://webui.xepher.fun/iidx/scores/{user_id}/"))?;
        let score_url = referer.join("list")?;

        let response = self
            .http_client
            .get(score_url)
            .header("User-Agent", &self.user_agent)
            .header("Accept", "application/json")
            .header("Content-Type", "application/json; charset=utf-8")
            .header("Referer", referer.as_str())
            .header("Cookie", &self.cookie)
            .send()
            .await?;

        if !response.status().is_success() {
            anyhow::bail!("fail fetching result for user {user_id}")
        }

        let body = response.bytes().await?;
        let score_detail: GetScoreResponse = serde_json::from_slice(&body)?;

        Ok(score_detail.attempts)
    }
}

#[derive(Debug, Deserialize)]
struct GetScoreResponse {
    attempts: Vec<IIDXScoreResult>,
}

///! Data for one IIDX song attempt
#[derive(Debug, Deserialize)]
pub struct IIDXScoreResult {
    ///! `chart` is the song difficulty, like SPH, SPN.
    pub chart: u8,
    ///! `great` is normal great count
    pub great: u16,
    ///! `miss_count` is the miss count. `miss_count` will be `-1` when user abort this attempt.
    pub miss_count: i16,
    ///! `pgreat` is the perfect great count
    pub pgreat: u16,
    ///! `points` is the score of the current attempt
    pub points: u16,
    ///! `raised` represent if current attempt is the new high score
    pub raised: bool,
    ///! `songid` is the ID of the song
    pub songid: u32,
    ///! `status` is the clear mode, like clear, easy clear, failed...
    pub status: String,
    ///! `timestamp` is the timestamp of when this current attempt is uploaded
    pub timestamp: u64,
    ///! `userid` is the user ID of current attempt, but it always "None" for when query user is
    ///! the login user
    pub userid: String,
}

impl IIDXScoreResult {
    pub fn stringify_difficulty(&self) -> &'static str {
        match self.chart {
            0 => "SPN",
            1 => "SPH",
            2 => "SPA",
            3 => "DPN",
            4 => "DPH",
            5 => "DPA",
            6 => "SPB",
            7 => "SPL",
            8 => "DPB",
            9 => "DPL",
            _ => panic!(
                "Xepher internal error: returning unknown chart id: {}",
                self.chart
            ),
        }
    }
}

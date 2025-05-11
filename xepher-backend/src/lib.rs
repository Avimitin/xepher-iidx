mod tests;

use reqwest::Url;
use serde::Deserialize;
use serde::Deserializer;
use std::collections::HashMap;
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
    #[builder(setter(transform = |p: impl IntoIterator<Item = impl ToString>| p.into_iter().map(|ts| ts.to_string()).collect()))]
    songs_db_paths: Vec<String>,

    // songs_db should be initialized at runtime
    #[builder(default = None, setter(skip))]
    songs_db: Option<HashMap<u32, IIDXSong>>,
}

impl XepherClient {
    pub async fn get_all_iidx_scores(&self, user_id: u64) -> anyhow::Result<Vec<IIDXScoreResult>> {
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

    pub async fn get_iidx_songs_db(&mut self) -> &HashMap<u32, IIDXSong> {
        if self.songs_db.is_none() {
            let db_tasks = self.songs_db_paths.iter().map(|path| async move {
                let data = tokio::fs::read(path)
                    .await
                    .unwrap_or_else(|_| panic!("fail to read song db from: {path}"));
                serde_json::from_slice::<HashMap<u32, IIDXSong>>(&data).unwrap_or_else(|_| {
                    panic!("{path} doesn't contains correct format of IIDX song DB")
                })
            });

            let mut final_db = HashMap::new();
            for t in db_tasks {
                let db = t.await;
                final_db.extend(db);
            }

            self.songs_db = Some(final_db);
        }

        self.songs_db.as_ref().unwrap()
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

///! Represent metadata for one IIDX song
#[derive(Deserialize, Debug)]
pub struct IIDXSong {
    pub title: String,
    #[serde(rename = "asciiTitle")]
    pub ascii_title: String,
    pub genre: String,
    pub artist: String,
    pub version: u16,
    #[serde(rename = "otherFolder", deserialize_with = "u8_to_bool")]
    pub other_folder: bool,
    #[serde(rename = "bemaniFolder", deserialize_with = "u8_to_bool")]
    pub bemani_folder: bool,
    #[serde(rename = "splittableDiff", deserialize_with = "u8_to_bool")]
    pub splittable_diff: bool,
    pub difficulties: IIDXSongDiff,
    #[serde(rename = "entryId")]
    pub entry_id: u32,
    pub volume: u32,
}

fn u8_to_bool<'de, D>(de: D) -> Result<bool, D::Error>
where
    D: Deserializer<'de>,
{
    let val: u8 = Deserialize::deserialize(de)?;
    let actual = match val {
        0 => false,
        _ => true,
    };
    Ok(actual)
}

#[derive(Debug, Deserialize)]
pub struct IIDXSongDiff {
    pub sp: IIDXDiffLevel,
    pub dp: IIDXDiffLevel,
}

///! Actual level for each difficulty
#[derive(Debug, Deserialize)]
pub struct IIDXDiffLevel {
    pub beginner: u8,
    pub normal: u8,
    pub hyper: u8,
    pub another: u8,
    pub legendaria: u8,
}

#![cfg(test)]

use super::*;

fn get_sess_and_user() -> (String, String) {
    let sess_id = std::env::var("SESSION_ID").expect("SESSION_ID env not set");
    let user_id = std::env::var("USER_ID").expect("USER_ID env not set");
    (sess_id, user_id)
}

#[test]
fn test_builder() {
    let (sess_id, user_id) = get_sess_and_user();
    let _: XepherClient = XepherClient::builder().cookie(sess_id, user_id).build();
}

#[tokio::test]
async fn test_get_score() {
    let (sess_id, user_id) = get_sess_and_user();
    let xepher: XepherClient = XepherClient::builder().cookie(sess_id, &user_id).build();

    let scores = xepher
        .get_all_scores(user_id.parse().unwrap())
        .await
        .unwrap();

    assert!(!scores.is_empty());
}

use reqwest::Client;
use wiremock::matchers::{method, path};
use wiremock::{Mock, ResponseTemplate};
use crate::helpers::spawn_app;

#[tokio::test]
async fn subscribe_returns_a_200_for_valid_form_data() {
    // Arrange
    let app = spawn_app().await;
    let body = "name=Jeremy%20Zelaya&email=jeremyzelaya%40example.com";
    Mock::given(path("/email"))
        .and(method("POST"))
        .respond_with(ResponseTemplate::new(200))
        .mount(&app.email_server)
        .await;
    // Act
    let response = app.post_subscriptions(body.into()).await;
    // Assert
    assert_eq!(200, response.status().as_u16());
    let saved = sqlx::query!("SELECT email, name FROM subscriptions")
        .fetch_one(&app.db_pool)
        .await
        .expect("Failed to fetch saved subscription.");
    assert_eq!(saved.email, "jeremyzelaya@example.com");
    assert_eq!(saved.name, "Jeremy Zelaya");
}

#[tokio::test]
async fn subscribe_returns_a_400_when_data_is_missing() {
    // Arrange
    let app = spawn_app().await;
    let test_cases = vec![
        ("name=Jeremy%20Zelaya", "missing email"),
        ("email=john%40example.com", "missing name"),
        ("", "missing name and email"),
    ];
    // Act
    for (invalid_body, error_message) in test_cases {
        let response = app.post_subscriptions(invalid_body.into()).await;
        // Assert
        assert_eq!(
            400,
            response.status().as_u16(),
            "The API did not fail with an invalid request body {}.",
            error_message
        );
    }
}

#[tokio::test]
async fn susbscribe_returns_a_400_when_fields_are_present_but_invalid() {
    let app = spawn_app().await;
    let client = Client::new();
    let test_cases = vec![
        ("name=&email=ursula_le_guin%40gmail.com", "empty name"),
        ("name=Ursula&email=", "empty email"),
        ("name=Ursula&email=not-an-email", "invalid email"),
    ];
    for (body, description) in test_cases {
        let response = app.post_subscriptions(body.into()).await;
        assert_eq!(
            400,
            response.status().as_u16(),
            "The API did not return a 200 Ok when the payload was {}.",
            description
        )
    }
}

#[tokio::test]
async fn subscribe_sends_a_confirmation_email_for_valid_data() {
    // Arrange
    let app = spawn_app().await;
    let body = "name=le%20guin&email=ursula_le_guin%40gmail.com";

    Mock::given(path("/email"))
        .and(method("POST"))
        .respond_with(ResponseTemplate::new(200))
        .expect(1)
        .mount(&app.email_server)
        .await;
    // Act
    let response = app.post_subscriptions(body.into()).await;
    // Assert
    assert_eq!(200, response.status().as_u16());
}

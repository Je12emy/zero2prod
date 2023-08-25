use reqwest::Client;
use sqlx::{Connection, PgConnection};
use zero2prod::{configuration::get_configuration, startup};

#[tokio::test]
async fn health_check_works() {
    // Arrange
    let address = spawn_app();
    let client = Client::new();
    // Act
    let response = client
        .get(&format!("{}/health_check", &address))
        .send()
        .await
        .expect("Failed to execute test");
    // Assert
    assert!(response.status().is_success());
    assert_eq!(Some(0), response.content_length());
}

#[tokio::test]
async fn subscribe_returns_a_200_for_valid_form_data() {
    // Arrange
    let address = spawn_app();
    let configuration = get_configuration().expect("Failed to read configuration");
    let connection_string = configuration.database.connection_string();
    println!("{}", connection_string);
    let mut connection = PgConnection::connect(&connection_string)
        .await
        .expect("Failed to connect to Postgres.");
    let client = Client::new();
    let body = "name=Jeremy%20Zelaya&email=jeremyzelaya%40example.com";
    // Act
    let response = client
        .post(&format!("{}/subscriptions", &address))
        .header("Content-Type", "application/x-www-form-urlencoded")
        .body(body)
        .send()
        .await
        .expect("Failed to execute request");
    // Assert
    assert_eq!(200, response.status().as_u16());
    let saved = sqlx::query!("SELECT email, name FROM subscriptions")
        .fetch_one(&mut connection)
        .await
        .expect("Failed to fetch saved subscription.");
    assert_eq!(saved.email, "jeremyzelaya@example.com");
    assert_eq!(saved.name, "Jeremy Zelaya");
}

#[tokio::test]
async fn subscribe_returns_a_400_when_data_is_missing() {
    // Arrange
    let address = spawn_app();
    let client = Client::new();
    let test_cases = vec![
        ("name=Jeremy%20Zelaya", "missing email"),
        ("email=john%40example.com", "missing name"),
        ("", "missing name and email"),
    ];
    // Act
    for (invalid_body, error_message) in test_cases {
        let response = client
            .post(&format!("{}/subscriptions", &address))
            .body(invalid_body)
            .send()
            .await
            .expect("Failed to execute request");
        // Assert
        assert_eq!(
            400,
            response.status().as_u16(),
            "The API did not fail with an invalid request body {}.",
            error_message
        );
    }
}

fn spawn_app() -> String {
    let listener =
        std::net::TcpListener::bind("127.0.0.1:0").expect("Failed to bind to random port");
    let port = listener.local_addr().unwrap().port();
    let server = startup::run(listener).expect("Failed to bind to address");
    let _ = tokio::spawn(server);
    format!("http://127.0.0.1:{}", port)
}

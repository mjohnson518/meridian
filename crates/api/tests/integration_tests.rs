//! Integration tests for Meridian REST API

use actix_web::{test, web, App};
use meridian_api::{routes, AppState};
use serde_json::json;
use std::sync::Arc;

#[actix_web::test]
async fn test_health_check() {
    let state = Arc::new(AppState::new().await);

    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(state))
            .configure(routes::configure),
    )
    .await;

    let req = test::TestRequest::get().uri("/health").to_request();
    let resp = test::call_service(&app, req).await;

    assert!(resp.status().is_success());

    let body: serde_json::Value = test::read_body_json(resp).await;
    assert_eq!(body["status"], "ok");
    assert!(body.get("version").is_some());
}

#[actix_web::test]
async fn test_create_single_currency_basket() {
    let state = Arc::new(AppState::new().await);

    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(state))
            .configure(routes::configure),
    )
    .await;

    let payload = json!({
        "name": "EUR Basket",
        "currency_code": "EUR",
        "chainlink_feed": "0xb49f677943BC038e9857d61E7d053CaA2C1734C1"
    });

    let req = test::TestRequest::post()
        .uri("/api/v1/baskets/single-currency")
        .set_json(&payload)
        .to_request();

    let resp = test::call_service(&app, req).await;

    assert_eq!(resp.status(), 201); // Created

    let body: serde_json::Value = test::read_body_json(resp).await;
    assert_eq!(body["name"], "EUR Basket");
    assert_eq!(body["basket_type"], "single_currency");
    assert!(body.get("id").is_some());
}

#[actix_web::test]
async fn test_list_baskets() {
    let state = Arc::new(AppState::new().await);

    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(state))
            .configure(routes::configure),
    )
    .await;

    // Create a basket first
    let payload = json!({
        "name": "EUR Basket",
        "currency_code": "EUR",
        "chainlink_feed": "0xb49f677943BC038e9857d61E7d053CaA2C1734C1"
    });

    let req = test::TestRequest::post()
        .uri("/api/v1/baskets/single-currency")
        .set_json(&payload)
        .to_request();
    test::call_service(&app, req).await;

    // List baskets
    let req = test::TestRequest::get()
        .uri("/api/v1/baskets")
        .to_request();
    let resp = test::call_service(&app, req).await;

    assert!(resp.status().is_success());

    let body: Vec<serde_json::Value> = test::read_body_json(resp).await;
    assert_eq!(body.len(), 1);
    assert_eq!(body[0]["name"], "EUR Basket");
}

#[actix_web::test]
async fn test_get_nonexistent_basket() {
    let state = Arc::new(AppState::new().await);

    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(state))
            .configure(routes::configure),
    )
    .await;

    let fake_id = "00000000-0000-0000-0000-000000000000";
    let uri = format!("/api/v1/baskets/{}", fake_id);
    let req = test::TestRequest::get().uri(&uri).to_request();
    let resp = test::call_service(&app, req).await;

    assert_eq!(resp.status(), 404);
}

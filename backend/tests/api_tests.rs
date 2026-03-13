use actix_web::{test, App};
use esp32::controllers::api::{get_all_evacuation_paths, get_building_graph, get_evacuation_path, get_fire_status, get_status};
use esp32::state::app_state::AppState;
use actix_web::web;
use std::sync::Arc;

#[actix_web::test]
async fn test_get_status() {
    let state = Arc::new(AppState::default());
    let app = test::init_service(
        App::new()
            .app_data(web::Data::from(state.clone()))
            .service(get_status)
    ).await;

    let req = test::TestRequest::get().uri("/api/status").to_request();
    let resp = test::call_service(&app, req).await;

    assert!(resp.status().is_success());
}

#[actix_web::test]
async fn test_get_fire_status() {
    let state = Arc::new(AppState::default());
    let app = test::init_service(
        App::new()
            .app_data(web::Data::from(state.clone()))
            .service(get_fire_status)
    ).await;

    let req = test::TestRequest::get().uri("/api/fire/status").to_request();
    let resp = test::call_service(&app, req).await;

    assert!(resp.status().is_success());
}

#[actix_web::test]
async fn test_get_evacuation_path() {
    let state = std::sync::Arc::new(AppState::default());
    let app = actix_web::test::init_service(
        App::new()
            .app_data(web::Data::from(state.clone()))
            .service(get_evacuation_path)
    ).await;

    // Test a valid node
    let req = actix_web::test::TestRequest::get().uri("/api/evacuate/1").to_request();
    let resp = actix_web::test::call_service(&app, req).await;
    assert!(resp.status().is_success() || resp.status() == actix_web::http::StatusCode::NOT_FOUND);
}

#[actix_web::test]
async fn test_get_all_evacuation_paths() {
    let state = std::sync::Arc::new(AppState::default());
    let app = actix_web::test::init_service(
        App::new()
            .app_data(web::Data::from(state.clone()))
            .service(get_all_evacuation_paths)
    ).await;

    let req = actix_web::test::TestRequest::get().uri("/api/evacuate/all").to_request();
    let resp = actix_web::test::call_service(&app, req).await;
    assert!(resp.status().is_success());
}

#[actix_web::test]
async fn test_get_building_graph() {
    let state = std::sync::Arc::new(AppState::default());
    let app = actix_web::test::init_service(
        App::new()
            .app_data(web::Data::from(state.clone()))
            .service(get_building_graph)
    ).await;

    let req = actix_web::test::TestRequest::get().uri("/api/building/graph").to_request();
    let resp = actix_web::test::call_service(&app, req).await;
    assert!(resp.status().is_success());
}

use actix_web::{web, App, HttpServer, Responder, HttpResponse};
use serde_json::Value;
use std::fs;
use std::sync::Mutex;

// Shared state to hold the weather data
struct AppState {
    weather_data: Mutex<Value>,
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Load the weather data from the JSON file
    let weather_data = fs::read_to_string("../python-app/webapp/weather.json")
        .expect("Unable to read weather.json");
    let weather_data: Value = serde_json::from_str(&weather_data).expect("Invalid JSON format");

    let app_state = web::Data::new(AppState {
        weather_data: Mutex::new(weather_data),
    });

    HttpServer::new(move || {
        App::new()
            .app_data(app_state.clone())
            .route("/", web::get().to(root))
            .route("/countries", web::get().to(get_countries))
            .route("/countries/{country}/{city}/{month}", web::get().to(get_weather))
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}

async fn root() -> impl Responder {
    HttpResponse::MovedPermanently()
        .append_header(("Location", "/docs"))
        .finish()
}

async fn get_countries(data: web::Data<AppState>) -> impl Responder {
    let weather_data = data.weather_data.lock().unwrap();
    let countries: Vec<String> = weather_data.as_object()
        .unwrap()
        .keys()
        .cloned()
        .collect();
    HttpResponse::Ok().json(countries)
}

async fn get_weather(
    data: web::Data<AppState>,
    path: web::Path<(String, String, String)>,
) -> impl Responder {
    let (country, city, month) = path.into_inner();
    let weather_data = data.weather_data.lock().unwrap();

    if let Some(country_data) = weather_data.get(&country) {
        if let Some(city_data) = country_data.get(&city) {
            if let Some(month_data) = city_data.get(&month) {
                return HttpResponse::Ok().json(month_data);
            }
        }
    }

    HttpResponse::NotFound().body("Weather data not found")
}

#[cfg(test)]
mod tests {
    use super::*;
    use actix_web::{test, App};

    #[actix_web::test]
    async fn test_get_countries() {
        let weather_data = serde_json::json!({
            "USA": {},
            "Canada": {}
        });
        let app_state = web::Data::new(AppState {
            weather_data: Mutex::new(weather_data),
        });

        let app = test::init_service(
            App::new()
                .app_data(app_state.clone())
                .route("/countries", web::get().to(get_countries)),
        )
        .await;

        let req = test::TestRequest::get().uri("/countries").to_request();
        let mut resp: Vec<String> = test::call_and_read_body_json(&app, req).await;

        // Sort the response to ensure consistent order
        resp.sort();

        let mut expected = vec!["USA", "Canada"];
        expected.sort();

        assert_eq!(resp, expected);
    }
}


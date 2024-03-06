use actix_web::HttpResponse;

pub async fn admin_dashboard() -> HttpResponse {
    HttpResponse::Ok().body("Admin dashboard")
}

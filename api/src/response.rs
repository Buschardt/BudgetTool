use axum::Json;
use serde::Serialize;

#[derive(Serialize)]
pub struct ApiResponse<T: Serialize> {
    pub ok: bool,
    pub data: T,
}

impl<T: Serialize> ApiResponse<T> {
    pub fn success(data: T) -> Json<ApiResponse<T>> {
        Json(ApiResponse { ok: true, data })
    }
}

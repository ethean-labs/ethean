use axum::{
    extract::{State, Path},
    response::Json,
    routing::get,
    Router,
};
use crate::api::{ApiState, types::*};
use crate::types::StateRoot;

pub fn create_routes() -> Router<ApiState> {
    Router::new()
        .route("/genesis", get(get_genesis))
        .route("/states/:state_id/root", get(get_state_root))
        .route("/states/:state_id/fork", get(get_state_fork))
}

pub async fn get_genesis(State(_state): State<ApiState>) -> Json<ApiResponse<Genesis>> {
    Json(ApiResponse::success(Genesis::default()))
}

pub async fn get_state_root(
    Path(_state_id): Path<String>,
    State(_state): State<ApiState>,
) -> Json<ApiResponse<StateRoot>> {
    Json(ApiResponse::success(StateRoot::default()))
}

pub async fn get_state_fork(
    Path(_state_id): Path<String>, 
    State(_state): State<ApiState>,
) -> Json<ApiResponse<Fork>> {
    Json(ApiResponse::success(Fork::default()))
}

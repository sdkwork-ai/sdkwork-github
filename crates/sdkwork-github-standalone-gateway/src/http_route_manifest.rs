use sdkwork_web_core::{HttpRoute, HttpRouteManifest};

use sdkwork_github_gateway_assembly::{APP_HTTP_ROUTES, BACKEND_HTTP_ROUTES};

const GITHUB_HTTP_ROUTES: [HttpRoute; 33] = [
    APP_HTTP_ROUTES[0],
    APP_HTTP_ROUTES[1],
    APP_HTTP_ROUTES[2],
    APP_HTTP_ROUTES[3],
    APP_HTTP_ROUTES[4],
    APP_HTTP_ROUTES[5],
    APP_HTTP_ROUTES[6],
    APP_HTTP_ROUTES[7],
    APP_HTTP_ROUTES[8],
    APP_HTTP_ROUTES[9],
    APP_HTTP_ROUTES[10],
    APP_HTTP_ROUTES[11],
    APP_HTTP_ROUTES[12],
    APP_HTTP_ROUTES[13],
    APP_HTTP_ROUTES[14],
    APP_HTTP_ROUTES[15],
    APP_HTTP_ROUTES[16],
    APP_HTTP_ROUTES[17],
    APP_HTTP_ROUTES[18],
    APP_HTTP_ROUTES[19],
    APP_HTTP_ROUTES[20],
    APP_HTTP_ROUTES[21],
    APP_HTTP_ROUTES[22],
    APP_HTTP_ROUTES[23],
    APP_HTTP_ROUTES[24],
    APP_HTTP_ROUTES[25],
    APP_HTTP_ROUTES[26],
    APP_HTTP_ROUTES[27],
    APP_HTTP_ROUTES[28],
    APP_HTTP_ROUTES[29],
    BACKEND_HTTP_ROUTES[0],
    BACKEND_HTTP_ROUTES[1],
    BACKEND_HTTP_ROUTES[2],
];

pub fn github_route_manifest() -> HttpRouteManifest {
    HttpRouteManifest::new(&GITHUB_HTTP_ROUTES)
}

pub fn github_public_path_prefixes() -> Vec<String> {
    sdkwork_web_bootstrap::infra_public_path_prefixes()
}

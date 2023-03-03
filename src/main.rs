use actix_cors::Cors;
use actix_web::http::header;
use actix_web::web::Data;
use actix_web::{guard, web, App, HttpRequest, HttpResponse, HttpServer};
use anyhow::Result;
use async_graphql::http::{playground_source, GraphQLPlaygroundConfig};
use async_graphql::{EmptyMutation, EmptySubscription, Object, Schema};
use async_graphql_actix_web::{GraphQLRequest, GraphQLResponse, GraphQLSubscription};
use dotenv::dotenv;
use std::env;

struct Query;
#[Object]
impl Query {
    /// Returns the sum of a and b
    async fn add(&self, a: i32, b: i32) -> i32 {
        a + b
    }
}

type ApiSchema = Schema<Query, EmptyMutation, EmptySubscription>;

async fn index(schema: web::Data<ApiSchema>, req: GraphQLRequest) -> GraphQLResponse {
    schema.execute(req.into_inner()).await.into()
}

async fn index_ws(
    schema: web::Data<ApiSchema>,
    req: HttpRequest,
    payload: web::Payload,
) -> actix_web::Result<HttpResponse> {
    GraphQLSubscription::new(Schema::clone(&*schema)).start(&req, payload)
}

async fn index_playground() -> actix_web::Result<HttpResponse> {
    let path = env::var("API_PATH").expect("API_PATH must be set");
    let source =
        playground_source(GraphQLPlaygroundConfig::new(&path).subscription_endpoint(&path));
    Ok(HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(source))
}

#[actix_web::main]
async fn main() -> Result<()> {
    dotenv().ok();
    let port = env::var("API_PORT").expect("API_PORT must be set");
    let path = env::var("API_PATH").expect("API_PATH must be set");
    let schema = Schema::build(Query, EmptyMutation, EmptySubscription).finish();

    println!("Playground: http://{port}{path}");
    HttpServer::new(move || {
        let cors = Cors::default()
            .allow_any_origin()
            .allowed_methods(vec!["GET", "POST"])
            .allowed_headers(vec![header::AUTHORIZATION, header::ACCEPT])
            .allowed_header(header::CONTENT_TYPE)
            .supports_credentials();
        App::new()
            .wrap(cors)
            .app_data(Data::new(schema.clone()))
            .service(web::resource(&path).guard(guard::Post()).to(index))
            .service(
                web::resource(&path)
                    .guard(guard::Get())
                    .guard(guard::Header("upgrade", "websocket"))
                    .to(index_ws),
            )
            .service(
                web::resource(&path)
                    .guard(guard::Get())
                    .to(index_playground),
            )
    })
    .bind(&port)?
    .run()
    .await?;
    Ok(())
}

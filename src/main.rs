use actix_web::{get, post, middleware, web::{self, Json, Form}, App, HttpRequest, HttpServer, Responder, HttpResponse, http::StatusCode};
use mongodb::{
    bson::{doc, Document},
    options::{ClientOptions, ResolverConfig},
    Client, Collection,
};
use std::fs;
use std::io::prelude::*;
use std::net::{TcpListener, TcpStream};
use serde::{Serialize, Deserialize};
use futures_util::stream::StreamExt;
use chrono::{DateTime, Utc, Local};

#[derive(Debug, Serialize, Deserialize)]
pub struct Details{
    username: String,
    name: String,
    address: String
}
const DB_NAME: &str = "rust-api";
const COL_NAME: &str = "apidata";

#[actix_web::main] // or #[tokio::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| {
        App::new()
            .service(home)
            .service(new_user)
            .service(get_data)
            .service(user)      
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}

#[get("/")]
async fn home() -> impl Responder{
    HttpResponse::build(StatusCode::OK).content_type("text/html; charset=utf-8").body(include_str!("../initial.html"))
}

#[post("/add_user")]
async fn new_user(val : Form<Details>) -> impl Responder{
    let client = Client::with_uri_str("mongodb://localhost:27017").await.expect("Unable to connect.");
    let db:Collection<Document> = client.database(DB_NAME).collection(COL_NAME);
    let ins_data = doc!{
        "username":&val.username,
        "name": &val.name,
        "address": &val.address
    };
    let res = db.insert_one(ins_data, None).await;
    match res {
        Ok(_) => HttpResponse::Ok().body("User added."),
        Err(e)  => HttpResponse::InternalServerError().body(e.to_string())
    }
}

#[get("/data/{usrname}")]
async fn user(usrname: web::Path<String>) -> impl Responder{
    let client = Client::with_uri_str("mongodb://localhost:27017").await.expect("Unable to connect.");
    let db:Collection<Document> = client.database(DB_NAME).collection(COL_NAME);
    // match db.find_one(doc! {"username":usrname.to_string()}, None).await{
    //     Ok(Some(data)) => HttpResponse::Ok().json(data),
    //     Ok(None) =>{
    //         HttpResponse::InternalServerError().body("Data not found.")
    //     }
    //     Err(e) => HttpResponse::InternalServerError().body(e.to_string())
    //}
    match db.find_one(doc! {"username":usrname.to_string()}, None).await.unwrap(){
        Some(data) => HttpResponse::Ok().json(data),
        None => HttpResponse::InternalServerError().body("Not found")
    }
}

#[get("/all_records")]
async fn get_data() -> impl Responder{
    let client = Client::with_uri_str("mongodb://localhost:27017").await.expect("Unable to connect.");
    let db:Collection<Document> = client.database(DB_NAME).collection(COL_NAME);
    let mut v = db.find(doc! {}, None).await.expect("Not found.");
    let mut vector:Vec<Document> = vec![];
    while let Some(Ok(r)) = v.next().await{
        vector.push(r);
    }
    let new_data = serde_json::to_string_pretty(&vector).unwrap();
    HttpResponse::Ok().body(new_data)
}


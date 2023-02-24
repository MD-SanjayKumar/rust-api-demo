use actix_web::{
    get,
    http::StatusCode,
    middleware, post,
    web::{self, Form, Json},
    App, HttpRequest, HttpResponse, HttpServer, Responder,
};
use actix_web_httpauth::extractors::basic::BasicAuth;
use chrono::{DateTime, Local, Utc};
use futures_util::stream::StreamExt;
use mongodb::{
    bson::{doc, Document},
    options::{ClientOptions, ResolverConfig},
    Client, Collection,
};
use serde::{Deserialize, Serialize};
use std::fs;
use std::io::prelude::*;
use std::net::{TcpListener, TcpStream};

#[derive(Debug, Serialize, Deserialize)]
pub struct Details {
    username: String,
    email: String,
    name: String,
    address: String,
}
static USR: &str = "Admin";
static PWD: &str = "Admin123";

#[actix_web::main] // or #[tokio::main]
async fn main() -> std::io::Result<()> {
    let client = Client::with_uri_str("mongodb://localhost:27017")
        .await
        .expect("Unable to connect.");
    let db: Collection<Document> = client.database("rust-api").collection("apidata");
    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(db.clone()))
            .service(home)
            .service(new_user)
            .service(get_data)
            .service(user)
            .service(delete_user)
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}

#[get("/")]
async fn home() -> impl Responder {
    HttpResponse::build(StatusCode::OK)
        .content_type("text/html; charset=utf-8")
        .body(include_str!("../initial.html"))
}

#[post("/add_user")]
async fn new_user(
    db: web::Data<Collection<Document>>,
    crad: BasicAuth,
    val: Form<Details>,
) -> impl Responder {
    if crad.user_id() == USR && crad.password().unwrap() == PWD {
        let ins_data = doc! {
            "username":&val.username,
            "email":&val.email,
            "name": &val.name,
            "address": &val.address
        };
        let res = db.insert_one(ins_data, None).await;
        match res {
            Ok(_) => HttpResponse::Ok().body("User added."),
            Err(e) => HttpResponse::InternalServerError().body(e.to_string()),
        }
    } else {
        HttpResponse::InternalServerError().body("Please enter valid credentials.")
    }
}

#[get("/user/{usrname}")]
async fn user(
    db: web::Data<Collection<Document>>,
    crad: BasicAuth,
    usrname: web::Path<String>,
) -> impl Responder {
    if crad.user_id() == USR && crad.password().unwrap() == PWD {
        // match db.find_one(doc! {"username":usrname.to_string()}, None).await{
        //     Ok(Some(data)) => HttpResponse::Ok().json(data),
        //     Ok(None) =>{
        //         HttpResponse::InternalServerError().body("Data not found.")
        //     }
        //     Err(e) => HttpResponse::InternalServerError().body(e.to_string())
        //}
        match db
            .find_one(doc! {"username":usrname.to_string()}, None)
            .await
            .unwrap()
        {
            Some(data) => HttpResponse::Ok().json(data),
            None => HttpResponse::InternalServerError().body("Not found"),
        }
    } else {
        HttpResponse::InternalServerError().body("Please enter valid credentials.")
    }
}

#[get("/all_records")]
async fn get_data(db: web::Data<Collection<Document>>, crad: BasicAuth) -> impl Responder {
    if crad.user_id() == USR && crad.password().unwrap() == PWD {
        let mut v = db.find(doc! {}, None).await.expect("Not found.");
        let mut vector: Vec<Document> = vec![];
        while let Some(Ok(r)) = v.next().await {
            vector.push(r);
        }
        let new_data = serde_json::to_string_pretty(&vector).unwrap();
        HttpResponse::Ok().body(new_data)
    } else {
        HttpResponse::InternalServerError().body("Please enter valid credentials.")
    }
}

#[get("/delete/{usrname}")]
async fn delete_user(
    db: web::Data<Collection<Document>>,
    crad: BasicAuth,
    usrname: web::Path<String>,
) -> impl Responder {
    if crad.user_id() == USR && crad.password().unwrap() == PWD {
        match db
            .delete_one(doc! {"username":usrname.to_string()}, None)
            .await
        {
            Ok(_) => HttpResponse::Ok().body("Deleted record."),
            Err(_) => HttpResponse::InternalServerError().body("Not found."),
        }
    } else {
        HttpResponse::InternalServerError().body("Please enter valid credentials.")
    }
}

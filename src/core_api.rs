use actix_web::{get, post, web::{self, Payload}, App, FromRequest, HttpMessage, HttpRequest, HttpServer, Responder, Result};

#[post("/user/auth")]
async fn auth(req: HttpRequest) -> Result<String> {
    let mut tmp = req.headers().get("ID");
    match tmp {
        None => Ok(format!("{}", "Failed")),
        Some(val) => {
            let key = val.to_str().expect("sas");
            if key.eq("1234") {
                Ok(format!("Xin chao{}", key))
            }
            else {
                Ok(format!("Cut' lon. xao` me"))
            }
        }
    }
}

#[get("/user/cac")] // <- define path parameters
async fn index(req: HttpRequest) -> Result<String> {
    //let name: String = req.match_info().get("friend").unwrap().parse().unwrap();
    match req.headers().get("ID") {
        None => Ok(format!("Failed")),
        Some(secret) => {
            let a= secret.to_str().expect("msg");
            if a == "123" {
                Ok(format!("{}",a))
            }
            else {
                Ok(format!("Ok"))
            }
        }
    }
}


#[actix_web::main]
pub async fn actix_web_main() -> std::io::Result<()> {
    HttpServer::new(|| {
        App::new()
        .service(index)
        .service(auth)
    }).bind(("127.0.0.1", 8080))?
    .run()
    .await
}
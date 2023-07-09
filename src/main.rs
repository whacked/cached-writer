extern crate argparse_rs as argparse;
use actix_web::{get, post, web, App, HttpResponse, HttpServer, Responder};
use argparse::{hashmap_parser, vec_parser, ArgParser, ArgType};
use std::collections::HashMap;
mod cached_writer;
use ctrlc;
use serde_json::{json, Value};
use std::fs::OpenOptions;
use std::io::{Result, Write};
use std::rc::Rc;
use std::sync::mpsc::channel;
use std::sync::{Arc, Mutex};

/* simple example of using mutex to share a file between routes

type FileWriter = Mutex<std::fs::File>;

fn append_data(file: &FileWriter, data: &str) -> Result<()> {
    let mut file = file.lock().unwrap();
    file.write_all(data.as_bytes())?;
    file.write_all(b"\n")?;
    Ok(())
}

#[get("/route1")]
async fn route1(writer: web::Data<FileWriter>) -> impl Responder {
    append_data(&writer, "Data from route1").unwrap();
    HttpResponse::Ok().body("Data appended from route1")
}

#[post("/route2")]
async fn route2(writer: web::Data<FileWriter>, body: web::Bytes) -> impl Responder {
    let data = String::from_utf8(body.to_vec()).unwrap();
    append_data(&writer, &data).unwrap();
    HttpResponse::Ok().body("Data appended from route2")
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let file = OpenOptions::new().append(true).open("/tmp/mutex-example.txt").expect("Failed to open file");
    let file_writer = web::Data::new(Mutex::new(file));

    HttpServer::new(move || {
        App::new()
            .app_data(file_writer.clone())
            .service(route1)
            .service(route2)
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}
*/

type CachedWriterMutex = Mutex<cached_writer::CachedWriter>;

#[get("/")]
async fn index(mutex_writer: web::Data<CachedWriterMutex>) -> impl Responder {
    HttpResponse::Ok().body("post please")
}

#[post("/")]
async fn slurp(
    mutex_writer: web::Data<CachedWriterMutex>,
    payload: web::Json<Value>,
) -> impl Responder {
    let payload_data = payload.into_inner();
    let jsonl = serde_json::to_string(&payload_data).unwrap();
    let mut writer = mutex_writer.lock().unwrap();
    writer.append(jsonl.to_string());
    HttpResponse::Ok().body(jsonl)
}

#[actix_web::main]
async fn start_webserver(
    host: String,
    port: u16,
    writer_output_file: String,
    writer_flush_frequency: u16,
) -> std::io::Result<()> {
    let writer = cached_writer::CachedWriter::new(writer_output_file, writer_flush_frequency);

    let mutex_writer = web::Data::new(Mutex::new(writer));
    let cleanup_writer = mutex_writer.clone();

    ctrlc::set_handler(move || {
        println!("received Ctrl+C! Flushing writer...");
        cleanup_writer.lock().unwrap().flush();
        println!("bye!");
    })
    .expect("Error setting Ctrl-C handler");

    println!("Starting webserver... on {}:{}", host, port);
    HttpServer::new(move || {
        App::new()
            .app_data(mutex_writer.clone())
            .service(index)
            .service(slurp)
    })
    .bind((host, port))?
    .run()
    .await
}

fn main() -> Result<()> {
    const DEFAULT_PORT: u16 = 8888;
    const DEFAULT_OUTPUT_FILE: &str = "./out.jsonl";
    const DEFAULT_FLUSH_FREQUENCY: u16 = 100;

    let mut parser = ArgParser::new("logger".into());

    parser.add_opt(
        "flush-frequency",
        Some(&DEFAULT_FLUSH_FREQUENCY.to_string()),
        'f',
        false,
        "flush when <f> records are received",
        ArgType::Option,
    );
    parser.add_opt(
        "expose",
        Some("false"),
        'e',
        false,
        "bind 0.0.0.0",
        ArgType::Flag,
    );
    parser.add_opt(
        "port",
        Some(&DEFAULT_PORT.to_string()),
        'p',
        false,
        "bind port",
        ArgType::Option,
    );
    parser.add_opt(
        "output-file",
        Some(DEFAULT_OUTPUT_FILE),
        'o',
        false,
        "output file path",
        ArgType::Dict,
    );

    if false {
        let test_1 = "./go -q 100"
            .split_whitespace()
            .map(|s| s.into())
            .collect::<Vec<String>>();

        let p_res = parser.parse(test_1.iter()).unwrap();

        assert!(p_res.get("flush-frequency") == Some(100));
        assert!(p_res.get::<String>("output-file") == Some("out.jsonl".into()));
    }

    // parse the command line arguments
    let args: Vec<String> = std::env::args().collect();
    let p_res = parser.parse(args.iter()).unwrap();

    if p_res.get("help") == Some(true) {
        parser.help();
        return Ok(());
    }

    let host = if p_res.get::<bool>("expose") == Some(true) {
        "0.0.0.0"
    } else {
        "127.0.0.1"
    };

    start_webserver(
        host.to_string(),
        p_res.get("port").unwrap_or(DEFAULT_PORT),
        p_res
            .get("output-file")
            .unwrap_or(DEFAULT_OUTPUT_FILE.into()),
        p_res
            .get("flush-frequency")
            .unwrap_or(DEFAULT_FLUSH_FREQUENCY),
    )?;

    Ok(())
}

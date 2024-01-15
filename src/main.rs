// src/main.rs
#[macro_use]
extern crate log;

use actix_web::{get, App, HttpResponse, HttpServer, Responder};
use dotenv::dotenv;
use listenfd::ListenFd;
use std::env;
use serde::{Serialize, Deserialize};
use serde_json::{json};

use rusqlite::{Connection, Result};
// use rusqlite::NO_PARAMS;

// const DB_CONNECTION: Connection = Connection::open("natural_nails.db").unwrap();
fn get_database_connection() -> Connection {
    Connection::open("natural_nails.db").unwrap()
}

#[derive(Debug, Serialize, Deserialize)]
struct Employees {
    id: i32,
    name: String,
    phone: String,
    address: String,
    ssn: String,
    work_percentage: u8,
    cash_percentage: u8,
}

fn map_to_employees(row: &rusqlite::Row) -> Result<Employees> {
    Ok(Employees {
        id: row.get(0)?,
        name: row.get(1)?,
        phone: row.get(2)?,
        address: row.get(3)?,
        ssn: row.get(4)?,
        work_percentage: row.get(5)?,
        cash_percentage: row.get(6)?,
    })
}

#[derive(Debug, Serialize, Deserialize)]
struct Work {
    id: i32,
    date: f32,
    total: f32,
    tip: f32,
    discount: f32,
    charged: f32
}

fn map_to_work(row: &rusqlite::Row) -> Result<Work> {
    Ok(Work {
        id: row.get(0)?,
        date: row.get(1)?,
        total: row.get(2)?,
        tip: row.get(3)?,
        discount: row.get(4)?,
        charged: row.get(5)?
    })
}

#[derive(Debug, Serialize, Deserialize)]
struct EmployeeWork {
    id: i32,
    employee_id: i32,
    work_id: i32,
    amount: f32,
    tip: f32
}

fn map_to_employee_work(row: &rusqlite::Row) -> Result<EmployeeWork> {
    Ok(EmployeeWork {
        id: row.get(0)?,
        employee_id: row.get(1)?,
        work_id: row.get(2)?,
        amount: row.get(3)?,
        tip: row.get(4)?
    })
}
    

#[get("/")]
async fn index() -> impl Responder {
    HttpResponse::Ok().body("Hello world! Everything works")
}

#[get("/database/init")]
async fn database_init() -> impl Responder {
    let conn: Connection = get_database_connection();
    conn.execute("
        create table if not exists employees (
            id integer primary key,
            name text not null,
            phone text not null,
            address text,
            ssn text,
            work_percentage integer,
            cash_percentage integer
        )", []
    ).unwrap();

    conn.execute("
        create table if not exists work (
            id integer primary key,
            date text not null,
            total float not null,
            tip float,
            discount float,
            charged float
        )", []
    ).unwrap();

    conn.execute("
        create table if not exists employee_work (
            id integer primary key,
            employee_id integer references employees.id,
            work_id integer references work.id,
            amount float not null,
            tip float
        )", []
    ).unwrap();
    HttpResponse::Ok().body("Database has been initialized!")
}

#[get("/database/format")]
async fn database_format() -> impl Responder {
    get_database_connection().execute({
        "drop table employees;
        drop table work;
        drop table employee_work;"
    }, []).unwrap();
    HttpResponse::Ok().body("Database has been deleted!")
}

#[get("/employees")]
async fn get_all_employees() -> HttpResponse {
    // grab data from database
    // let query = "select * from employees";
    let conn: Connection = get_database_connection();
    let stmt = conn.execute("SELECT * FROM employees", []).unwrap();

    // convert data to vector
    // let rows = stmt.query_map([], map_to_employees);
    // let mut result = Vec::new();
    // // for row in rows {
    // //     result.push(row?);
    // // }

    // let json_data = serde_json::to_value(&result).unwrap_or_else(|err| {
    //     print!("Error serializing to JSON: {:?}", err);
    //     Value::Null
    // });

    HttpResponse::Ok()
        .content_type("application/json")
        .body(json!(stmt).to_string())
}

#[actix_rt::main]
async fn main() -> std::io::Result<()> {
    // .env file data grap and log init
    dotenv().ok();
    env_logger::init();

    let mut listenfd = ListenFd::from_env();
    let mut server = HttpServer::new(||
        App::new()
            .service(index)
    );

    server = match listenfd.take_tcp_listener(0)? {
        Some(listener) => server.listen(listener)?,
        None => {
            let host = env::var("HOST").expect("Host not set");
            let port = env::var("PORT").expect("Port not set");
            server.bind(format!("{}:{}", host, port))?
        }
    };

    info!("Starting server");
    server.run().await
}
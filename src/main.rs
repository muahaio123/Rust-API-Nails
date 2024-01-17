// src/main.rs
#[macro_use]
extern crate log;

use std::marker::Sized;
use actix_web::{web, App, HttpResponse, HttpServer, Responder, get, post, put, delete};
use dotenv::dotenv;
use listenfd::ListenFd;
use std::env;
use serde::{Serialize, Deserialize};
use serde_json::json;

use rusqlite::{Connection, Result, Statement, named_params, params};
// use rusqlite::NO_PARAMS;

// const DB_CONNECTION: Connection = Connection::open("natural_nails.db").unwrap();
fn get_database_connection() -> Connection {
    Connection::open("natural_nails.db").unwrap()
}

#[derive(Debug, Serialize, Deserialize)]
struct Employees {
    id: i64,
    name: String,
    phone: String,
    address: String,
    ssn: String,
    work_percentage: u8,
    cash_percentage: u8,
    is_active: bool
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
        is_active: row.get(7)?
    })
}

#[derive(Debug, Serialize, Deserialize)]
struct Work {
    id: i64,
    timestamp: String,
    total: f32,
    tip: f32,
    discount: f32,
    charged: f32
}

fn map_to_work(row: &rusqlite::Row) -> Result<Work> {
    Ok(Work {
        id: row.get(0)?,
        timestamp: row.get(1)?,
        total: row.get(2)?,
        tip: row.get(3)?,
        discount: row.get(4)?,
        charged: row.get(5)?
    })
}

#[derive(Debug, Serialize, Deserialize)]
struct EmployeeWork {
    id: i64,
    employee_id: i64,
    work_id: i64,
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
    // Assume get_database_connection() returns a Connection
    let conn: Connection = get_database_connection();

    if let Err(err) = conn.execute(
        "
        create table if not exists employees (
            id integer primary key,
            name text not null,
            phone text not null,
            address text,
            ssn text,
            work_percentage integer,
            cash_percentage integer,
            is_active boolean not null
        )",
        [],
    ) {
        return HttpResponse::InternalServerError().body(format!("Error initializing employees table: {:?}", err));
    }

    if let Err(err) = conn.execute(
        "
        create table if not exists work (
            id integer primary key,
            timestamp text not null,
            total float not null,
            tip float,
            discount float,
            charged float
        )",
        [],
    ) {
        return HttpResponse::InternalServerError().body(format!("Error initializing work table: {:?}", err));
    }

    if let Err(err) = conn.execute(
        "
        create table if not exists employee_work (
            id integer primary key,
            employee_id integer references employees(id),
            work_id integer references work(id),
            amount float not null,
            tip float
        )",
        [],
    ) {
        return HttpResponse::InternalServerError().body(format!("Error initializing employee_work table: {:?}", err));
    }

    return HttpResponse::Ok().body("Database has been initialized!");
}

#[get("/database/format")]
async fn database_format() -> impl Responder {

    match get_database_connection().execute("
            drop table employees;
            drop table work;
            drop table employee_work;
        ", 
        [],
    ) {
        Err(err) => 
            return HttpResponse::InternalServerError()
                .body(format!("Error formatting database: {:?}", err)),
        Ok(_) => HttpResponse::Ok().body("Database been formatted!")
    }
}

// GET, POST, PUT, DELETE for Employee
#[get("/employees")]
async fn get_all_employees() -> HttpResponse {
    // grab data from database
    let conn: Connection = get_database_connection();
    let mut result: Statement<'_> = conn.prepare("SELECT * FROM employees").unwrap();
    // map data to struct
    let rows = result.query_map([], map_to_employees).unwrap();
    // collect results into vector
    let mut json_vec = Vec::new();
    // Convert the vector of Employee to JSON
    for row in rows {
        json_vec.push(row.unwrap());
    }

    let json = json!(&json_vec);
    HttpResponse::Ok()
        .content_type("application/json")
        .body(json.to_string())
}

#[post("/employees")]
async fn post_new_employee(mut new_employee: web::Json<Employees>) -> HttpResponse {
    let conn: Connection = get_database_connection();
    let mut stmt = conn.prepare
        ("INSERT INTO employees (name, phone, address, ssn, work_percentage, cash_percentage, is_active) 
        VALUES (:name, :phone, :address, :ssn, :work_percentage, :cash_percentage, :is_active)").unwrap();

    match stmt.execute(named_params! {
            ":name" : new_employee.name,
            ":phone" : new_employee.phone,
            ":address" : new_employee.address,
            ":ssn" : new_employee.ssn,
            ":work_percentage" : new_employee.work_percentage,
            ":cash_percentage" : new_employee.cash_percentage,
            ":is_active" : new_employee.is_active
        }) {
            Ok(_) => {
                new_employee.id = conn.last_insert_rowid();
                HttpResponse::Ok()
                    .content_type("application/json")
                    .body(json!(new_employee).to_string())
            },
            Err(err) => HttpResponse::InternalServerError()
                    .content_type("application/json")
                    .body(format!("Error: {:?}", err))
        }

}

#[put("/employees")]
async fn put_update_employee(employee: web::Json<Employees>) -> HttpResponse {
    let conn: Connection = get_database_connection();

    if check_table_row_exist("employees".to_string(), employee.id.to_string()) {
        let mut stmt = conn.prepare
            ("UPDATE employees SET name = :name, phone = :phone, address = :address, ssn = :ssn, work_percentage = :work_percentage, cash_percentage = :cash_percentage WHERE id = :emp_id").unwrap();
    
        let _ = stmt.execute(named_params! {
            ":name" : employee.name,
            ":phone" : employee.phone,
            ":address" : employee.address,
            ":ssn" : employee.ssn,
            ":work_percentage" : employee.work_percentage,
            ":cash_percentage" : employee.cash_percentage,
            ":emp_id" : employee.id
        }).unwrap();
        
        HttpResponse::Ok()
            .content_type("application/json")
            .body(json!(employee).to_string())
    } else {
        HttpResponse::NotFound()
            .content_type("application/json")
            .body("Employee Not Found!")
    }
}

#[delete("/employees/{id}")]
async fn delete_employee(path: web::Path<i64>) -> HttpResponse {
    let id = path.into_inner(); // get path variable

    let conn: Connection = get_database_connection();
    match conn.execute("DELETE FROM employees where id = ?1", params![id.to_string()]) {
        Ok(_) => HttpResponse::Ok().body("Employee has been deleted!"),
        Err(err) => HttpResponse::NotFound().body(format!("Error: {:?}", err))
    }
}
// GET, POST, PUT, DELETE for Employee---------------

// GET, POST, PUT, DELETE for Work-------------------
#[derive(Debug, Serialize, Deserialize)]
struct WorkAPI {
    timestamp: String,
    total: f32,
    tip: f32,
    discount: f32,
    charged: f32,
    employee_work: Vec<EmployeeWork>
}

async fn get_work_time_range(date_from: String, date_to: String) -> Vec<Work>{
    let conn: Connection = get_database_connection();
    // grab data from database
    let mut work_stmt: Statement<'_> = conn.prepare("SELECT * FROM works WHERE timestamp BETWEEN ?1 AND ?2").unwrap();
    // map data to struct
    let rows_work = work_stmt.query_map(&[&date_from, &date_to], map_to_work).unwrap();
    // collect results into vector
    let mut vec_work: Vec<Work> = Vec::new();
    for row in rows_work {
        vec_work.push(row.unwrap());
    }

    return vec_work;
}

async fn get_detail_from_work_ids(work_ids: Vec<u8>) -> Vec<EmployeeWork> {
    let conn: Connection = get_database_connection();
    // grab data from database
    let mut emp_work_stmt: Statement<'_> = conn.prepare("SELECT * FROM employee_work WHERE work_id IN (?1)").unwrap();
    let rows_emp_work = emp_work_stmt.query_map(&[&work_ids], map_to_employee_work).unwrap();

    // collect results into vector
    let mut vec_emp_work: Vec<EmployeeWork> = Vec::new();
    for row in rows_emp_work {
        vec_emp_work.push(row.unwrap());
    }

    return vec_emp_work;
}   

#[get("/work")]
async fn get_all_works() -> HttpResponse {
    let conn: Connection = get_database_connection();

    // grab data from database
    let mut result_work: Statement<'_> = conn.prepare("SELECT * FROM works").unwrap();
    // map data to struct
    let rows_work = result_work.query_map([], map_to_employees).unwrap();
    // collect results into vector
    let mut json_vec_work = Vec::new();
    // Convert the vector of Employee to JSON
    for row in rows_work {
        json_vec_work.push(row.unwrap());
    }

    // do same for employee work


    let json = json!(&json_vec_work);
    HttpResponse::Ok()
        .content_type("application/json")
        .body(json.to_string())
}

// GET, POST, PUT, DELETE for Work-------------------

fn check_table_row_exist(table: String, row_id: String) -> bool {
    let conn: Connection = get_database_connection();
    conn.query_row(format!("SELECT COUNT(*) FROM {} WHERE id = {}", table, row_id).as_str(),
        [],
        |row| row.get::<_, i64>(0),)
        .unwrap() > 0
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
            .service(database_init)
            .service(database_format)
            .service(get_all_employees)
            .service(post_new_employee)
            .service(put_update_employee)
            .service(delete_employee)
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
use std::result;

use sqlx::{
    //mysql::MySqlPool,
    Pool,
    MySql,
    Error,
    MySqlPool, 
    FromRow,
    types::time::Date,
};

use time::macros::format_description;

use async_std::task;

#[derive(FromRow, Debug, Clone)]
struct Person {
    person_id: i32, //INT NOT NULL
    fname: String, //varchar(50) NOT NULL
    mname: Option<String>, //varchar(50)
    lname: String, //varchar(50) NOT NULL
    dob: Date, //date NOT NULL
    is_active: bool, //boolean NOT NULL
    //PRIMARY KEY(person_id)
}

#[derive(FromRow, Debug, Clone)]
struct EmpAttendance {
    person_id: i32, //INT NOT NULL
	time_in: Date, //datetime NOT NULL
    time_out: Date, //datetime NOT NULL
    //PRIMARY KEY(person_id)
}

#[derive(FromRow, Debug, Clone)]
struct EnrolledFingerprint {
    person_id: i32, //INT NOT NULL
	fprint_hash: String, //varchar(512) NOT NULL
    //PRIMARY KEY(person_id)
}

async fn connect() -> Result<Pool<MySql>, Error> {
    return MySqlPool::connect("mysql://root:pcb.2176310315865259@localhost:3306/employees").await; //insert url here
}

async fn do_test_connection() {
    let result = task::block_on(connect());

    match result {
        Err(err) => {
            println!("Cannot connect to database [{}]", err.to_string());
        }

        Ok(_) => {
            println!("Connected to database successfully.");
        }
    }
}

fn main() {
    task::block_on(do_test_connection());
}
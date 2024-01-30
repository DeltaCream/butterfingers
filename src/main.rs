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

use time::{macros::format_description, Time};

use async_std::task;

// #[derive(FromRow, Debug, Clone)]
// struct Person {
//     person_id: i32, //INT NOT NULL
//     fname: String, //varchar(50) NOT NULL
//     mname: Option<String>, //varchar(50)
//     lname: String, //varchar(50) NOT NULL
//     dob: Date, //date NOT NULL
//     is_active: bool, //boolean NOT NULL
//     //PRIMARY KEY(person_id)
// }

#[derive(FromRow, Debug, Clone)]
struct EmpRole {
    role_code: u16,
    role_name: String,
}

#[derive(FromRow, Debug, Clone)]
struct Employee {
    emp_id: u64,
    fname: String,
    mname: Option<String>,
    lname: String,
    dob: Date,
    doh: Date,
    role_code: u16,
    tin_num: u32,
    image: u64, //placeholder type for a binary type
}

// #[derive(FromRow, Debug, Clone)]
// struct EmpAttendance {
//     person_id: i32, //INT NOT NULL
// 	time_in: Date, //datetime NOT NULL
//     time_out: Date, //datetime NOT NULL
//     //PRIMARY KEY(person_id)
// }

#[derive(FromRow, Debug, Clone)]
struct EmpStatus {
    emp_id: u64,
    is_active: bool, //default true
    days_tenured: u64, //default 1
}

#[derive(FromRow, Debug, Clone)]
struct EmpLeaveStatus {
    emp_id: u64,
    on_leave: bool, //default false
    paid_leaves: u16, //default 5
}

#[derive(FromRow, Debug, Clone)]
struct ManagerialPosition {
    managerial_position_code: u16,
    position_name: String,
}

#[derive(FromRow, Debug, Clone)]
struct Management {
    emp_id: u64,
    managerial_position_code:  u16,
}

#[derive(FromRow, Debug, Clone)]
struct UserAccount {
    emp_id: u64,
    username: String,
    password: String,
}

#[derive(FromRow, Debug, Clone)]
struct StaffPosition {
    position_code: u16,
    position_name: String,
}

#[derive(FromRow, Debug, Clone)]
struct ProductionStaff {
    emp_id: u64,
    position_code: u16,
}

#[derive(FromRow, Debug, Clone)]
struct EnrolledFingerprint {
    emp_id: u64, //INT NOT NULL
	fprint_uuid: String, //varchar(512) NOT NULL varchar(36)
    //PRIMARY KEY(person_id)
}

#[derive(FromRow, Debug, Clone)]
struct AttendanceStatus {
    attendance_status_code: u16,
    attendance_status_meaning: String,
}

#[derive(FromRow, Debug, Clone)]
struct AttendanceRecord {
    emp_id: u64,
    attendance_date: Date,
    attendance_time: Time,
    attendance_status_code: u16,
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
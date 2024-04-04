use butterfingers::{
    verify,
    enroll,
};

// #[derive(FromRow, Debug, Clone)] old unused struct
// struct Person {
//     person_id: i32, //INT NOT NULL
//     fname: String, //varchar(50) NOT NULL
//     mname: Option<String>, //varchar(50)
//     lname: String, //varchar(50) NOT NULL
//     dob: Date, //date NOT NULL
//     is_active: bool, //boolean NOT NULL
//     //PRIMARY KEY(person_id)
// }

// #[derive(FromRow, Debug, Clone)] old unused struct
// struct EmpAttendance {
//     person_id: i32, //INT NOT NULL
// 	time_in: Date, //datetime NOT NULL
//     time_out: Date, //datetime NOT NULL
//     //PRIMARY KEY(person_id)
// }

// #[derive(FromRow, Debug, Clone)]
// struct EmpRole {
//     role_code: u16,
//     role_name: String,
// }

// #[derive(FromRow, Debug, Clone)]
// struct Employee {
//     emp_id: u64,
//     fname: String,
//     mname: Option<String>,
//     lname: String,
//     dob: Date,
//     doh: Date,
//     role_code: u16,
//     tin_num: u32,
//     image: Vec<u8>, //placeholder type for a BLOB type
// }



// #[derive(FromRow, Debug, Clone)]
// struct EmpStatus {
//     emp_id: u64,
//     is_active: bool, //default true
//     days_tenured: u64, //default 1
// }

// #[derive(FromRow, Debug, Clone)]
// struct EmpLeaveStatus {
//     emp_id: u64,
//     on_leave: bool, //default false
//     paid_leaves: u16, //default 5
// }

// #[derive(FromRow, Debug, Clone)]
// struct ManagerialPosition {
//     managerial_position_code: u16,
//     position_name: String,
// }

// #[derive(FromRow, Debug, Clone)]
// struct Management {
//     emp_id: u64,
//     managerial_position_code:  u16,
// }

// #[derive(FromRow, Debug, Clone)]
// struct UserAccount {
//     emp_id: u64,
//     username: String, //unique
//     user_password: String,
// }

// #[derive(FromRow, Debug, Clone)]
// struct StaffPosition {
//     position_code: u16,
//     position_name: String,
// }

// #[derive(FromRow, Debug, Clone)]
// struct ProductionStaff {
//     emp_id: u64,
//     position_code: u16,
// }

// #[derive(FromRow, Debug, Clone)]
// struct EnrolledFingerprint {
//     emp_id: u64, //INT NOT NULL
// 	fprint_uuid: String, //varchar(512) NOT NULL varchar(36)
//     //PRIMARY KEY(person_id)
// }

// #[derive(FromRow, Debug, Clone)]
// struct AttendanceStatus {
//     attendance_status_code: u16,
//     attendance_status_meaning: String,
// }

// #[derive(FromRow, Debug, Clone)]
// struct AttendanceRecord {
//     emp_id: u64,
//     attendance_date: Date,
//     attendance_time: Time,
//     attendance_status_code: u16,
// }

// async fn connect() -> Result<Pool<MySql>, Error> {
//     return MySqlPool::connect("mysql://root:root@localhost:3306/pyfi_db").await; //insert url here
// }

// async fn do_test_connection() {
//     let result = task::block_on(connect());

//     match result {
//         Err(err) => {
//             println!("Cannot connect to database [{}]", err);
//         }

//         Ok(_) => {
//             println!("Connected to database successfully.");
//         }
//     }
// }

// // async fn add_employee(pool: &MySqlPool, _image_link: &Path) -> Result<u64, Error> {
// //     // let key = "DATABASE_URL";
// //     // env::set_var(key, "mysql://root:root@localhost:3306/pyfi_db");

// //     //Insert employee, then obtain the ID of the row
// //     let emp_id = sqlx::query!( //use query_as! later on
// //         r#"
// // INSERT INTO employee(emp_id, fname, mname, lname, dob, doh, role_code, tin_num, image)
// // VALUES(1, "John", "Michael", "Doe", "2024-01-30", "2024-01-31", 2, 64, NULL)
// //         "# //idk what to put for the image column
// //     ,)
// //     .execute(pool)
// //     .await?
// //     .last_insert_id();

// //     Ok(emp_id)
// // }

// // async fn select(_query: &str) -> anyhow::Result<()> {
// //     dotenvy::dotenv()?;
// //     let pool = MySqlPool::connect(&env::var("DATABASE_URL")?).await?; 
// //     let _result = sqlx::query!("SELECT * FROM production_staff join employee using(emp_id) where production_staff.emp_id not in (select emp_id from enrolled_fingerprints)")
// //     .fetch_all(&pool)
// //     .await?;

// //     pool.close().await;
// //     Ok(())
// // }

//"INSERT INTO enrolled_fingerprints VALUES(emp_id, fprint_uuid)" <- some query I have to put later

// // #[tokio::main]
// // async fn main() -> anyhow::Result<()> {
// //     //task::block_on(do_test_connection());
// //     dotenvy::dotenv()?;
// //     let pool = MySqlPool::connect(&env::var("DATABASE_URL")?).await?; //MySqlPool::connect("mysql://root:root@localhost:3306/pyfi_db").await?;
// //     let insert_emp = add_employee(&pool, Path::new("random_pic")).await?;
// //     println!("Added employee with id {insert_emp}");
// //     Ok(())
// }
// }

use std::io::{self, Write};
use tokio::net::TcpStream;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use serde_json::json;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let target_host = "localhost";
    let target_port = 80;

    loop {
        let mut client = TcpStream::connect(format!("{}:{}", target_host, target_port)).await?;
        
        let mut input = String::new();
        print!("Message: ");
        io::stdout().flush()?; //flushes out everything stored in stdout to receive input with no problems
        io::stdin().read_line(&mut input)?; //read input and store in variable input of String type

        let message_body = json!({"message": input.trim()}); //message converted into a json Value type
        let message_string = serde_json::to_string(&message_body)?; //line above converted from json Value type to a string
        let message_length = message_string.len();
        let post_message = format!("POST /attendance/kiosk/api/sendMessage HTTP/1.1\r\nHost: {}:{}", target_host, target_port) +
            "\r\nContent-Type: application/json\r\n" +
            &format!("Content-Length: {}\r\nConnection: close\r\n\r\n{}", message_length, message_string); //the entire post_message
        
        println!("{}", post_message);

        client.write_all(post_message.as_bytes()).await?; //send post_message as u8 (bytes), with matching async support via await

        let mut response = vec![0; 1024]; //allocate an array of 1024 bytes for the response to be stored
        let n = client.read(&mut response).await?; //read a response to the client via the socket with async support via await
        println!("{}", String::from_utf8_lossy(&response[..n])); //print the response to the command line
    }

    //Ok(())
}
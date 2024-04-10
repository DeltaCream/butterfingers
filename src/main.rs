use std::thread::JoinHandle;
use std::{env, thread};
use std::io::{self, Write};
use std::net::SocketAddr;
use std::sync::Arc;
use butterfingers::{butterfingersd_enroll, butterfingersd_identify, identify};
use tokio::net::{TcpListener, TcpStream};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use serde_json::json;
use serde::Deserialize;
use tokio::sync::{mpsc, Mutex};
/*
Algorithm:

loop forever {
get message from stomp broker
add message to queue

dequeue message
if enroll:
check if we have a current task
if task is enroll, send the last message to resume state
if we do, send message to server
otherwise, spawn a thread to enroll

if verify:
check if we have a current task
if task is verify, send the last message to resume state
if not, send message to server
otherwise, spawn a thread to verify

if disconnect:
destroy thread

repeat loop
}
*/

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    //type "to" or "from" to determine which function to call
    dotenvy::dotenv()?;
    //let process_type = env::var("PROCESS_TYPE")?;
    //if process_type == "from" {
    //    println!("From Butterfingers...");
    //    from_butterfingers().await?;
    //} else if process_type == "to" {
      println!("To Butterfingers...");
      to_butterfingers().await?;
    //}
	
    Ok(())
}

//static mut currMode: Arc<Mutex<String>> = Arc::new(Mutex::new(String::from("none")));
async fn to_butterfingers() -> Result<(), Box<dyn std::error::Error>> {
    let curr_mode = Arc::new(Mutex::new(String::from("none")));
    //declare fp device here but not open it
    let thread_handle: Arc<Mutex<Option<thread::JoinHandle<()>>>> = Arc::new(Mutex::new(None));

    dotenvy::dotenv()?;
    let listen_host = env::var("LISTEN_HOST")?;
    let listen_port = env::var("LISTEN_PORT")?;

    let ip_port_addr = format!("{}:{}", listen_host, listen_port); //concat string

    println!("{}", ip_port_addr); //print formatted ip
    
    let addr = ip_port_addr.parse::<SocketAddr>().unwrap(); //convert socket addr
    //TcpListener::bind because we want to accept connections
    let server = TcpListener::bind(&addr).await.unwrap();
    println!("[*] Listening on {}", addr);
    
    loop {
        let (client, addr) = server.accept().await.unwrap();
        //let (tx, rx) = mpsc::channel(5);
        println!("[*] Accepted Connection from {}", addr);
        let mode_clone = curr_mode.clone();
        let thread_clone = thread_handle.clone();
        //pass fpdevice here
        tokio::spawn(async move {
            if let Err(e) = handle_client(client, thread_clone).await {
                eprintln!("Error handling client: {}", e);
            }
        });
        // thread::spawn(move || {
        //     handle_client(client, mode_clone)
        // });
    }
}
#[derive(Deserialize, Debug)]
struct ConnectionMessage {
	fingerprintMode: String,
	emp_id: Option<u64>
}
//response types:
//type 0 - connect success
//type 1 - disconnect success
//type 2 - error
//type 3 - plain message to identify mode
//type 4 - plain message to enroll mode
//static mut currMode: Option<String> = None;
async fn handle_client(mut client: TcpStream, handle: Arc<Mutex<Option<JoinHandle<()>>>>) -> Result<(), Box<dyn std::error::Error>> { //handle client function
    //let mut curr_mode = mode.lock().await;
    //println!("current mode before handler: {}", *curr_mode);
    let handle_inner = handle.lock().await;
    
    println!("Inside client handler");
    let mut buffer = [0; 1024]; //buffer for 1024 bytes
    println!("Passed buffer");
    let n = client.read(&mut buffer).await?; //read a response to the client via the socket
    println!("Passed read");
    let msg_from_client = String::from_utf8_lossy(&buffer[..n]); //get the message and decode it as a string
    println!("{}", msg_from_client);
    let c_msg: ConnectionMessage = serde_json::from_str(&msg_from_client).unwrap();
	//println!("Passed message processing");
    println!("[*] Received: {}", msg_from_client);
	let response;
	if c_msg.fingerprintMode == "disconnect"{
		//destroy co-routine
		//close fp device here
		*curr_mode = "none".to_string();
		response = json!({
            		"responseType": 1,
            		"responseMsg" : "Disconnection successful."
        	});
    	//} else if *curr_mode != "none"{ //if device is open, send error
        } else if handle_inner.is_
  
        	response = json!({
            		"responseType": 2,
            		"responseMsg" : "Another procedure is using the scanner!"
        	});
	} else if *curr_mode == "none" && c_msg.fingerprintMode == "enroll" {
		//call enroll and pass fpdevice handle and emp id
		if c_msg.emp_id.is_none() {
			response = json!({
            			"responseType": 2,
            			"responseMsg" : "Employee ID not specified!"
        		});
		} else {
			*curr_mode = "enroll".to_string();
			response = json!({
            			"responseType": 0,
            			"responseMsg" : "Enrollment mode started."
        		});
		}
		
	} else if *curr_mode == "none" && c_msg.fingerprintMode == "identify" {
		//call identify and pass fpdevice handle
		*curr_mode = "identify".to_string();
        //identify().await;
        //identify().await;
        thread::spawn(|| {
            identify(); 
        });
		response = json!({
            		"responseType": 0,
            		"responseMsg" : "Attendance mode started."
        	});

	} else {
        	response = json!({
            		"responseType": 2,
            		"responseMsg": "Default Message"
        	});
    } 
    //if let Some(serde_json::Value) = response {
    client.write_all(response.to_string().as_bytes()).await?;
    //}else {
    //    println!("How is it possible that the response is empty?");
    //}
    //let response = "Server Acknowledged!";
 
    println!("current mode after handler: {}", curr_mode);


    Ok(())
}



async fn from_butterfingers() -> Result<(), Box<dyn std::error::Error>> {
    dotenvy::dotenv()?;
    let target_host = &env::var("TARGET_HOST")?;
    let target_port = &env::var("TARGET_PORT")?;
    loop {
        //TcpStream::connect because we want to connect to the server
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
        
        println!("{}", post_message); //print the post message to the command line

        client.write_all(post_message.as_bytes()).await?; //send post_message as u8 (bytes), with matching async support via await

        let mut response = vec![0; 1024]; //allocate an array of 1024 bytes for the response to be stored
        let n = client.read(&mut response).await?; //read a response to the client via the socket with async support via await
        println!("{}", String::from_utf8_lossy(&response[..n])); //print the response to the command line
    }
    
    //Ok(())
}

//will listen whether enroll or verify (asynchronously)
//spawn a thread to enroll
//spawn a thread to verify

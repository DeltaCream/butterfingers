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
use libfprint_rs::{
        FpContext, 
        FpPrint, 
        FpDevice,
        Cancellable,
    };
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
        //let mode_clone = curr_mode.clone();
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

//response types:
//type 0 - connect success
//type 1 - disconnect success
//type 2 - error
//type 3 - plain message to identify mode
//type 4 - plain message to enroll mode
//type 5 - special message to identify mode
//type 6 - special message to identify mode
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
    let c_msg: butterfingersd_identify::ConnectionMessage = serde_json::from_str(&msg_from_client).unwrap();
	//println!("Passed message processing");
    println!("[*] Received: {}", msg_from_client);
	let response;
	if c_msg.fingerprintMode == "disconnect"{
		//destroy co-routine
		if !handle_inner.is_none()  {
			//use mprc to end function of thread
			handle_inner.join();
			*handle_inner = None;
		}
		
		//close fp device here
		//Get FpContext to get devices
        	let context = FpContext::new();
        	//Use FpContext to get devices (returns a vector/array of devices)
        	let devices = context.devices();
        	for device in devices {
        		if device.is_open(){
        			device.close_sync(None).unwrap();
        		}
        	}
		//*curr_mode = "none".to_string();
		response = json!({
            		"responseType": 1,
            		"responseMsg" : "Disconnection successful."
        	});
    	//} else if *curr_mode != "none"{ //if device is open, send error
        } else if !handle_inner.is_none()  {
        	response = json!({
            		"responseType": 2,
            		"responseMsg" : "Another procedure is using the scanner!"
        	});
	} else if handle_inner.is_none()  && c_msg.fingerprintMode == "enroll" {
		//call enroll and pass fpdevice handle and emp id
		if c_msg.emp_id.is_none() {
			response = json!({
            			"responseType": 2,
            			"responseMsg" : "Employee ID not specified!"
        		});
		} else {
			//*curr_mode = "enroll".to_string();
			response = json!({
            			"responseType": 0,
            			"responseMsg" : "Enrollment mode started."
        		});
		}
		
	} else if handle_inner.is_none() && c_msg.fingerprintMode == "identify" {
		//call identify and pass fpdevice handle
		//*curr_mode = "identify".to_string();
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
 
    //println!("current mode after handler: {}", curr_mode);


    Ok(())
}







//will listen whether enroll or verify (asynchronously)
//spawn a thread to enroll
//spawn a thread to verify

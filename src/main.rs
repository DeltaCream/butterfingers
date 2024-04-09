use std::env;
use std::io::{self, Write};
use std::net::SocketAddr;
use tokio::net::{TcpListener, TcpStream};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use serde_json::json;

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
    let process_type = env::var("PROCESS_TYPE")?;
    if process_type == "from" {
        println!("From Butterfingers...");
        from_butterfingers().await?;
    } else if process_type == "to" {
        println!("To Butterfingers...");
        to_butterfingers().await?;
    }
    Ok(())
}


async fn to_butterfingers() -> Result<(), Box<dyn std::error::Error>> {
    dotenvy::dotenv()?;
    let listen_host = env::var("LISTEN_HOST")?;
    let listen_port = env::var("LISTEN_PORT")?;

    let ip_port_addr = format!("{}:{}", listen_host, listen_port);

    println!("{}", ip_port_addr);

    let addr = ip_port_addr.parse::<SocketAddr>().unwrap();
    //TcpListener::bind because we want to accept connections
    let server = TcpListener::bind(&addr).await.unwrap();
    println!("[*] Listening on {}", addr);
    loop {
        let (client, addr) = server.accept().await.unwrap();
        println!("[*] Accepted Connection from {}", addr);

        tokio::spawn(async move {
            if let Err(e) = handle_client(client).await {
                eprintln!("Error handling client: {}", e);
            }
        });
    }
}

async fn handle_client(mut client: TcpStream) -> Result<(), Box<dyn std::error::Error>> { //handle client function
    println!("Inside client handler");
    let mut buffer = [0; 1024]; //buffer for 1024 bytes
    println!("Passed buffer");
    let n = client.read(&mut buffer).await?; //read a response to the client via the socket
    println!("Passed read");
    let msg_from_client = String::from_utf8_lossy(&buffer[..n]); //get the message and decode it as a string
    println!("Passed message processing");
    println!("[*] Received: {}", msg_from_client);

    let response = "Server Acknowledged!";
    client.write_all(response.as_bytes()).await?;
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
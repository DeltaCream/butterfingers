use butterfingers::{
    verify,
    enroll,
};

use std::io::{self, Write};
use tokio::net::TcpStream;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use serde_json::json;
use futures::prelude::*;
use tokio_stomp_2::client;
use tokio_stomp_2::FromServer;

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

    /*
    Stomp Client version - receive stuff

    let mut conn = client::connect("127.0.0.1:61613", None, None).await.unwrap();
    //another example used this:
     let mut conn = client::connect(
        "127.0.0.1:61613",
        "/".to_string(),
        "guest".to_string().into(),
        "guest".to_string().into(),
    ).await.unwrap();

    conn.send(client::subscribe("queue.test", "custom-subscriber-id")).await.unwrap();

    while let Some(item) = conn.next().await {
        if let FromServer::Message { message_id,body, .. } = item.unwrap().content {
            println!("{:?}", body);
            println!("{}", message_id);
        }
    }
    Ok(())
    */
    //Ok(())
}

async fn send_message() {
    let target_host = "localhost";
    let target_port = 80;

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

fn handle_request(mut stream: TcpStream) {
    let mut buffer = [0; 1024];
    stream.read(&mut buffer).unwrap();
    let request = json!(&buffer);
    println!("Request: {}", String::from_utf8_lossy(&buffer[..]));
}

//will listen whether enroll or verify (asynchronously)
//spawn a thread to enroll
//spawn a thread to verify
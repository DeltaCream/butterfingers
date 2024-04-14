const { invoke } = window.__TAURI__.tauri;
const { emit, listen } = window.__TAURI__.event;

let resultString = document.querySelector("#result-body");

async function start_identify() {
  try {
    const response = await invoke("start_identify");
    const result = JSON.parse(response);

    // check if response has error key
    if (result && result.error) {
      console.error("Error: ", result.error);
    } else {
      console.log("Response: ", result);
      resultString.textContent = `Employee ID: ${result.emp_id}\nName: ${result.name}\nTime In: ${result.time_in}`
    }
  } catch (err) {
    console.error("Error invoking start_identify: ", err);
  }
}


/*
let result;
window.addEventListener("DOMContentLoaded", () => {
  // greetInputEl = document.querySelector("#greet-input");
  // greetMsgEl = document.querySelector("#greet-msg");
  document.querySelector("form").addEventListener("submit", (e) => {
    e.preventDefault();
  });
  startEventListener();
});

async function startEventListener(){
  let str = "";
  result = document.querySelector("#result-body");
  while(true){
    str = await invoke("start_identify");
    console.log(str);
    result.textContent = str;
  }
    
}
*/
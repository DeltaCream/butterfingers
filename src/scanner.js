const { invoke } = window.__TAURI__.tauri;
const { emit, listen } = window.__TAURI__.event;

let resultString = document.querySelector("#result-body");
const revert = 7500; // time until text revert in ms

async function start_identify() {
  try {
    const response = await invoke("start_identify");
    const result = JSON.parse(response);

    // check if response has error key
    if (result && result.responsecode === "success") {
      const data = result.body;
      resultString.textContent = JSON.stringify(data);
      console.log("Response: ", JSON.stringify(data));

      revertText();
    } else {
      resultString.textContent = "<span class=\"error\">", result.body, "</span>";
      console.error("Error in response: ", result.body);

      revertText();
    }
  } catch (err) {
    resultString.textContent = "<span class=\"error\">", result.body, "</span>";
    console.error("Error invoking start_identify: ", result.body);

    revertText();
  }
}

function revertText() {
  // revert text after [revert] ms
  setTimeout(() => {
    resultString.textContent = "Awaiting connection to scanner...";
  }, revert);
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
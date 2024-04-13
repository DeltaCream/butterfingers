const { invoke } = window.__TAURI__.tauri;
const { emit,listen } = window.__TAURI__.event ;
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

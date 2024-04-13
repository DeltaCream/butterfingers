const { invoke } = window.__TAURI__.tauri;
const { emit,listen } = window.__TAURI__.event ;

window.addEventListener("DOMContentLoaded", () => {
  // greetInputEl = document.querySelector("#greet-input");
  // greetMsgEl = document.querySelector("#greet-msg");
  document.querySelector("form").addEventListener("submit", (e) => {
    e.preventDefault();
    greet();
  });
  startEventListener();
});
window.addEventListener("beforeunload", () =>{
  window.unlisten();
});

async function greet(){
  await invoke("start_identify");
}
async function startEventListener(){
    window.unlisten = await listen('identify-messages', (event) => {
      console.log("response: "+event.payload);
      $("#result-body").text(event.payload);
    });
    console.log(window.unlisten);
    await invoke("start_identify");
}

const { invoke } = window.__TAURI__.tauri;
const { emit, listen } = window.__TAURI__.event;

/* JSON STRUCTURE
responsecode: "success" | "failure"
body: {
  id
  fname
  lname
  time
  date
  code
}
 */

let resultString = document.querySelector("#result-body");
let btnIdentify = document.querySelector("#identify");
let btnManual = document.querySelector("#manual");
const revert = 7500; // time until text revert in ms

// identify fingerprint
async function start_identify() {
  try {
    const response = await invoke("start_identify");
    const result = JSON.parse(response);

    // check if response has error key
    if (result && result.responsecode === "success") {
      const data = result.body;
      resultString.textContent = "<span class=\"success\">Attendance Recorded!</span>";

      // change data
      document.querySelector("#employee-image").src = "http://192.168.254.191/attendance/images/" + data[0];
      document.querySelector("#employee").textContent = "<span class=\"success\">" + data[1] + " " + data[2] + "</span>";
      document.querySelector("#employee-id").textContent = data[0];
      document.querySelector("#date").textContent = data[4];
      document.querySelector("#time").textContent = data[3];
      if (data[5] === "in") {
        document.querySelector("#code").textContent = "Time-In";
      } else {
        document.querySelector("#code").textContent = "Time-Out";
      }

      // popup employee window
      showPopup();

      console.log("Response: " + JSON.stringify(data));

      revertText();
    } else {
      resultString.textContent = "<span class=\"error\">" + result.body + "</span>";
      console.error("Error in response: " + result.body);

      revertText();
    }
  } catch (err) {
    resultString.textContent = "<span class=\"error\">" + result.body + "</span>";
    console.error("Error invoking start_identify: " + result.body);

    revertText();
  }
}

// manual attendance
let inputID = document.querySelector("#emp_id");
async function manual_attendance() {
  try {
    const emp_id = inputID.value;
    console.log("Before manual invoke");
    console.log(inputID.value);
    const response = await invoke("manual_attendance", { emp: emp_id }); //await invoke("manual_attendance", { emp: inputID.value });
    console.log("After manual invoke");
    const result = JSON.parse(response);

    if (result && result.responsecode === "success") {
      const data = result.body;
      resultString.textContent = "<span class=\"success\">Attendance Recorded!</span>";

      // change data
      document.querySelector("#employee-image").src = "http://192.168.254.191/attendance/images/" + data[0];
      document.querySelector("#employee").textContent = "<span class=\"success\">" + data[1] + " " + data[2] + "</span>";
      document.querySelector("#employee-id").textContent = data[0];
      document.querySelector("#date").textContent = data[4];
      document.querySelector("#time").textContent = data[3];
      if (data[5] === "in") {
        document.querySelector("#code").textContent = "Time-In";
      } else {
        document.querySelector("#code").textContent = "Time-Out";
      }

      // popup employee window
      showPopup();

      console.log("Response: " + JSON.stringify(data));

      revertText();
    } else {
      resultString.textContent = "<span class=\"error\">" + result.body + "</span>";
      console.error("Error in response: " + result.body);

      revertText();
    }

  } catch (err) {
    resultString.textContent = "<span class=\"error\">", result.body, "</span>";
    console.error("Error invoking start_identify: ", result.body);

    revertText();
  }
}

// disable buttons and revert text back after a time
function revertText() {
  // disable buttons
  btnIdentify.disabled = true;
  btnManual.disabled = true;

  // revert text after [revert] ms
  setTimeout(() => {
    resultString.textContent = "Awaiting connection to scanner...";
    btnIdentify.disabled = false;
    btnManual.disabled = false;
  }, revert);
}

// popup
function showPopup() {
  const popup = document.querySelector("#popup");
  popup.style.display = "block";
  setTimeout(() => {
    closePopup();
  }, revert);
}

function closePopup() {
  const popup = document.querySelector("#popup");
  popup.style.display = "none";
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
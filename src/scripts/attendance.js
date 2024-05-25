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
let isOnIdentify = false;
const revert = 3000; // time until text revert in ms
const imageURL = "http://localhost/attendance/images/"; // change ip address

window.addEventListener("DOMContentLoaded", () => {
    let result = load_fingerprints();
    console.log(result);
});

async function load_fingerprints() {
    return await invoke("load_fingerprints");
}

async function cancel_identify() {
    console.log("Identify cancelling");
    await invoke("cancel_function");
}
// identify fingerprint
async function start_identify() {
    if (isOnIdentify == false) {
        try {
            isOnIdentify = true;
            btnManual.disabled = true;
            btnIdentify.textContent = "Cancel Scan";
            const response = await invoke("start_identify");
            const result = JSON.parse(response);

            // check if response has error key
            if (result && result.responsecode === "success") {
                const data = result.body;
                resultString.innerHTML =
                    '<span class="success">Attendance Recorded!</span>';

                // change data
                document.querySelector("#employee-image").src =
                    imageURL + data[0];
                document.querySelector("#employee").innerHTML =
                    '<span class="success">' +
                    data[1] +
                    " " +
                    data[2] +
                    "</span>";
                document.querySelector("#employee-id").textContent = data[0];
                document.querySelector("#date").textContent = data[4];
                document.querySelector("#time").textContent = data[3];
                document.querySelector("#code").textContent = data[5];

                // popup employee window
                showPopup();

                console.log("Response: " + JSON.stringify(data));

                revertText();
            } else {
                resultString.innerHTML =
                    '<span class="error">' + result.body + "</span>";
                console.error("Error in response: " + result.body);

                revertText();
            }

            document.querySelector("#emp_id").value = "";
        } catch (err) {
            resultString.innerHTML =
                '<span class="error">' + result.body + "</span>";
            console.error("Error invoking start_identify: " + result.body);
            revertText();
        } finally {
            isOnIdentify = false;
            btnIdentify.textContent = "Begin Scan";
        }
    } else {
        cancel_identify();
        isOnIdentify = false;
        btnIdentify.textContent = "Begin Scan";
        revertText();
    }
}

// manual attendance
let inputID = document.querySelector("#emp_id");
async function manual_attendance() {
    try {
        const emp_id = inputID.value;
        //console.log("Before manual invoke");
        console.log(inputID.value);
        const response = await invoke("manual_attendance", {
            emp: String(emp_id),
        }); //await invoke("manual_attendance", { emp: inputID.value });
        //console.log("After manual invoke");
        const result = JSON.parse(response);

        if (result && result.responsecode === "success") {
            const data = result.body;
            resultString.innerHTML =
                '<span class="success">Attendance Recorded!</span>';

            // change data
            document.querySelector("#employee-image").src = imageURL + data[0];
            document.querySelector("#employee").innerHTML =
                '<span class="success">' + data[1] + " " + data[2] + "</span>";
            document.querySelector("#employee-id").textContent =
                "ID: " + data[0];
            document.querySelector("#date").textContent = "Date: " + data[4];
            document.querySelector("#time").textContent =
                "Time: " + formatTime(data[3]);
            // if (data[5] == 1) {
            //   document.querySelector("#code").textContent = "Action: Time-In";
            // } else {
            //   document.querySelector("#code").textContent = "Action: Time-Out";
            // }
            document.querySelector("#code").textContent = data[5];

            // popup employee window
            showPopup();

            console.log("Response: " + JSON.stringify(data));

            revertText();
        } else {
            resultString.innerHTML =
                '<span class="error">' + result.body + "</span>";
            console.error("Error in response: " + result.body);

            revertText();
        }

        document.querySelector("#emp_id").value = "";
    } catch (err) {
        (resultString.innerHTML = '<span class="error">'),
            result.body,
            "</span>";
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
        resultString.textContent =
            'Press "Begin Scan" to scan your right index finger for attendance.';
        btnIdentify.disabled = false;
        btnManual.disabled = false;
    }, revert);
}

// popup
function showPopup() {
    const popup = document.querySelector("#popup");
    popup.style.display = "flex";
    setTimeout(() => {
        closePopup();
    }, revert);
}

function closePopup() {
    const popup = document.querySelector("#popup");
    popup.style.display = "none";
}

function formatTime(timeString) {
    const [hourString, minute, seconds] = timeString.split(":");
    const hour = +hourString % 24;
    return (
        (hour % 12 || 12) +
        ":" +
        minute +
        ":" +
        seconds +
        " " +
        (hour < 12 ? "AM" : "PM")
    );
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

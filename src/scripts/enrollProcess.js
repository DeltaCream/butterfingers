const { invoke } = window.__TAURI__.tauri;
const { listen } = window.__TAURI__.event;

window.addEventListener("DOMContentLoaded", () => {
    document.getElementById("cancel").onclick = function () {
        cancel_enroll();
    };

    let queryString = window.location.search; //get raw url params
    console.log(queryString);
    let urlParams = new URLSearchParams(queryString); //parse url params
    let id = urlParams.get('id'); //get id param
    let empname = urlParams.get('empname');
    console.log(id);
    if (id == null) { //check if id exists
        //show error
        document.getElementById("proc-num").innerHTML = "no id";
    } else {
        get_enroll_stages(id, empname); //how many enroll stages will the scanner perform
        //print "Please press your finger (stages) times making sure it blinks each time."
        //return if success
        //invoke('');
    }
});

async function cancel_enroll() {
    await invoke('cancel_function'); //they use the same underlying tauri command to cancel fingerprint-related stuff
}

//let btnCancel = document.querySelector("#cancel");

async function get_enroll_stages(id, empname) {
    //let stages = await invoke('get_device_enroll_stages');
    let stages = 5;
    console.log("Device enroll stages received");
    document.getElementById("proc-num").innerHTML = "Please press your right index finger at the fingerprint scanner " + stages + " times, making sure it blinks each time.";
    // btnCancel.style.display = "block";
    document.getElementById("cancel").style.display = "block";
    let response = await invoke('enroll_proc', { emp: id });

    const result = JSON.parse(response);

    if (result && result.responsecode === "success") {
        document.getElementsByClassName('process')[0].style.display = "none";
        document.getElementsByClassName('success')[0].style.display = "flex";
        let fempname = empname.replace('-', ' ');
        document.getElementById("succ-id").innerHTML = "For Employee: " + fempname + " ID: " + id;
    } else {
        console.log("ERROR: " + result.body);
        //if (result.body == "Fingerprint scan cancelled")
        document.getElementById("proc-num").innerHTML = "ERROR: " + result.body;
        document.getElementById("cancel").style.display = "none";
        //btnCancel.style.display = "none";
        document.getElementsByClassName('failure')[0].style.display = "flex"; //added line
    }
}
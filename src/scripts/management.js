const { invoke } = window.__TAURI__.tauri;
const { listen } = window.__TAURI__.event;
const dialog = window.__TAURI__.dialog;

window.addEventListener("DOMContentLoaded", () => {
    document.getElementById("search").onkeyup = function () {
        searchEmp();
    };

    document.getElementById("verify").onclick = function () {
        verifyEmp();
    }

    document.getElementById("delete").onclick = function () {
        deleteEmp();
    }

    enumerate_enrolled_employees();
});

// Verify Fingerprint
async function verifyEmp() {

    const selectedRadio = document.querySelector('input[name="emp"]:checked');
    if (selectedRadio == null) {
        // console.log("no employee selected");
        document.getElementById("selected").innerHTML = "<span class=\"error\">No Employee Selected</span>";
        return;
    }
    const employee = selectedRadio.id.split("_");
    const emp_id = employee[0];
    const emp_fname = employee[1];
    const emp_lname = employee[2];

    const confirmed = await dialog.confirm("Verify " + emp_fname + " " + emp_lname + "'s fingerprint?",
        { title: "Confirm Verify", okLabel: "Yes", });

    if (confirmed) {
        // TODO: Put the INVOKE here
        document.getElementById("selected").innerHTML = "Verify Pressed: " + " " + emp_fname + " " + emp_lname + " | " + emp_id;
    }

    console.log("verify pressed: " + emp_id);
}

async function deleteEmp() {
    const selectedRadio = document.querySelector('input[name="emp"]:checked');
    if (selectedRadio == null) {
        // console.log("no employee selected");
        document.getElementById("selected").innerHTML = "<span class=\"error\">No Employee Selected</span>";
        return;
    }
    const employee = selectedRadio.id.split("_");
    const emp_id = employee[0];
    const emp_fname = employee[1];
    const emp_lname = employee[2];


    const confirmed = await dialog.confirm("Delete " + emp_fname + " " + emp_lname + "'s fingerprint?",
        { title: "Confirm Delete", okLabel: "Yes", });

    if (confirmed) {
        // TODO: Put the INVOKE here
        document.getElementById("selected").innerHTML = "<span class=\"error\">Deleted " + emp_fname + " " + emp_lname + "'s fingerprint</span>";
    }

    console.log("delete pressed: " + emp_id);
}

async function enumerate_enrolled_employees() {
    //await invoke('count');
    let results = await invoke('enumerate_enrolled_employees'); // TODO: Change this invoke to enrolled employees
    let results_json = JSON.parse(results);
    // console.log(results_json);

    if (results_json.length == 0) {
        document.getElementById("item-list").innerHTML = "<span class=\"error no-employees\">No enrolled employees</span>";
        return;
    }
    for (var i = 0; i < results_json.length; i++) { //loop for each element
        var emp = results_json[i];
        if (emp.hasOwnProperty("error")) {
            console.log("error: " + emp['error']);
            //hindi nag didisplay error sa page?
            return;
        }
        if (emp.hasOwnProperty('emp_id')) {
            console.log("emp_id: " + emp['emp_id']);
        }
        if (emp.hasOwnProperty('fname')) {
            console.log("first name : " + emp['fname']);
        }
        if (emp.hasOwnProperty('lname')) {
            console.log("last name: " + emp['lname']);
        }

        if (emp.hasOwnProperty('emp_id') && emp.hasOwnProperty('fname') && emp.hasOwnProperty('lname')) {
            addToList(emp['emp_id'], emp['fname'], emp['lname']);
        }
    }

}

// render items from enumerate() into page list
function addToList(id, fname, lname) {
    var itemList = document.getElementById("item-list");
    var div = document.createElement("div");
    var input = document.createElement("input");
    var label = document.createElement("label");

    div.setAttribute("class", "item");

    input.setAttribute("type", "radio");
    input.setAttribute("name", "emp");
    input.setAttribute("value", id);
    input.setAttribute("id", id + "_" + fname + "_" + lname);
    input.setAttribute("class", "emp-radio");

    label.setAttribute("for", id);
    label.setAttribute("class", "item-label");

    var p1 = document.createElement("p");
    var p2 = document.createElement("p");
    p1.appendChild(document.createTextNode("ID: " + id));
    p1.setAttribute("class", "emp-id")

    p2.appendChild(document.createTextNode("NAME: " + fname + " " + lname));
    p2.setAttribute("class", "emp-name")

    label.appendChild(p1);
    label.appendChild(p2);
    label.setAttribute("id", id);

    div.appendChild(input);
    div.appendChild(label);

    div.onclick = function () {
        check(div, id, fname, lname);
    }

    itemList.appendChild(div);
}

// behaviour when checking a radio button
function check(div, id, fname, lname) {
    var items = document.querySelectorAll('.item');

    for (var i = 0; i < items.length; i++) {
        items[i].classList.remove('checked');
    }

    var radio = div.querySelector('.emp-radio');
    radio.checked = true;
    div.classList.add('checked');

    document.getElementById("selected").innerHTML = "Selected: " + fname + " " + lname + " | " + id;
}

function searchEmp() {
    // Declare variables
    var input, filter, items, name, id, i, txtValueName, txtValueId;
    input = document.getElementById('search');
    filter = input.value.toUpperCase();
    items = document.getElementsByClassName('item');

    // Loop through all list items, and hide those who don't match the search query
    for (i = 0; i < items.length; i++) {
        name = items[i].getElementsByClassName("emp-name")[0]; // target emp-name
        id = items[i].getElementsByClassName("emp-id")[0]; // target emp-id
        txtValueName = name.textContent || name.innerText;
        txtValueId = id.textContent || id.innerText;
        if (txtValueName.toUpperCase().indexOf(filter) > -1 || txtValueId.toUpperCase().indexOf(filter) > -1) {
            items[i].style.display = "";
        } else {
            items[i].style.display = "none";
        }
    }
}
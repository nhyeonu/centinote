function setFormWarning(description, focus_element) {
    const container = document.getElementById("warning-container");
    const paragraph = document.getElementById("warning-paragraph");

    container.hidden = false;
    paragraph.innerHTML = description;

    focus_element.focus();
}

let method;
let target;

function submitJson(form) {
    const title_element = document.getElementById("title");
    const body_element = document.getElementById("body");
    const submit_element = document.getElementById("submit");

    let xhr = new XMLHttpRequest();
    xhr.open(method, target);
    xhr.setRequestHeader("Accept", "application/json");
    xhr.setRequestHeader("Content-Type", "application/json");

    xhr.onreadystatechange = function() {
        if(xhr.readyState == 4) {
            if(xhr.status > 99 && xhr.status < 300) {
                window.location.href = "/timeline.html";
            } else {
                setFormWarning(
                    "Something has gone wrong! " +
                    "Please contact the server admin if the problem persists.",
                    submit_element);
            }
        }
    };

    const data = {};
    data.title = title_element.value;
    data.body = body_element.value;

    if(method == "POST") {
        data.timezone_offset = new Date().getTimezoneOffset();
    }

    xhr.send(JSON.stringify(data));
}

function get_cookie_value(target_name) {
    const cookies = document.cookie.split("; ");
    const target_cookie = cookies.find((cookie) => cookie.startsWith(target_name + "="));
    const value = target_cookie?.split("=")[1];
    return value;
}

const query_string = window.location.search;
const url_parameters = new URLSearchParams(query_string);

const user_uuid = get_cookie_value("user_uuid");
const entry_uuid = url_parameters.get("entry-uuid");

function deleteEntry() {
    if(confirm("Do you really want to delete this entry?")) {
        const xhr = new XMLHttpRequest();
        xhr.open("DELETE", "/api/users/" + user_uuid + "/journals/" + entry_uuid)
        xhr.onreadystatechange = function() {
            if(xhr.readyState == 4) {
                if(xhr.status > 99 && xhr.status < 300) {
                    window.location.href = "/timeline.html";
                } else {
                    const submit_element = document.getElementById("submit");
                    setFormWarning(
                        "Something has gone wrong! " +
                        "Please contact the server admin if the problem persists.",
                        submit_element);
                }
            }
        };

        xhr.send();
    }
}

if(entry_uuid != null) {
    document.getElementById("delete-button").hidden = false;

    method = "PATCH";
    target = "/api/users/" + user_uuid + "/journals/" + entry_uuid;

    const xhr = new XMLHttpRequest();
    xhr.open("GET", target);
    xhr.setRequestHeader("Accept", "application/json");
    xhr.setRequestHeader("Content-Type", "application/json");

    xhr.onreadystatechange = function() {
        if(this.readyState == 4 && this.status == 200) {
            const entry = JSON.parse(xhr.response);
            document.getElementById("title").value = entry.title;
            document.getElementById("body").value = entry.body;
        }
    };

    xhr.send();
} else {
    method = "POST";
    target = "/api/users/" + user_uuid + "/journals";
}

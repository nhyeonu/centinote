let method;
let target;

function submitJson(form) {
    let xhr = new XMLHttpRequest();
    xhr.open(method, target);
    xhr.setRequestHeader("Accept", "application/json");
    xhr.setRequestHeader("Content-Type", "application/json");

    xhr.onreadystatechange = function() {
        if(this.readyState == 4 && this.status > 99 && this.status < 300) {
            window.location.href = "/timeline.html";
        }
    };

    const title_element = document.getElementById("title");
    const body_element = document.getElementById("body");

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

if(entry_uuid != null) {
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

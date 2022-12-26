function submitForm(form) {
    let xhr = new XMLHttpRequest();
    xhr.open(form.getAttribute("method"), form.getAttribute("action"));

    xhr.setRequestHeader("Accept", "application/json");
    xhr.setRequestHeader("Content-Type", "application/json");

    var data = {};

    let inputs = form.getElementsByTagName("input");
    for (const input of inputs) {
        const data_type = input.getAttribute("data-type");
        if(data_type == "integer") {
            data[input.getAttribute("name")] = parseInt(input.value, 10);
        } else {
            data[input.getAttribute("name")] = input.value;
        }
    }

    let textareas = form.getElementsByTagName("textarea");
    for (const textarea of textareas) {
        data[textarea.getAttribute("name")] = textarea.value;
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
const editor_form = document.getElementById("editor-form");

if(editor_form !== null) {
    const entry_uuid = url_parameters.get("entry-uuid");

    if(entry_uuid !== null) {
        editor_form.setAttribute("method", "PUT");
        editor_form.setAttribute("action", "/api/journals/" + entry_uuid);
    } else {
        editor_form.setAttribute("method", "POST");
        editor_form.setAttribute("action", "/api/users/" + user_uuid + "/journals");
    }

    const timezone_offset = document.getElementById("timezone_offset");
    timezone_offset.setAttribute("value", new Date().getTimezoneOffset());
}

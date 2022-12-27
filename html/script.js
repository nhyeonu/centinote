function submitForm(form) {
    let xhr = new XMLHttpRequest();
    xhr.open(form.getAttribute("method"), form.getAttribute("action"));

    xhr.setRequestHeader("Accept", "application/json");
    xhr.setRequestHeader("Content-Type", "application/json");

    xhr.onreadystatechange = function() {
        if(this.readyState == 4 && this.status > 99 && this.status < 300) {
            window.location.replace(form.getAttribute("data-redirect"));
        }
    };

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

const months = [
    "January",
    "February",
    "March",
    "April",
    "May",
    "June",
    "July",
    "August",
    "September",
    "October",
    "November",
    "December"
];

const query_string = window.location.search;
const url_parameters = new URLSearchParams(query_string);

const user_uuid = get_cookie_value("user_uuid");

const editor_form = document.getElementById("editor-form");
if(editor_form != null) {
    const entry_uuid = url_parameters.get("entry-uuid");

    if(entry_uuid != null) {
        editor_form.setAttribute("method", "PUT");
        editor_form.setAttribute("action", "/api/journals/" + entry_uuid);

        const entry_xhr = new XMLHttpRequest();
        entry_xhr.onreadystatechange = function() {
            if(this.readyState == 4 && this.status == 200) {
                const entry = JSON.parse(entry_xhr.response);
                document.getElementById("title").value = entry.title;
                document.getElementById("body").value = entry.body;
            }
        };
        entry_xhr.open("GET", "/api/users/" + user_uuid + "/journals/" + entry_uuid);
        entry_xhr.send();
    } else {
        editor_form.setAttribute("method", "POST");
        editor_form.setAttribute("action", "/api/users/" + user_uuid + "/journals");
    }

    const timezone_offset = document.getElementById("timezone_offset");
    timezone_offset.setAttribute("value", new Date().getTimezoneOffset());
}

const timeline = document.getElementById("timeline");
function processAnotherEntry(uuids, i) {
    if(i < 0) {
        return;
    }

    const entry_xhr = new XMLHttpRequest();

    entry_xhr.onreadystatechange = function() {
        if(this.readyState == 4 && this.status == 200) {
            const entry = JSON.parse(entry_xhr.response);

            let group = document.getElementById(
                entry.created.slice(0, 10) + 
                " " +
                entry.created.slice(26)
            );

            if(group == null) {
                group = document.createElement("div");
                group.id = entry.created.slice(0, 10) + " " + entry.created.slice(26);
                group.classList.add("group");

                if(timeline.firstChild == null) {
                    timeline.appendChild(group);
                } else {
                    timeline.insertBefore(group, timeline.firstChild);
                }

                let timedate_header = document.createElement("h2");
                timedate_header.innerHTML = 
                    months[parseInt(entry.created.slice(5, 7)) - 1] + 
                    " " +
                    entry.created.slice(8, 10) + 
                    " " +
                    entry.created.slice(0, 4);

                group.appendChild(timedate_header);
            }

            let div = document.createElement("div");
            group.appendChild(div);

            let link = document.createElement("a");
            link.classList.add("link");
            link.setAttribute("href", "/editor.html?entry-uuid=" + uuids[i]);
            div.appendChild(link);

            let title = document.createElement("h3");
            title.innerHTML = entry.title;
            link.appendChild(title);

            let body = document.createElement("p");
            body.innerHTML = entry.body;
            div.appendChild(body);

            processAnotherEntry(uuids, i - 1);
        }
    };

    entry_xhr.open("GET", "/api/users/" + user_uuid + "/journals/" + uuids[i]);
    entry_xhr.send();
}

function onListResponse() {
    if(this.readyState == 4 && this.status == 200) {
        const response = JSON.parse(this.response);
        processAnotherEntry(response.uuids, response.uuids.length - 1);
    }
}

if(timeline != null) {
    const list_xhr = new XMLHttpRequest();
    list_xhr.onreadystatechange = onListResponse;
    list_xhr.open("GET", "/api/users/" + user_uuid + "/journals");
    list_xhr.send();
}

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

function get_cookie_value(target_name) {
    const cookies = document.cookie.split("; ");
    const target_cookie = cookies.find((cookie) => cookie.startsWith(target_name + "="));
    const value = target_cookie?.split("=")[1];
    return value;
}

function get_group_id_from_created(created) {
    return created.slice(0, 10) + " " + created.slice(26);
}

function create_journal_group(datetime) {
    const timeline = document.getElementById("timeline");

    group = document.createElement("div");
    group.id = get_group_id_from_created(datetime);
    group.classList.add("group");

    if(timeline.firstChild == null) {
        timeline.appendChild(group);
    } else {
        timeline.insertBefore(group, timeline.firstChild);
    }

    let timedate_header = document.createElement("h2");
    timedate_header.innerHTML = 
        months[parseInt(datetime.slice(5, 7)) - 1] + 
        " " +
        datetime.slice(8, 10) + 
        " " +
        datetime.slice(0, 4);

    group.appendChild(timedate_header);

    return group;
}

function create_journal_entry(entry_json, entry_uuid) {
    let div = document.createElement("div");

    let link = document.createElement("a");
    link.classList.add("link");
    link.setAttribute("href", "/editor.html?entry-uuid=" + entry_uuid);
    div.appendChild(link);

    let title = document.createElement("h3");
    title.innerHTML = entry_json.title;
    link.appendChild(title);

    let body = document.createElement("p");
    body.innerHTML = entry_json.body;
    div.appendChild(body);

    return div;
}

function processAnotherEntry(uuids, i) {
    if(i < 0) {
        return;
    }

    const entry_xhr = new XMLHttpRequest();

    entry_xhr.onreadystatechange = function() {
        if(this.readyState == 4 && this.status == 200) {
            const entry = JSON.parse(entry_xhr.response);

            const timeline = document.getElementById("timeline");

            let group = document.getElementById(get_group_id_from_created(entry.created));
            if(group == null) {
                group = create_journal_group(entry.created);
            }

            const entry_element = create_journal_entry(entry, uuids[i]);
            group.appendChild(entry_element);

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

const user_uuid = get_cookie_value("user_uuid");

const list_xhr = new XMLHttpRequest();
list_xhr.onreadystatechange = onListResponse;
list_xhr.open("GET", "/api/users/" + user_uuid + "/journals");
list_xhr.send();

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
    group.classList.add("day");

    timeline.appendChild(group);

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

function create_journal_entry(entry_uuid, created, title, body) {
    let div = document.createElement("div");
    div.classList.add("entry");

    let link = document.createElement("a");
    link.classList.add("entry-link");
    link.setAttribute("href", "/editor.html?entry-uuid=" + entry_uuid);
    div.appendChild(link);

    let title_element = document.createElement("h3");
    title_element.innerHTML = title;
    link.appendChild(title_element);

    let body_element = document.createElement("p");
    body_element.innerHTML = body;
    div.appendChild(body_element);

    return div;
}

const user_uuid = get_cookie_value("user_uuid");

const list_xhr = new XMLHttpRequest();
list_xhr.onreadystatechange = function() {
    if(this.readyState == 4 && this.status == 200) {
        const timeline = document.getElementById("timeline");
        const response = JSON.parse(this.response);

        response.uuid.forEach(function(uuid, index) {
            const created = response.created[index];
            const title = response.title[index];
            const body = response.body[index];

            let group = document.getElementById(get_group_id_from_created(response.created[index]));
            if(group == null) {
                group = create_journal_group(response.created[index]);
            }

            const entry_element = create_journal_entry(uuid, created, title, body);
            if(group.children.length == 1) {
                group.appendChild(entry_element);
            } else {
                group.insertBefore(entry_element, group.children[1]);
            }
        });
    }
};
list_xhr.open("GET", "/api/users/" + user_uuid + "/entries");
list_xhr.send();

const query_string = window.location.search;
const url_parameters = new URLSearchParams(query_string);

const entry_uuid = url_parameters.get("entry-uuid");
const form = document.getElementById("editor-form");

if(entry_uuid == null) {
    form.setAttribute("method", "POST");
    form.setAttribute("action", "/api/journals");
} else {
    form.setAttribute("method", "PUT");
    form.setAttribute("action", "/api/journals/" + entry_uuid);
}

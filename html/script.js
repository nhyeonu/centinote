function submitForm(form) {
    let xhr = new XMLHttpRequest();
    xhr.open(form.getAttribute("method"), form.getAttribute("action"));

    xhr.setRequestHeader("Accept", "application/json");
    xhr.setRequestHeader("Content-Type", "application/json");

    var data = {};

    let inputs = form.getElementsByTagName("input");
    for (const input of inputs) {
        data[input.getAttribute("name")] = input.value;
    }

    let textareas = form.getElementsByTagName("textarea");
    for (const textarea of textareas) {
        data[textarea.getAttribute("name")] = textarea.value;
    }
    
    xhr.send(JSON.stringify(data));
}
